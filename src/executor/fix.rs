use std::{fs, io::Read, path::Path};

use walkdir::{DirEntry, WalkDir};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

use std::io::Write;

use crate::params::fix::FixParams;

pub fn execute(params: &FixParams) {
    for entry in WalkDir::new(&params.input_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir() && is_epub(e))
    {
        let temp_dir = tempfile::tempdir_in(&params.input_dir).unwrap();

        let file_path = entry.path();
        println!("Opening file: {:?}", file_path);
        let reader = fs::File::open(file_path).unwrap();
        let mut archive = ZipArchive::new(reader).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let out_path = temp_dir.path().join(file.name());

            if let Some(parent_dir) = out_path.parent() {
                if !parent_dir.exists() {
                    fs::create_dir_all(parent_dir).unwrap();
                }
            }

            if file.name().to_lowercase().ends_with(".xhtml") {
                let mut modified_file = fs::File::create(&out_path).unwrap();
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();

                let modified_content = content.replace("&nbsp;", "&#160;");

                write!(modified_file, "{}", modified_content).unwrap();
            } else {
                let mut out_file = fs::File::create(&out_path).unwrap();
                std::io::copy(&mut file, &mut out_file).unwrap();
            }
        }

        let output_dir = Path::new(params.input_dir.as_str()).join("output");
        if !output_dir.exists() {
            fs::create_dir(&output_dir).unwrap();
        }
        let output_file = output_dir.join(entry.file_name());
        let writer = fs::File::create(output_file).unwrap();
        let mut zip = ZipWriter::new(writer);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);
        for entry in WalkDir::new(&temp_dir).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                zip.start_file(
                    path.strip_prefix(&temp_dir).unwrap().to_str().unwrap(),
                    options,
                )
                .unwrap();
            }
            // TODO On windows, this will fail because of the permission denied error
            let mut file = fs::File::open(path).unwrap();
            std::io::copy(&mut file, &mut zip).unwrap();
        }
    }
}

fn is_epub(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_string_lossy()
        .to_lowercase()
        .ends_with(".epub")
}
