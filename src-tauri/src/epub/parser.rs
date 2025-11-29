//! EPUB parsing module.
//!
//! Handles reading EPUB files (ZIP archives) and extracting OPF metadata.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use thiserror::Error;
use zip::ZipArchive;

use super::manifest::{
    Contributor, Link, Metadata, TocEntry, WebpubManifest,
};

/// Errors that can occur during EPUB parsing.
#[derive(Debug, Error)]
pub enum EpubError {
    #[error("Failed to open EPUB file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to read ZIP archive: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("Failed to parse XML: {0}")]
    XmlError(#[from] roxmltree::Error),

    #[error("Invalid EPUB: {0}")]
    InvalidEpub(String),

    #[error("Missing required element: {0}")]
    MissingElement(String),
}

pub type Result<T> = std::result::Result<T, EpubError>;

/// EPUB parser that converts EPUB files to Webpub Manifest.
pub struct EpubParser {
    archive: ZipArchive<BufReader<File>>,
    /// Base directory of the OPF file within the EPUB.
    opf_base_dir: String,
}

impl EpubParser {
    /// Open an EPUB file for parsing.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let archive = ZipArchive::new(reader)?;

        Ok(Self {
            archive,
            opf_base_dir: String::new(),
        })
    }

    /// Parse the EPUB and generate a Webpub Manifest.
    pub fn parse(&mut self) -> Result<WebpubManifest> {
        // 1. Parse container.xml to find OPF path
        let opf_path = self.parse_container()?;

        // Store base directory for resolving relative paths
        self.opf_base_dir = opf_path
            .rfind('/')
            .map(|i| opf_path[..=i].to_string())
            .unwrap_or_default();

        // 2. Parse OPF file
        let opf = self.parse_opf(&opf_path)?;

        // 3. Convert to Webpub Manifest
        let manifest = self.build_manifest(opf)?;

        Ok(manifest)
    }

    /// Read a file from the EPUB archive.
    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        let mut file = self.archive.by_name(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    /// Parse META-INF/container.xml to find the OPF file path.
    fn parse_container(&mut self) -> Result<String> {
        let xml = self.read_file("META-INF/container.xml")?;
        let xml_str = String::from_utf8_lossy(&xml);
        let doc = roxmltree::Document::parse(&xml_str)?;

        // Find rootfile element
        let rootfile = doc
            .descendants()
            .find(|n| n.has_tag_name("rootfile"))
            .ok_or_else(|| EpubError::MissingElement("rootfile".to_string()))?;

        rootfile
            .attribute("full-path")
            .map(|s| s.to_string())
            .ok_or_else(|| EpubError::MissingElement("rootfile full-path".to_string()))
    }

    /// Parse the OPF (Package Document) file.
    fn parse_opf(&mut self, path: &str) -> Result<OpfPackage> {
        let xml = self.read_file(path)?;
        let xml_str = String::from_utf8_lossy(&xml);
        let doc = roxmltree::Document::parse(&xml_str)?;

        let package = doc
            .root_element();

        // Parse metadata
        let metadata = self.parse_opf_metadata(&package)?;

        // Parse manifest (list of all files)
        let manifest_items = self.parse_opf_manifest(&package)?;

        // Parse spine (reading order)
        let spine = self.parse_opf_spine(&package)?;

        Ok(OpfPackage {
            metadata,
            manifest: manifest_items,
            spine,
        })
    }

    /// Parse OPF metadata section.
    fn parse_opf_metadata(&self, package: &roxmltree::Node) -> Result<OpfMetadata> {
        let metadata_node = package
            .children()
            .find(|n| n.has_tag_name("metadata"))
            .ok_or_else(|| EpubError::MissingElement("metadata".to_string()))?;

        let mut metadata = OpfMetadata::default();

        for node in metadata_node.children().filter(|n| n.is_element()) {
            let tag = node.tag_name().name();
            let text = node.text().map(|s| s.to_string());

            match tag {
                "title" => {
                    if let Some(t) = text {
                        metadata.title = Some(t);
                    }
                }
                "creator" => {
                    if let Some(c) = text {
                        metadata.creators.push(c);
                    }
                }
                "language" => {
                    if let Some(l) = text {
                        metadata.languages.push(l);
                    }
                }
                "identifier" => {
                    if let Some(id) = text {
                        metadata.identifier = Some(id);
                    }
                }
                "publisher" => {
                    if let Some(p) = text {
                        metadata.publishers.push(p);
                    }
                }
                "description" => {
                    metadata.description = text;
                }
                "date" => {
                    metadata.date = text;
                }
                "meta" => {
                    // Handle EPUB 3 meta elements
                    if let Some(property) = node.attribute("property") {
                        if property == "dcterms:modified" {
                            metadata.modified = text;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(metadata)
    }

    /// Parse OPF manifest section (list of all resources).
    fn parse_opf_manifest(&self, package: &roxmltree::Node) -> Result<HashMap<String, ManifestItem>> {
        let manifest_node = package
            .children()
            .find(|n| n.has_tag_name("manifest"))
            .ok_or_else(|| EpubError::MissingElement("manifest".to_string()))?;

        let mut items = HashMap::new();

        for node in manifest_node.children().filter(|n| n.has_tag_name("item")) {
            if let (Some(id), Some(href)) = (node.attribute("id"), node.attribute("href")) {
                let item = ManifestItem {
                    id: id.to_string(),
                    href: href.to_string(),
                    media_type: node.attribute("media-type").map(|s| s.to_string()),
                    properties: node.attribute("properties").map(|s| s.to_string()),
                };
                items.insert(id.to_string(), item);
            }
        }

        Ok(items)
    }

    /// Parse OPF spine section (reading order).
    fn parse_opf_spine(&self, package: &roxmltree::Node) -> Result<Vec<SpineItem>> {
        let spine_node = package
            .children()
            .find(|n| n.has_tag_name("spine"))
            .ok_or_else(|| EpubError::MissingElement("spine".to_string()))?;

        let mut items = Vec::new();

        for node in spine_node.children().filter(|n| n.has_tag_name("itemref")) {
            if let Some(idref) = node.attribute("idref") {
                let item = SpineItem {
                    idref: idref.to_string(),
                    linear: node.attribute("linear").map(|s| s != "no").unwrap_or(true),
                };
                items.push(item);
            }
        }

        Ok(items)
    }

    /// Build Webpub Manifest from parsed OPF data.
    fn build_manifest(&self, opf: OpfPackage) -> Result<WebpubManifest> {
        // Build metadata
        let title = opf.metadata.title.unwrap_or_else(|| "Untitled".to_string());
        let mut metadata = Metadata::new(title);

        if !opf.metadata.creators.is_empty() {
            metadata.author = Some(
                opf.metadata.creators.into_iter().map(Contributor::from).collect()
            );
        }

        if !opf.metadata.languages.is_empty() {
            metadata.language = Some(opf.metadata.languages);
        }

        metadata.identifier = opf.metadata.identifier;
        metadata.modified = opf.metadata.modified;
        metadata.published = opf.metadata.date;
        metadata.description = opf.metadata.description;

        if !opf.metadata.publishers.is_empty() {
            metadata.publisher = Some(
                opf.metadata.publishers.into_iter().map(Contributor::from).collect()
            );
        }

        let mut manifest = WebpubManifest::new(metadata);

        // Build reading order from spine
        for spine_item in &opf.spine {
            if let Some(item) = opf.manifest.get(&spine_item.idref) {
                let href = self.resolve_href(&item.href);
                let mut link = Link::new(href);
                link.media_type = item.media_type.clone();
                manifest.reading_order.push(link);
            }
        }

        // Build resources (items not in spine)
        let spine_idrefs: std::collections::HashSet<_> =
            opf.spine.iter().map(|s| &s.idref).collect();

        let resources: Vec<Link> = opf
            .manifest
            .values()
            .filter(|item| !spine_idrefs.contains(&item.id))
            .map(|item| {
                let href = self.resolve_href(&item.href);
                let mut link = Link::new(href);
                link.media_type = item.media_type.clone();
                link
            })
            .collect();

        if !resources.is_empty() {
            manifest.resources = Some(resources);
        }

        // TODO: Parse navigation document for TOC

        Ok(manifest)
    }

    /// Resolve a relative href to be relative to the EPUB root.
    fn resolve_href(&self, href: &str) -> String {
        if href.starts_with('/') || href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else {
            format!("{}{}", self.opf_base_dir, href)
        }
    }
}

// ============================================================================
// Internal OPF structures
// ============================================================================

/// Parsed OPF package document.
#[derive(Debug)]
struct OpfPackage {
    metadata: OpfMetadata,
    manifest: HashMap<String, ManifestItem>,
    spine: Vec<SpineItem>,
}

/// Parsed OPF metadata.
#[derive(Debug, Default)]
struct OpfMetadata {
    title: Option<String>,
    creators: Vec<String>,
    languages: Vec<String>,
    identifier: Option<String>,
    publishers: Vec<String>,
    description: Option<String>,
    date: Option<String>,
    modified: Option<String>,
}

/// An item in the OPF manifest.
#[derive(Debug)]
struct ManifestItem {
    id: String,
    href: String,
    media_type: Option<String>,
    #[allow(dead_code)]
    properties: Option<String>,
}

/// An item in the OPF spine.
#[derive(Debug)]
struct SpineItem {
    idref: String,
    #[allow(dead_code)]
    linear: bool,
}
