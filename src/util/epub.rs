use std::{fs::File, io::BufReader};

use anyhow::Result;
use quick_xml::{events::Event, Reader};
use zip::ZipArchive;

use crate::error::AppError;

pub fn get_rootfile_path(archive: &mut ZipArchive<File>) -> Result<String> {
    let container = match archive.by_name("META-INF/container.xml") {
        Err(_) => {
            return Err(AppError::BadEPubFile {
                reason: "Cannot find META-INF/container.xml".to_string(),
            }
            .into())
        }
        Ok(v) => v,
    };
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
                    let rootfile_path = match e.try_get_attribute("full-path") {
                        Ok(Some(v)) => match v.unescape_value() {
                            Ok(v) => v,
                            Err(e) => {
                                return Err(AppError::BadEPubFile {
                                    reason: "Failed to unescape full-path attribute".to_string(),
                                }
                                .into())
                            }
                        },
                        _ => {
                            return Err(AppError::BadEPubFile {
                                reason: "Cannot find full-path attribute".to_string(),
                            }
                            .into())
                        }
                    };

                    break rootfile_path.to_string();
                }
            }
            _ => {}
        }
    };

    Ok(rootfile_path)
}
