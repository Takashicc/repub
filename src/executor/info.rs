use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use crate::{
    error::AppError,
    params::info::InfoParams,
    util::{epub, files},
};

use anyhow::Result;
use quick_xml::{events::Event, Reader};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use zip::ZipArchive;

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
    let rootfile_path = epub::get_rootfile_path(&mut archive)?;
    let book_type = get_book_type(&mut archive, &rootfile_path)?;
    if let Some(book_type) = book_type {
        println!("{} \"{}\"", book_type, path.to_str().unwrap());
    } else {
        println!("None \"{}\"", path.to_str().unwrap());
    }

    Ok(())
}

fn get_book_type(archive: &mut ZipArchive<File>, rootfile_path: &str) -> Result<Option<String>> {
    let rootfile = archive.by_name(rootfile_path)?;
    let mut reader = Reader::from_reader(BufReader::new(rootfile));
    reader
        .trim_text(true)
        .expand_empty_elements(true)
        .check_end_names(false);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(AppError::XMLReadError {
                    err: e,
                    position: reader.buffer_position(),
                    path: rootfile_path.to_string(),
                }
                .into())
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"meta" => {
                let mut is_book_type = false;
                for attr in e.attributes() {
                    let attr = attr?;
                    if attr.key.as_ref() == b"name" && attr.value.as_ref() == b"book-type" {
                        is_book_type = true;
                        continue;
                    }
                    if is_book_type && attr.key.as_ref() == b"content" {
                        return Ok(Some(String::from_utf8(attr.value.as_ref().to_vec())?));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(None)
}
