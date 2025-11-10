use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    cup_parser::{FileTarget, find_cup_targets},
    file_finder::{self},
    init::Config,
    version_extractor::{clean_tag, try_replace_version_in_line},
};

pub fn update(config: Config) {
    let targets = do_cups(&config);

    // Group targets by file path to handle multiple targets per file
    let mut targets_by_file: HashMap<String, Vec<FileTarget>> = HashMap::new();
    for target in targets {
        let file_path = target.file.full_path.to_string_lossy().to_string();
        targets_by_file.entry(file_path).or_default().push(target);
    }

    // Process each file with all its targets
    targets_by_file.par_iter().for_each(|(_, file_targets)| {
        process_file_targets(file_targets);
    });
}

fn process_file_targets(targets: &[FileTarget]) {
    if targets.is_empty() {
        return;
    }

    let file_path = &targets[0].file.full_path;

    // Read file content once
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path.display(), e);
            return;
        }
    };

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut updated_count = 0;

    // Process each target and update the corresponding line
    for target in targets {
        let latest_tag = match get_latest_tag_from_github(target) {
            Ok(tag) => tag,
            Err(e) => {
                eprintln!(
                    "Error getting latest tag for {}: {}",
                    target.extracted_config.tag.remote_tag, e
                );
                continue;
            }
        };

        let clean_version = clean_tag(latest_tag);

        if target.row as usize >= lines.len() {
            eprintln!(
                "Row index {} out of bounds for file {}",
                target.row,
                file_path.display()
            );
            continue;
        }

        let line = &lines[target.row as usize];

        if let Some(updated_line) = try_replace_version_in_line(line, &clean_version) {
            lines[target.row as usize] = updated_line;
            updated_count += 1;
            println!(
                "Updated {}:{} to version {}",
                file_path.display(),
                target.row + 1,
                clean_version
            );
        } else {
            eprintln!(
                "No matching pattern found for version replacement in {}:{}",
                file_path.display(),
                target.row + 1
            );
        }
    }

    // Write the updated content back to file if any updates were made
    if updated_count > 0 {
        let new_content = lines.join("\n");
        if let Err(e) = std::fs::write(file_path, new_content) {
            eprintln!("Error writing file {}: {}", file_path.display(), e);
        } else {
            println!(
                "Successfully updated {} lines in {}",
                updated_count,
                file_path.display()
            );
            println!();
        }
    }
}

fn do_cups(config: &Config) -> Vec<FileTarget> {
    let files = match file_finder::find_all_files(".") {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error finding files: {e}");
            return vec![];
        }
    };

    find_cup_targets(&files, config)
}

#[derive(Debug, Deserialize)]
struct LatestTag {
    #[serde(alias = "tagName")]
    tag_name: String,
}

fn get_latest_tag_from_github(target: &FileTarget) -> Result<String, Box<dyn std::error::Error>> {
    let sh = xshell::Shell::new()?;
    let owner_and_repo = target.extracted_config.tag.remote_tag.clone();

    // First try to get the latest release
    if let Ok(res) =
        xshell::cmd!(sh, "gh release view --repo {owner_and_repo} --json tagName").read()
    {
        if let Ok(tag_name) = serde_json::from_str::<LatestTag>(&res) {
            return Ok(tag_name.tag_name);
        }
    }

    // If release fails, try to get the latest tag
    println!("release not found");
    let res = xshell::cmd!(sh, "gh api repos/{owner_and_repo}/tags --jq '.[0].name'")
        .read()
        .map_err(|e| format!("Failed to get tags for {owner_and_repo}: {e}"))?;

    let tag_name = res.trim().to_string();
    if tag_name.is_empty() {
        return Err(format!("No tags found for repository {owner_and_repo}").into());
    }

    Ok(tag_name)
}
