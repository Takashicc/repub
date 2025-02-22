use anyhow::{anyhow, Result};
use quick_xml::{events::Event, Reader};

// XHTML内の<image>タグからxlink:href属性を抽出する
pub fn extract_image_href_from_xhtml(xhtml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xhtml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(anyhow!(e)),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => match e.name().into_inner() {
                b"image" => {
                    for attr in e.attributes() {
                        let attr = attr?;
                        if attr.key.into_inner() == b"xlink:href" {
                            return Ok(String::from_utf8(attr.value.to_vec())?);
                        }
                    }
                }
                b"img" => {
                    for attr in e.attributes() {
                        let attr = attr?;
                        if attr.key.into_inner() == b"src" {
                            return Ok(String::from_utf8(attr.value.to_vec())?);
                        }
                    }
                }
                _ => continue,
            },
            _ => {}
        }
    }

    Err(anyhow!("xhtmlから画像の参照が見つかりませんでした"))
}

mod tests {
    #[test]
    fn test_extract_image_href_from_xhtml_image_tag() {
        let xhtml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE html>
            <html
            xmlns="http://www.w3.org/1999/xhtml"
            xmlns:epub="http://www.idpf.org/2007/ops"
            xml:lang="ja"
            >
                <head>
                    <title>Test</title>
                </head>
                <body>
                    <image width="1440" height="2048" xlink:href="../image/cover.jpg"/>
                </body>
            </html>
        "#;

        let href = super::extract_image_href_from_xhtml(xhtml).unwrap();
        assert_eq!(href, "../image/cover.jpg");
    }

    #[test]
    fn test_extract_image_href_from_xhtml_img_tag() {
        let xhtml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE html>
            <html
            xmlns="http://www.w3.org/1999/xhtml"
            xmlns:epub="http://www.idpf.org/2007/ops"
            xml:lang="ja"
            >
                <head>
                    <title>Test</title>
                </head>
                <body>
                    <img width="1440" height="2048" xlink:href="../image/cover.jpg"/>
                </body>
            </html>
        "#;

        let href = super::extract_image_href_from_xhtml(xhtml).unwrap();
        assert_eq!(href, "../image/cover.jpg");
    }
}
