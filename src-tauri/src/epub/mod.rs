//! EPUB parsing and Webpub Manifest generation module.

pub mod manifest;
pub mod parser;

pub use manifest::WebpubManifest;
pub use parser::EpubParser;
