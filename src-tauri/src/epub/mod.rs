//! EPUB parsing and Webpub Manifest generation module.

pub mod manifest;
pub mod parser;

#[cfg(test)]
mod tests;

pub use manifest::WebpubManifest;
pub use parser::EpubParser;
