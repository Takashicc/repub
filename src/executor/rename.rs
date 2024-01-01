use core::panic;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::{fs, path::PathBuf};

use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::params::rename::RenameParams;

pub fn execute(params: &RenameParams) {
    let filepaths = walkdir::WalkDir::new(&params.target_dir)
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
        .collect::<Vec<PathBuf>>();

    for filepath in filepaths.iter() {
        let file = fs::File::open(filepath).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let rootfile_path = get_rootfile_path(&mut archive);
        let metadata = get_book_metadata(&mut archive, &rootfile_path);
        println!("{:?}", metadata);
    }
}

fn get_rootfile_path(archive: &mut ZipArchive<File>) -> String {
    let container = archive.by_name("META-INF/container.xml").unwrap();
    let mut reader = Reader::from_reader(BufReader::new(container));
    reader
        .trim_text(true)
        .expand_empty_elements(true)
        .check_end_names(false);
    let mut buf = Vec::new();
    let rootfile_path: String = loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => panic!("Cannot find rootfile"),
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"rootfile" {
                    let rootfile_path = e
                        .try_get_attribute("full-path")
                        .unwrap()
                        .unwrap()
                        .unescape_value()
                        .unwrap();
                    break rootfile_path.to_string();
                }
            }
            _ => {}
        }
    };

    rootfile_path
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
}

fn get_book_metadata(archive: &mut ZipArchive<File>, rootfile_path: &str) -> BookMetadata {
    let rootfile = archive.by_name(rootfile_path).unwrap();
    let mut reader = Reader::from_reader(BufReader::new(rootfile));
    reader
        .trim_text(true)
        .expand_empty_elements(true)
        .check_end_names(false);
    let mut buf = Vec::new();

    let mut result = BookMetadata::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"dc:creator" => {
                let mut buf_inner = Vec::new();
                let content = loop {
                    match reader.read_event_into(&mut buf_inner) {
                        Ok(Event::Text(e)) => {
                            break e.unescape().unwrap().to_string();
                        }
                        Ok(Event::End(ref e)) if e.name().as_ref() == b"dc:creator" => {
                            break "".to_string();
                        }
                        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
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
                            break e.unescape().unwrap().to_string();
                        }
                        Ok(Event::End(ref e)) if e.name().as_ref() == b"dc:title" => {
                            break "".to_string();
                        }
                        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
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

    result
}