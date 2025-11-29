//! Test fixtures for EPUB testing.
//!
//! Provides utilities to create test EPUB files.

use std::io::Write;
use tempfile::NamedTempFile;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Builder for creating test EPUB files.
pub struct TestEpubBuilder {
    title: String,
    identifier: String,
    language: String,
    creators: Vec<String>,
    publishers: Vec<String>,
    chapters: Vec<(String, String, String)>, // (id, filename, content)
    stylesheets: Vec<(String, String, String)>, // (id, filename, content)
}

impl Default for TestEpubBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEpubBuilder {
    pub fn new() -> Self {
        Self {
            title: "Test Book".to_string(),
            identifier: "urn:uuid:12345678-1234-1234-1234-123456789012".to_string(),
            language: "en".to_string(),
            creators: vec!["Test Author".to_string()],
            publishers: vec!["Test Publisher".to_string()],
            chapters: vec![
                (
                    "chapter1".to_string(),
                    "chapter1.xhtml".to_string(),
                    "<h1>Chapter 1</h1><p>Content</p>".to_string(),
                ),
                (
                    "chapter2".to_string(),
                    "chapter2.xhtml".to_string(),
                    "<h1>Chapter 2</h1><p>More content</p>".to_string(),
                ),
            ],
            stylesheets: vec![(
                "style".to_string(),
                "style.css".to_string(),
                "body { font-family: serif; }".to_string(),
            )],
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = identifier.into();
        self
    }

    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creators.push(creator.into());
        self
    }

    pub fn clear_creators(mut self) -> Self {
        self.creators.clear();
        self
    }

    pub fn chapter(
        mut self,
        id: impl Into<String>,
        filename: impl Into<String>,
        body_content: impl Into<String>,
    ) -> Self {
        self.chapters
            .push((id.into(), filename.into(), body_content.into()));
        self
    }

    pub fn clear_chapters(mut self) -> Self {
        self.chapters.clear();
        self
    }

    /// Build the test EPUB file.
    pub fn build(self) -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        let mut zip = ZipWriter::new(file.reopen().unwrap());
        let options = SimpleFileOptions::default();

        // mimetype (must be first)
        zip.start_file("mimetype", options).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();

        // container.xml
        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .unwrap();

        // content.opf
        zip.start_file("OEBPS/content.opf", options).unwrap();
        let opf = self.generate_opf();
        zip.write_all(opf.as_bytes()).unwrap();

        // Chapter files
        for (_, filename, body_content) in &self.chapters {
            zip.start_file(format!("OEBPS/{}", filename), options)
                .unwrap();
            let xhtml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter</title></head>
<body>{}</body>
</html>"#,
                body_content
            );
            zip.write_all(xhtml.as_bytes()).unwrap();
        }

        // Stylesheet files
        for (_, filename, content) in &self.stylesheets {
            zip.start_file(format!("OEBPS/{}", filename), options)
                .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }

        zip.finish().unwrap();
        file
    }

    fn generate_opf(&self) -> String {
        let mut manifest_items = String::new();
        let mut spine_items = String::new();

        for (id, filename, _) in &self.chapters {
            manifest_items.push_str(&format!(
                r#"    <item id="{}" href="{}" media-type="application/xhtml+xml"/>"#,
                id, filename
            ));
            manifest_items.push('\n');
            spine_items.push_str(&format!(r#"    <itemref idref="{}"/>"#, id));
            spine_items.push('\n');
        }

        for (id, filename, _) in &self.stylesheets {
            manifest_items.push_str(&format!(
                r#"    <item id="{}" href="{}" media-type="text/css"/>"#,
                id, filename
            ));
            manifest_items.push('\n');
        }

        let creators: String = self
            .creators
            .iter()
            .map(|c| format!("    <dc:creator>{}</dc:creator>", c))
            .collect::<Vec<_>>()
            .join("\n");

        let publishers: String = self
            .publishers
            .iter()
            .map(|p| format!("    <dc:publisher>{}</dc:publisher>", p))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">{}</dc:identifier>
    <dc:title>{}</dc:title>
    <dc:language>{}</dc:language>
{}
{}
    <meta property="dcterms:modified">2024-01-01T00:00:00Z</meta>
  </metadata>
  <manifest>
{}  </manifest>
  <spine>
{}  </spine>
</package>"#,
            self.identifier,
            self.title,
            self.language,
            creators,
            publishers,
            manifest_items,
            spine_items
        )
    }
}

/// Create a minimal valid EPUB file with default settings.
pub fn create_test_epub() -> NamedTempFile {
    TestEpubBuilder::new().build()
}

/// Create an invalid EPUB file (missing container.xml).
pub fn create_invalid_epub_missing_container() -> NamedTempFile {
    let file = NamedTempFile::new().unwrap();
    let mut zip = ZipWriter::new(file.reopen().unwrap());
    let options = SimpleFileOptions::default();

    zip.start_file("mimetype", options).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();
    zip.finish().unwrap();

    file
}
