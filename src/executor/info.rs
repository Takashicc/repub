use std::{fs, path::Path};

use crate::{params::info::InfoParams, services, util::files};

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub fn execute(params: &InfoParams) -> Result<()> {
    files::list_epub_filepaths(Path::new(&params.input))
        .par_iter()
        .for_each(|filepath| {
            if let Err(e) = process(filepath) {
                println!("{e}");
            }
        });

    Ok(())
}

fn process(path: &Path) -> Result<()> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let container_xml = services::epub::read_container_xml(&mut archive)?;
    let rootfile_path = services::epub::get_rootfile_path(&container_xml)?;
    let opf_content = services::epub::read_file_from_archive(&mut archive, &rootfile_path)?;

    let book_type = services::epub::get_book_type(&opf_content)?;
    if let Some(book_type) = book_type {
        println!("{} \"{}\"", book_type, path.to_str().unwrap());
    } else {
        println!("None \"{}\"", path.to_str().unwrap());
    }

    Ok(())
}
