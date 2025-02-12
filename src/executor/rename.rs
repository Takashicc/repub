use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rayon::prelude::*;

use crate::params::rename::RenameParams;
use crate::services;
use crate::util::files;

pub fn execute(params: &RenameParams) -> Result<()> {
    files::list_epub_filepaths(Path::new(&params.input))
        .par_iter()
        .for_each(|filepath| {
            if let Err(e) = process(filepath) {
                println!("{e}");
            };
        });

    Ok(())
}

fn process(path: &Path) -> Result<()> {
    let file = fs::File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.to_string_lossy()))?;
    let mut archive = zip::ZipArchive::new(file).with_context(|| {
        format!(
            "Failed to open file as zip archive: {}",
            path.to_string_lossy()
        )
    })?;

    let container_xml = services::epub::read_container_xml(&mut archive)?;
    let rootfile_path = services::epub::get_rootfile_path(&container_xml)?;
    let opf_content = services::epub::read_file_from_archive(&mut archive, &rootfile_path)?;
    let mut metadata = services::epub::get_book_metadata(&opf_content)?;
    metadata.format()?;
    println!(
        "rename \"{}\" \"[{}]{}.epub\"",
        path.file_name().unwrap().to_string_lossy(),
        metadata.author.unwrap(),
        metadata.title.unwrap()
    );

    Ok(())
}
