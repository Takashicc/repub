use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

pub fn get_filepaths(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        WalkDir::new(path)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .unwrap_or(OsStr::new(""))
                    .to_ascii_lowercase()
                    == "epub"
            })
            .map(|v| v.into_path())
            .collect::<Vec<PathBuf>>()
    }
}
