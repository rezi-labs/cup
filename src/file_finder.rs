use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub full_path: PathBuf,
    pub filename: String,
    pub file_type: String,
    #[serde(skip)]
    pub content: String,
}

impl FileInfo {
    fn new(path: PathBuf, content: String) -> Self {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        let file_type = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        FileInfo {
            full_path: path,
            filename,
            file_type,
            content,
        }
    }
}

pub fn find_all_files<P: AsRef<Path>>(dir: P) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    let walker = WalkBuilder::new(dir.as_ref())
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for result in walker {
        let entry = result.map_err(std::io::Error::other)?;
        let path = entry.path();

        if path.is_file() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    files.push(FileInfo::new(path.to_path_buf(), content));
                }
                Err(_) => {
                    files.push(FileInfo::new(
                        path.to_path_buf(),
                        String::from("[Binary file or read error]"),
                    ));
                }
            }
        }
    }

    Ok(files)
}
