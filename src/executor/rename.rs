use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use rayon::prelude::*;
use zip::ZipArchive;

use crate::error::AppError;
use crate::params::rename::RenameParams;
use crate::util::{epub, files, strings};

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
    let rootfile_path = epub::get_rootfile_path(&mut archive)?;
    let mut metadata = get_book_metadata(&mut archive, &rootfile_path)?;
    metadata.format()?;
    println!(
        "rename \"{}\" \"[{}]{}.epub\"",
        path.file_name().unwrap().to_string_lossy(),
        metadata.author.unwrap(),
        metadata.title.unwrap()
    );

    Ok(())
}

#[derive(Debug)]
struct BookMetadata {
    author: Option<String>,
    title: Option<String>,
}

impl BookMetadata {
    fn new() -> Self {
        BookMetadata {
            author: None,
            title: None,
        }
    }

    fn is_filled(&self) -> bool {
        self.author.is_some() && self.title.is_some()
    }

    fn format(&mut self) -> Result<()> {
        self.format_author();
        self.format_title()?;
        Ok(())
    }

    fn format_author(&mut self) {
        if let Some(author) = self.author.as_ref() {
            let author = strings::to_half_width(author);
            let author = strings::replace_unsafe_symbols(&author);
            let author = strings::remove_spaces(&author);
            self.author = Some(author);
        }
    }

    fn format_title(&mut self) -> Result<()> {
        // TODO move file load to initialization
        let target_characters_file = BufReader::new(File::open("regex_raw_strings.txt")?);
        let regex_raw_strings = target_characters_file
            .lines()
            .map(|l| l.unwrap())
            .filter(|l| !l.is_empty())
            .collect::<Vec<String>>();

        if self.title.is_some() {
            let title = self.title.as_ref().unwrap();
            let title = strings::to_half_width(title);
            let title = strings::replace_unsafe_symbols(&title);
            let title = strings::replace_round_brackets(&title);
            let title = strings::remove_characters(&regex_raw_strings, &title);
            let title = strings::pad_volume_number(&title);
            let title = strings::remove_spaces(&title);
            self.title = Some(title);
        }

        Ok(())
    }
}

fn get_book_metadata(archive: &mut ZipArchive<File>, rootfile_path: &str) -> Result<BookMetadata> {
    let rootfile = archive
        .by_name(rootfile_path)
        .or(Err(AppError::BadEPubFile {
            reason: format!("Cannot open rootfile in epub: {rootfile_path}"),
        }))?;
    let mut reader = Reader::from_reader(BufReader::new(rootfile));
    reader
        .trim_text(true)
        .expand_empty_elements(true)
        .check_end_names(false);
    let mut buf = Vec::new();

    let mut result = BookMetadata::new();
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
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"dc:creator" => {
                let mut buf_inner = Vec::new();
                let content = loop {
                    match reader.read_event_into(&mut buf_inner) {
                        Ok(Event::Text(e)) => {
                            break e.unescape()?.to_string();
                        }
                        Ok(Event::End(ref e)) if e.name().as_ref() == b"dc:creator" => {
                            break "".to_string();
                        }
                        Err(e) => {
                            return Err(AppError::XMLReadError {
                                err: e,
                                position: reader.buffer_position(),
                                path: rootfile_path.to_string(),
                            }
                            .into())
                        }
                        _ => {}
                    }
                };
                result.author = Some(content);
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"dc:title" => {
                let mut buf_inner = Vec::new();
                let content = loop {
                    match reader.read_event_into(&mut buf_inner) {
                        Ok(Event::Text(e)) => {
                            break e.unescape()?.to_string();
                        }
                        Ok(Event::End(ref e)) if e.name().as_ref() == b"dc:title" => {
                            break "".to_string();
                        }
                        Err(e) => {
                            return Err(AppError::XMLReadError {
                                err: e,
                                position: reader.buffer_position(),
                                path: rootfile_path.to_string(),
                            }
                            .into())
                        }
                        _ => {}
                    }
                };
                result.title = Some(content);
            }
            _ => {
                if result.is_filled() {
                    break;
                }
            }
        }
    }

    Ok(result)
}
