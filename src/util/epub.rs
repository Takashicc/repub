use std::{fs::File, io::BufReader};

use anyhow::Result;
use quick_xml::{events::Event, Reader};
use zip::ZipArchive;

use crate::error::AppError;

pub fn get_rootfile_path(archive: &mut ZipArchive<File>) -> Result<String> {
    let container_xml_path = "META-INF/container.xml";
    let container = archive
        .by_name(container_xml_path)
        .or(Err(AppError::BadEPubFile {
            reason: "Cannot find META-INF/container.xml".to_string(),
        }))?;
    let mut reader = Reader::from_reader(BufReader::new(container));
    reader
        .trim_text(true)
        .expand_empty_elements(true)
        .check_end_names(false);
    let mut buf = Vec::new();
    let rootfile_path: String = loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(AppError::XMLReadError {
                    err: e,
                    position: reader.buffer_position(),
                    path: container_xml_path.to_string(),
                }
                .into())
            }
            Ok(Event::Eof) => unreachable!(),
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"rootfile" {
                    let rootfile_path = match e.try_get_attribute("full-path") {
                        Ok(Some(v)) => v.unescape_value().or(Err(AppError::BadEPubFile {
                            reason: "Failed to unescape full-path attribute".to_string(),
                        }))?,
                        _ => Err(AppError::BadEPubFile {
                            reason: "Cannot find full-path attribute".to_string(),
                        })?,
                    };

                    break rootfile_path.to_string();
                }
            }
            _ => {}
        }
    };

    Ok(rootfile_path)
}
