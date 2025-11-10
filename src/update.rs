use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    file_finder::{self, FileInfo},
    init::{Config, Remote, Tag, Target},
};

// Lazy static regex patterns compiled only once at startup
// Pattern 1: name = version // comment or name = version # comment
// More permissive version pattern: allows .25.0, 1.0, 1.2.3.4, etc.
static VERSION_EXTRACT_RE_1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+)\s*=\s*([0-9]*\.?[0-9]+(?:\.[0-9]+)*)")
        .expect("Failed to compile version extract regex 1")
});
static VERSION_REPLACE_RE_1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*=\s*)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 1")
});

// Pattern 2: name := version // comment or name := version # comment
static VERSION_EXTRACT_RE_2: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+)\s*:=\s*([0-9]*\.?[0-9]+(?:\.[0-9]+)*)")
        .expect("Failed to compile version extract regex 2")
});
static VERSION_REPLACE_RE_2: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*:=\s*)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 2")
});

// Pattern 3: name: version // comment or name: version # comment
static VERSION_EXTRACT_RE_3: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+):\s*([0-9]*\.?[0-9]+(?:\.[0-9]+)*)")
        .expect("Failed to compile version extract regex 3")
});
static VERSION_REPLACE_RE_3: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+:\s*)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 3")
});

// Pattern 4: "name:version" // comment
static VERSION_EXTRACT_RE_4: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#""(\w+):([0-9]*\.?[0-9]+(?:\.[0-9]+)*)""#)
        .expect("Failed to compile version extract regex 4")
});
static VERSION_REPLACE_RE_4: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"("(\w+):)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(")(\s*(?://|#).*)"#)
        .expect("Failed to compile version replace regex 4")
});

// Pattern 5: "name": "version" // comment
static VERSION_EXTRACT_RE_5: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#""(\w+)":\s*"([0-9]*\.?[0-9]+(?:\.[0-9]+)*)""#)
        .expect("Failed to compile version extract regex 5")
});
static VERSION_REPLACE_RE_5: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"("(\w+)":\s*")([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(")(\s*(?://|#).*)"#)
        .expect("Failed to compile version replace regex 5")
});

// Pattern 6: name = 'version' // comment (single quotes)
static VERSION_EXTRACT_RE_6: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+)\s*=\s*'([0-9]*\.?[0-9]+(?:\.[0-9]+)*)'")
        .expect("Failed to compile version extract regex 6")
});
static VERSION_REPLACE_RE_6: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*=\s*')([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 6")
});

// Pattern 7: name := 'version' // comment (single quotes)
static VERSION_EXTRACT_RE_7: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+)\s*:=\s*'([0-9]*\.?[0-9]+(?:\.[0-9]+)*)'")
        .expect("Failed to compile version extract regex 7")
});
static VERSION_REPLACE_RE_7: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*:=\s*')([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 7")
});

// Pattern 8: name: 'version' // comment (single quotes)
static VERSION_EXTRACT_RE_8: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+):\s*'([0-9]*\.?[0-9]+(?:\.[0-9]+)*)'")
        .expect("Failed to compile version extract regex 8")
});
static VERSION_REPLACE_RE_8: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+:\s*')([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 8")
});

// Pattern 9: 'name:version' // comment (single quotes)
static VERSION_EXTRACT_RE_9: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"'(\w+):([0-9]*\.?[0-9]+(?:\.[0-9]+)*)'")
        .expect("Failed to compile version extract regex 9")
});
static VERSION_REPLACE_RE_9: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"('(\w+):)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 9")
});

// Pattern 10: 'name': 'version' // comment (single quotes)
static VERSION_EXTRACT_RE_10: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"'(\w+)':\s*'([0-9]*\.?[0-9]+(?:\.[0-9]+)*)'")
        .expect("Failed to compile version extract regex 10")
});
static VERSION_REPLACE_RE_10: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"('(\w+)':\s*')([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 10")
});

pub fn update(config: Config) {
    let targets = do_cups(config);
    
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
        let latest_tag = match target.extracted_config.tag.remote_type {
            Remote::GitHub => match get_latest_tag_from_github(target) {
                Ok(tag) => tag,
                Err(e) => {
                    eprintln!("Error getting latest tag for {}: {}", target.extracted_config.tag.remote_tag, e);
                    continue;
                }
            },
        };
        
        let clean_version = clean_tag(latest_tag);
        
        if target.row as usize >= lines.len() {
            eprintln!("Row index {} out of bounds for file {}", target.row, file_path.display());
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
            println!("Successfully updated {} lines in {}", updated_count, file_path.display());
        }
    }
}

struct FileTarget {
    file: FileInfo,
    row: i128,
    #[expect(unused)]
    col: i128,
    extracted_config: Target,
}

fn do_cups(config: Config) -> Vec<FileTarget> {
    let files = match file_finder::find_all_files(".") {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error finding files: {e}");
            return vec![];
        }
    };

    let mut targets = Vec::new();

    for file_info in &files {
        for (row, line) in file_info.content.lines().enumerate() {
            if let Some(target) = parse_cup_line(file_info, line, row as i128, &config) {
                targets.push(target);
            }
        }
    }

    targets
}

const CUP_COMMENT: &str = "[cup]";

fn parse_cup_line(file_info: &FileInfo, line: &str, row: i128, config: &Config) -> Option<FileTarget> {
    if !line.contains(CUP_COMMENT) {
        return None;
    }

    // Find the position of [cup] comment
    let cup_pos = line.find(CUP_COMMENT)?;

    // Extract the part after [cup]
    let after_cup = &line[cup_pos + CUP_COMMENT.len()..].trim();

    // Determine the remote type and owner/repo
    let (remote_type, owner_repo) = if after_cup.starts_with("GitHub") {
        // Explicit GitHub type specified
        let github_part = after_cup.strip_prefix("GitHub")?.trim();
        let owner_repo = github_part.split_whitespace().next()?;
        (Remote::GitHub, owner_repo)
    } else if !after_cup.is_empty() {
        // No explicit type, use remote_default and treat the whole string as owner/repo
        let owner_repo = after_cup.split_whitespace().next()?;
        match config.remote_default.as_str() {
            "GitHub" => (Remote::GitHub, owner_repo),
            // Add more cases here when more remote types are supported
            _ => (Remote::GitHub, owner_repo), // fallback to GitHub for unknown defaults
        }
    } else {
        // Empty after [cup], nothing to parse
        return None;
    };

    // Find the version number before the comment
    let before_comment = &line[..cup_pos].trim();
    if let Some(version_info) = extract_version_from_line(before_comment) {
        let target = Target {
            name: format!("{}:{}", file_info.full_path.display(), row + 1),
            tag: Tag {
                remote_tag: owner_repo.to_string(),
                remote_type,
            },
        };

        return Some(FileTarget {
            file: file_info.clone(),
            row,
            col: version_info.col,
            extracted_config: target,
        });
    }

    None
}

#[derive(Debug)]
struct VersionInfo {
    #[expect(unused)]
    version: String,
    col: i128,
}

fn extract_version_from_line(line: &str) -> Option<VersionInfo> {
    // Try each pattern in order
    if let Some(captures) = VERSION_EXTRACT_RE_1.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_2.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_3.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_4.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_5.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_6.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_7.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_8.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_9.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    if let Some(captures) = VERSION_EXTRACT_RE_10.captures(line) {
        let version = captures.get(2)?.as_str().to_string();
        let col = captures.get(2)?.start() as i128;
        return Some(VersionInfo { version, col });
    }

    None
}

fn clean_tag(inp: String) -> String {
    if inp.starts_with(['V', 'v']) {
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

fn get_latest_tag_from_github(target: &FileTarget) -> Result<String, Box<dyn std::error::Error>> {
    let sh = xshell::Shell::new()?;
    let owner_and_repo = target.extracted_config.tag.remote_tag.clone();

    let res = xshell::cmd!(sh, "gh release view --repo {owner_and_repo} --json tagName")
        .read()
        .map_err(|e| format!("Failed to get release for {}: {}", owner_and_repo, e))?;

    let tag_name: LatestTag = serde_json::from_str(&res)
        .map_err(|e| format!("Failed to parse release data for {}: {}", owner_and_repo, e))?;

    Ok(tag_name.tag_name)
}

fn try_replace_version_in_line(line: &str, new_version: &str) -> Option<String> {
    if let Some(updated) = try_replace_pattern_1(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_2(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_3(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_4(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_5(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_6(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_7(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_8(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_9(line, new_version) {
        Some(updated)
    } else if let Some(updated) = try_replace_pattern_10(line, new_version) {
        Some(updated)
    } else {
        None
    }
}

fn try_replace_pattern_1(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_1.is_match(line) {
        Some(
            VERSION_REPLACE_RE_1
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}", &caps[1], new_version, &caps[3])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_2(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_2.is_match(line) {
        Some(
            VERSION_REPLACE_RE_2
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}", &caps[1], new_version, &caps[3])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_3(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_3.is_match(line) {
        Some(
            VERSION_REPLACE_RE_3
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}", &caps[1], new_version, &caps[3])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_4(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_4.is_match(line) {
        Some(
            VERSION_REPLACE_RE_4
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}{}", &caps[1], new_version, &caps[4], &caps[5])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_5(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_5.is_match(line) {
        Some(
            VERSION_REPLACE_RE_5
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}{}", &caps[1], new_version, &caps[4], &caps[5])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_6(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_6.is_match(line) {
        Some(
            VERSION_REPLACE_RE_6
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}{}", &caps[1], new_version, &caps[3], &caps[4])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_7(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_7.is_match(line) {
        Some(
            VERSION_REPLACE_RE_7
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}{}", &caps[1], new_version, &caps[3], &caps[4])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_8(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_8.is_match(line) {
        Some(
            VERSION_REPLACE_RE_8
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}{}", &caps[1], new_version, &caps[3], &caps[4])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_9(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_9.is_match(line) {
        Some(
            VERSION_REPLACE_RE_9
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}{}", &caps[1], new_version, &caps[4], &caps[5])
                })
                .to_string(),
        )
    } else {
        None
    }
}

fn try_replace_pattern_10(line: &str, new_version: &str) -> Option<String> {
    if VERSION_REPLACE_RE_10.is_match(line) {
        Some(
            VERSION_REPLACE_RE_10
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{}{}{}{}", &caps[1], new_version, &caps[4], &caps[5])
                })
                .to_string(),
        )
    } else {
        None
    }
}
