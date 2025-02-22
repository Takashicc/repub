use anyhow::{anyhow, Context};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::{params::unpack::UnpackParams, util::files};
use crate::{services, util};
use anyhow::Result;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use std::fs::{self, File};
use std::io::{self, Cursor, Read, Seek};
use zip::read::ZipArchive;

pub fn execute(params: &UnpackParams) -> Result<()> {
    files::list_epub_filepaths(Path::new(&params.input))
        .par_iter()
        .for_each(|filepath| {
            if let Err(e) = process_epub(filepath) {
                eprintln!("filename: {}, error: {:?}", filepath.display(), e);
            }
        });

    Ok(())
}

fn process_epub(epub_path: &Path) -> Result<()> {
    let epub_bytes = Arc::new(fs::read(epub_path)?);
    let mut archive = ZipArchive::new(Cursor::new(&**epub_bytes))?;

    let container_xml = services::epub::read_container_xml(&mut archive)?;
    let opf_path = services::epub::get_rootfile_path(&container_xml)?;
    let opf_content = services::epub::read_file_from_archive(&mut archive, &opf_path)?;

    // if !services::epub::is_comic(&opf_content)? {
    //     println!(
    //         "このEPUBは漫画ではないためスキップします。: {}",
    //         epub_path.display()
    //     );
    //     return Ok(());
    // }

    let opf_data = services::epub::parse_opf(&opf_content)?;

    let epub_dir = epub_path.parent().unwrap_or_else(|| Path::new("."));
    let epub_stem = epub_path.file_stem().unwrap().to_string_lossy();
    let output_dir = epub_dir.join(epub_stem.to_string());
    fs::create_dir_all(&output_dir)?;

    // spine の順に、manifest のアイテムの中から画像を抽出
    opf_data
        .spine_item_refs
        .par_iter()
        .enumerate()
        .try_for_each(|(i, spine_item_ref)| -> Result<()> {
            let counter = i + 1;
            let item = opf_data
                .manifest_items
                .get(&spine_item_ref.idref)
                .ok_or_else(|| {
                    anyhow!(
                        "manifestに存在しないidrefがspineに存在します: {}",
                        spine_item_ref.idref
                    )
                })?;
            let mut archive = ZipArchive::new(Cursor::new(&**epub_bytes))?;
            if let Some(fallback) = &item.fallback {
                extract_image_from_fallback(
                    &opf_data.manifest_items,
                    fallback,
                    &opf_path,
                    &mut archive,
                    counter,
                    &output_dir,
                )
                .with_context(|| format!("Failed to extract image from fallback: {}", fallback))?;
            } else {
                let image_suffixes = vec![".jpg", ".jpeg", ".png"];
                if image_suffixes
                    .iter()
                    .any(|suffix| item.href.ends_with(suffix))
                {
                    let image_path = Path::new(&opf_path)
                        .parent()
                        .unwrap_or(Path::new(""))
                        .join(&item.href);
                    extract_file(&mut archive, &image_path, counter, &output_dir)
                        .with_context(|| format!("Failed to extract image: {}", item.href))?;
                } else {
                    extract_image_from_xhtml(
                        &opf_path,
                        &item.href,
                        &mut archive,
                        counter,
                        &output_dir,
                    )
                    .with_context(|| {
                        format!("Failed to extract image from xhtml: {}", &item.href)
                    })?;
                }
            }

            Ok(())
        })?;

    println!("  抽出完了: {}", epub_path.display());
    Ok(())
}

fn extract_image_from_fallback<T>(
    manifest_items: &HashMap<String, services::epub::ManifestItem>,
    fallback: &str,
    opf_path: &str,
    archive: &mut ZipArchive<T>,
    counter: usize,
    output_dir: &Path,
) -> Result<()>
where
    T: Read + Seek,
{
    let fallback_item = manifest_items.get(fallback);
    if let Some(fallback_item) = fallback_item {
        if !fallback_item.media_type.starts_with("image/") {
            return Err(anyhow!("fallback先が画像ではありません: {}", fallback));
        }

        let image_path = Path::new(&opf_path)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(&fallback_item.href);
        extract_file(archive, &image_path, counter, output_dir)?;
    }

    Ok(())
}

fn extract_image_from_xhtml<T>(
    opf_path: &str,
    href: &str,
    archive: &mut ZipArchive<T>,
    counter: usize,
    output_dir: &Path,
) -> Result<()>
where
    T: Read + Seek,
{
    // Get image href from xhtml
    let xhtml_path = Path::new(&opf_path)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(href);
    let xhtml_content =
        services::epub::read_file_from_archive(archive, &util::paths::normalize_path(&xhtml_path))?;
    let image_href = services::xhtml::extract_image_href_from_xhtml(&xhtml_content)?;

    // Extract image
    let image_path = xhtml_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&image_href);
    extract_file(archive, &image_path, counter, output_dir)?;

    Ok(())
}

fn extract_file<T>(
    archive: &mut ZipArchive<T>,
    path: &Path,
    counter: usize,
    output_dir: &Path,
) -> Result<()>
where
    T: Read + Seek,
{
    let normalized_path = util::paths::normalize_path(path);
    let mut file = archive
        .by_name(&normalized_path)
        .with_context(|| format!("Failed to read file from archive: {}", &normalized_path))?;

    let ext = path
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .with_context(|| format!("Failed to get extension from path: {}", path.display()))?;
    let filename = format!("{:04}.{}", counter, ext);
    let output_path = output_dir.join(filename);
    let mut outfile = File::create(&output_path)
        .with_context(|| format!("Failed to create file: {}", output_path.display()))?;
    io::copy(&mut file, &mut outfile)
        .with_context(|| format!("Failed to copy file: {}", output_path.display()))?;

    Ok(())
}
