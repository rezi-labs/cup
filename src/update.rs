use std::{fs::File, path::Path};

use ignore::WalkBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};

use crate::{
    file_finder::{self, FileInfo},
    init::{Config, Remote, Target},
};

pub fn update(config: Config) {
    find_cups(config).par_iter().for_each(|t| {
        single(t);
    });
}

fn single(target: &FileTarget) {
    let latest_tag = match target.extracted_config.tag.remote_type {
        Remote::GitHub => get_latest_tag_from_github(target),
    };

    let clean = clean_tag(latest_tag);
}

struct FileTarget {
    file: FileInfo,
    row: i128,
    col: i128,
    extracted_config: Target,
}

fn do_cups(config: Config) {
    let files = file_finder::find_all_files(".")
        .iter()
        .flat_map(|f| f)
        .collect::<Vec<&FileInfo>>();

    files.iter().map(|fi|{
        todo!("if you find [cup] GitHub rezi-labs/rezi-web, deserialize the the type and the depending on the type: here GitHub. Go and get the latest release from there and update whatever number is befor with it: test = 1.3.0 // ")
        todo!("the line might look like this: test = 1.3.0 // [cup] GitHub rezi-labs/rezi-web, however the test = 1.3.0 // might be different")
        )
    
    vec![]
}

const CUP_COMMENT: &str = "[cup]";

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
