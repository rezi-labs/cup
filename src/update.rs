use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde::Deserialize;

use crate::{
    file_finder::{self, FileInfo},
    init::{Config, Remote, Tag, Target},
};

// Lazy static regex patterns compiled only once at startup
static VERSION_EXTRACT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+)\s*=\s*([0-9]+\.[0-9]+\.[0-9]+)").expect("Failed to compile version extract regex")
});

static VERSION_REPLACE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*=\s*)([0-9]+\.[0-9]+\.[0-9]+)(\s*//.*)").expect("Failed to compile version replace regex")
});

pub fn update(config: Config) {
    do_cups(config).par_iter().for_each(|t| {
        single(t);
    });
}

fn single(target: &FileTarget) {
    let latest_tag = match target.extracted_config.tag.remote_type {
        Remote::GitHub => get_latest_tag_from_github(target),
    };

    let clean_version = clean_tag(latest_tag);

    // Update the version in the file
    if let Err(e) = update_version_in_file(target, &clean_version) {
        eprintln!(
            "Error updating file {}: {}",
            target.file.full_path.display(),
            e
        );
    } else {
        println!(
            "Updated {} to version {}",
            target.file.full_path.display(),
            clean_version
        );
    }
}

struct FileTarget {
    file: FileInfo,
    row: i128,
    col: i128,
    extracted_config: Target,
}

fn do_cups(_config: Config) -> Vec<FileTarget> {
    let files = match file_finder::find_all_files(".") {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error finding files: {}", e);
            return vec![];
        }
    };

    let mut targets = Vec::new();

    for file_info in &files {
        for (row, line) in file_info.content.lines().enumerate() {
            if let Some(target) = parse_cup_line(file_info, line, row as i128) {
                targets.push(target);
            }
        }
    }

    targets
}

const CUP_COMMENT: &str = "[cup]";

fn parse_cup_line(file_info: &FileInfo, line: &str, row: i128) -> Option<FileTarget> {
    if !line.contains(CUP_COMMENT) {
        return None;
    }

    // Find the position of [cup] comment
    let cup_pos = line.find(CUP_COMMENT)?;

    // Extract the part after [cup]
    let after_cup = &line[cup_pos + CUP_COMMENT.len()..].trim();

    // Check if it's a GitHub reference
    if after_cup.starts_with("GitHub") {
        let github_part = after_cup.strip_prefix("GitHub")?.trim();

        // Extract owner/repo (format: owner/repo)
        let owner_repo = github_part.split_whitespace().next()?;

        // Find the version number before the comment
        let before_comment = &line[..cup_pos].trim();
        if let Some(version_info) = extract_version_from_line(before_comment) {
            let target = Target {
                name: format!("{}:{}", file_info.full_path.display(), row + 1),
                tag: Tag {
                    remote_tag: owner_repo.to_string(),
                    remote_type: Remote::GitHub,
                },
            };

            return Some(FileTarget {
                file: file_info.clone(),
                row,
                col: version_info.col,
                extracted_config: target,
            });
        }
    }

    None
}

#[derive(Debug)]
struct VersionInfo {
    version: String,
    col: i128,
}

fn extract_version_from_line(line: &str) -> Option<VersionInfo> {
    if let Some(captures) = VERSION_EXTRACT_RE.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;

        return Some(VersionInfo { version, col });
    }

    None
}

fn clean_tag(inp: String) -> String {
    if inp.starts_with(&['V', 'v']) {
        inp.replace("v", "").replace("V", "")
    } else {
        inp
    }
}

#[derive(Debug, Deserialize)]
struct LatestTag {
    #[serde(alias = "tagName")]
    tag_name: String,
}

fn get_latest_tag_from_github(target: &FileTarget) -> String {
    let sh = xshell::Shell::new().unwrap();
    let owner_and_repo = target.extracted_config.tag.remote_tag.clone();

    let res = xshell::cmd!(sh, "gh release view --repo {owner_and_repo} --json tagName")
        .read()
        .unwrap();

    let tag_name: LatestTag = serde_json::from_str(&res).unwrap();

    tag_name.tag_name
}

fn update_version_in_file(
    target: &FileTarget,
    new_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the current file content
    let content = std::fs::read_to_string(&target.file.full_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if target.row as usize >= lines.len() {
        return Err("Row index out of bounds".into());
    }

    let line = lines[target.row as usize];

    // Use lazy regex to replace the version number
    let updated_line = VERSION_REPLACE_RE.replace(line, |caps: &regex::Captures| {
        format!("{}{}{}", &caps[1], new_version, &caps[3])
    });

    // Replace the line in the content
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    new_lines[target.row as usize] = updated_line.to_string();
    let new_content = new_lines.join("\n");

    // Write back to file
    std::fs::write(&target.file.full_path, new_content)?;

    Ok(())
}
