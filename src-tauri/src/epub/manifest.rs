//! Readium Webpub Manifest structures.
//!
//! Based on: https://readium.org/webpub-manifest/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Readium Webpub Manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebpubManifest {
    #[serde(rename = "@context")]
    pub context: String,

    pub metadata: Metadata,

    pub links: Vec<Link>,

    pub reading_order: Vec<Link>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<Link>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc: Option<Vec<TocEntry>>,
}

impl WebpubManifest {
    pub fn new(metadata: Metadata) -> Self {
        Self {
            context: "https://readium.org/webpub-manifest/context.jsonld".to_string(),
            metadata,
            links: Vec::new(),
            reading_order: Vec::new(),
            resources: None,
            toc: None,
        }
    }
}

/// Publication metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: LocalizedString,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Vec<Contributor>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Vec<Contributor>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Vec<Subject>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Metadata {
    pub fn new(title: impl Into<LocalizedString>) -> Self {
        Self {
            title: title.into(),
            author: None,
            language: None,
            identifier: None,
            modified: None,
            published: None,
            publisher: None,
            subject: None,
            description: None,
        }
    }
}

/// A localized string that can be either a simple string or a map of language codes to strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LocalizedString {
    Simple(String),
    Localized(HashMap<String, String>),
}

impl From<String> for LocalizedString {
    fn from(s: String) -> Self {
        LocalizedString::Simple(s)
    }
}

impl From<&str> for LocalizedString {
    fn from(s: &str) -> Self {
        LocalizedString::Simple(s.to_string())
    }
}

/// A contributor (author, publisher, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Contributor {
    Simple(String),
    Detailed(ContributorDetails),
}

impl From<String> for Contributor {
    fn from(s: String) -> Self {
        Contributor::Simple(s)
    }
}

impl From<&str> for Contributor {
    fn from(s: &str) -> Self {
        Contributor::Simple(s.to_string())
    }
}

/// Detailed contributor information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributorDetails {
    pub name: LocalizedString,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_as: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Vec<String>>,
}

/// A subject/category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Subject {
    Simple(String),
    Detailed(SubjectDetails),
}

/// Detailed subject information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectDetails {
    pub name: LocalizedString,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_as: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
}

/// A link to a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub href: String,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<LinkProperties>,
}

impl Link {
    pub fn new(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            media_type: None,
            title: None,
            rel: None,
            properties: None,
        }
    }

    pub fn with_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }

    pub fn with_rel(mut self, rel: impl Into<String>) -> Self {
        self.rel = Some(vec![rel.into()]);
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Link properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<EncryptedProperties>,
}

/// Encryption properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedProperties {
    pub algorithm: String,
}

/// A table of contents entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TocEntry {
    pub href: String,

    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TocEntry>>,
}

impl TocEntry {
    pub fn new(href: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            title: title.into(),
            children: None,
        }
    }

    pub fn with_children(mut self, children: Vec<TocEntry>) -> Self {
        if !children.is_empty() {
            self.children = Some(children);
        }
        self
    }
}
