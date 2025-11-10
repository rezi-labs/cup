use crate::{
    file_finder::FileInfo,
    init::{Config, Remote, Tag, Target},
};

/// The default comment pattern to search for in files
pub const CUP_COMMENT: &str = "[cup]";

/// Represents a target found in a file with cup comment configuration
pub struct FileTarget {
    pub file: FileInfo,
    pub row: i128,
    pub extracted_config: Target,
}

/// Parses a line containing a cup comment and extracts target configuration
///
/// # Arguments
/// * `file_info` - Information about the file being processed
/// * `line` - The line of text to parse
/// * `row` - The row number of the line in the file
/// * `config` - The application configuration
///
/// # Returns
/// * `Some(FileTarget)` if the line contains a valid cup comment with version
/// * `None` if the line doesn't contain a cup comment or has no valid version
pub fn parse_cup_line(
    file_info: &FileInfo,
    line: &str,
    row: i128,
    config: &Config,
) -> Option<FileTarget> {
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
        extracted_config: target,
    });
}

/// Searches through all files and extracts targets with cup comments
///
/// # Arguments
/// * `files` - Vector of file information to search through
/// * `config` - The application configuration
///
/// # Returns
/// * Vector of `FileTarget` objects representing all found cup comment targets
pub fn find_cup_targets(files: &[FileInfo], config: &Config) -> Vec<FileTarget> {
    let mut targets = Vec::new();

    for file_info in files {
        for (row, line) in file_info.content.lines().enumerate() {
            if let Some(target) = parse_cup_line(file_info, line, row as i128, config) {
                targets.push(target);
            }
        }
    }

    targets
}
