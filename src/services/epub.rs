use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead as _, BufReader, Read, Seek},
};

use anyhow::{anyhow, Context, Result};
use quick_xml::{events::Event, Reader};
use zip::ZipArchive;

use crate::util;

const CONTAINER_PATH: &str = "META-INF/container.xml";

const TAG_ROOTFILES: &[u8] = b"rootfiles";
const TAG_ROOTFILE: &[u8] = b"rootfile";
const TAG_METADATA: &[u8] = b"metadata";
const TAG_DC_TITLE: &[u8] = b"dc:title";
const TAG_DC_CREATOR: &[u8] = b"dc:creator";
const TAG_META: &[u8] = b"meta";
const TAG_MANIFEST: &[u8] = b"manifest";
const TAG_ITEM: &[u8] = b"item";
const TAG_SPINE: &[u8] = b"spine";
const TAG_ITEMREF: &[u8] = b"itemref";

pub fn read_container_xml<T>(archive: &mut ZipArchive<T>) -> Result<String>
where
    T: Read + Seek,
{
    read_file_from_archive(archive, CONTAINER_PATH)
}

pub fn read_file_from_archive<T>(archive: &mut ZipArchive<T>, filepath: &str) -> Result<String>
where
    T: Read + Seek,
{
    let mut res = String::new();
    archive
        .by_name(filepath)
        .with_context(|| format!("Failed to find {} from archive", filepath))?
        .read_to_string(&mut res)
        .with_context(|| format!("Failed to read {} content from archive", filepath))?;
    Ok(res)
}

pub fn get_rootfile_path(container_content: &str) -> Result<String> {
    let mut reader = Reader::from_str(container_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut in_rootfiles = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(anyhow!(e)),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => match e.local_name().into_inner() {
                TAG_ROOTFILES => in_rootfiles = true,
                TAG_ROOTFILE => {
                    if !in_rootfiles {
                        continue;
                    }

                    for attr in e.attributes() {
                        let attr = attr?;
                        if attr.key.into_inner() == b"full-path" {
                            return Ok(String::from_utf8(attr.value.to_vec())?);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => {
                if e.local_name().into_inner() == TAG_ROOTFILES {
                    in_rootfiles = false
                }
            }
            _ => {}
        }
    }

    Err(anyhow!("rootfileパスが見つかりませんでした"))
}

// https://imagedrive.github.io/spec/epub30-publications.xhtml#sec-item-elem
pub struct ManifestItem {
    pub href: String,
    pub media_type: String,
    pub fallback: Option<String>,
}

pub struct SpineItemRef {
    pub idref: String,
}

pub struct OpfData {
    pub manifest_items: HashMap<String, ManifestItem>,
    pub spine_item_refs: Vec<SpineItemRef>,
}

impl OpfData {
    fn new() -> Self {
        Self {
            manifest_items: HashMap::new(),
            spine_item_refs: Vec::new(),
        }
    }
}

// OPFファイルをパースして、manifestとspineの情報を取得する
pub fn parse_opf(opf_content: &str) -> Result<OpfData> {
    let mut reader = Reader::from_str(opf_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut in_manifest = false;
    let mut in_spine = false;
    let mut res = OpfData::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(anyhow!(e)),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => match e.local_name().into_inner() {
                TAG_MANIFEST => in_manifest = true,
                TAG_SPINE => in_spine = true,

                // Extract manifest.item
                TAG_ITEM => {
                    if !in_manifest {
                        continue;
                    }

                    let mut id = String::new();
                    let mut manifest_item = ManifestItem {
                        href: "".to_string(),
                        media_type: "".to_string(),
                        fallback: None,
                    };
                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.into_inner() {
                            b"id" => id = String::from_utf8(attr.value.to_vec())?,
                            b"href" => manifest_item.href = String::from_utf8(attr.value.to_vec())?,
                            b"media-type" => {
                                manifest_item.media_type = String::from_utf8(attr.value.to_vec())?
                            }
                            b"fallback" => {
                                manifest_item.fallback =
                                    Some(String::from_utf8(attr.value.to_vec())?)
                            }
                            _ => {}
                        }
                    }
                    res.manifest_items.insert(id, manifest_item);
                }

                // Extract spine.itemref
                TAG_ITEMREF => {
                    if !in_spine {
                        continue;
                    }

                    for attr in e.attributes() {
                        let attr = attr?;
                        if attr.key.into_inner() == b"idref" {
                            let idref = String::from_utf8(attr.value.to_vec())?;
                            res.spine_item_refs.push(SpineItemRef { idref });
                            break;
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.local_name().into_inner() {
                TAG_MANIFEST => in_manifest = false,
                TAG_SPINE => in_manifest = false,
                _ => {}
            },
            _ => {}
        }
    }

    Ok(res)
}

// OPFファイルの内容から、book-typeを取得する
pub fn get_book_type(opf_content: &str) -> Result<Option<String>> {
    let mut reader = Reader::from_str(opf_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut in_metadata = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(anyhow!(e)),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => match e.local_name().into_inner() {
                TAG_METADATA => in_metadata = true,
                TAG_META => {
                    if !in_metadata {
                        continue;
                    }

                    let mut name = None;
                    let mut content = None;
                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.into_inner() {
                            b"name" => name = Some(String::from_utf8(attr.value.to_vec())?),
                            b"content" => content = Some(String::from_utf8(attr.value.to_vec())?),
                            _ => {}
                        }
                    }

                    if let Some("book-type") = name.as_deref() {
                        return Ok(content);
                    }
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => {
                if e.local_name().into_inner() == TAG_METADATA {
                    in_metadata = false
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

// OPFファイルの内容から、漫画かどうかを判定する
// metaタグからbook-typeがcomicかどうかを判定する
pub fn is_comic(opf_content: &str) -> Result<bool> {
    let book_type = get_book_type(opf_content)
        .with_context(|| "Failed to get book type from OPF content".to_string())?;
    if let Some(book_type) = book_type {
        Ok(book_type == "comic")
    } else {
        Ok(false)
    }
}

#[derive(Debug)]
pub struct BookMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
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

    pub fn format(&mut self) -> Result<()> {
        self.format_author();
        self.format_title()?;
        Ok(())
    }

    fn format_author(&mut self) {
        if let Some(author) = self.author.as_ref() {
            let author = util::strings::to_half_width(author);
            let author = util::strings::replace_unsafe_symbols(&author);
            let author = util::strings::remove_spaces(&author);
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
            let title = util::strings::to_half_width(title);
            let title = util::strings::replace_unsafe_symbols(&title);
            let title = util::strings::replace_round_brackets(&title);
            let title = util::strings::remove_characters(&regex_raw_strings, &title);
            let title = util::strings::pad_volume_number(&title);
            let title = util::strings::remove_spaces(&title);
            self.title = Some(title);
        }

        Ok(())
    }
}

pub fn get_book_metadata(opf_content: &str) -> Result<BookMetadata> {
    let mut reader = Reader::from_str(opf_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut in_metadata = false;
    let mut res = BookMetadata::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => Err(anyhow!(e))?,
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => match e.name().into_inner() {
                TAG_METADATA => {
                    in_metadata = true;
                }
                TAG_DC_TITLE => {
                    if !in_metadata {
                        continue;
                    }
                    let title = reader.read_text(e.name())?;
                    res.title = Some(title.into());
                }
                TAG_DC_CREATOR => {
                    if !in_metadata {
                        continue;
                    }
                    let author = reader.read_text(e.name())?;
                    res.author = Some(author.into());
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => {
                if e.local_name().into_inner() == TAG_METADATA {
                    in_metadata = false;
                }
            }
            _ => {
                if res.is_filled() {
                    break;
                }
            }
        }
    }

    Ok(res)
}

mod tests {
    // TODO add tests

    #[test]
    fn test_get_book_metadata() {
        let opf_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id" prefix="rendition: http://www.idpf.org/vocab/rendition/#">
            <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
                <dc:title>タイトル</dc:title>
                <dc:creator>作者</dc:creator>
            </metadata>
        </package>
        "#;

        let metadata = super::get_book_metadata(opf_content).unwrap();
        assert_eq!(metadata.title, Some("タイトル".to_string()));
        assert_eq!(metadata.author, Some("作者".to_string()));
    }
}
