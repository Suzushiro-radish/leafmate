//! Tests for the EPUB parsing module.

#[cfg(test)]
mod tests {
    use crate::epub::manifest::{Link, LocalizedString, Metadata, TocEntry, WebpubManifest};
    use crate::epub::parser::EpubParser;
    use crate::epub::test_fixtures::{create_invalid_epub_missing_container, create_test_epub};

    // =========================================================================
    // Parser tests
    // =========================================================================

    #[test]
    fn test_parse_epub() {
        let epub_file = create_test_epub();
        let mut parser = EpubParser::open(epub_file.path()).unwrap();
        let manifest = parser.parse().unwrap();

        // Check metadata
        assert!(matches!(
            &manifest.metadata.title,
            LocalizedString::Simple(s) if s == "Test Book"
        ));
        assert_eq!(
            manifest.metadata.identifier,
            Some("urn:uuid:12345678-1234-1234-1234-123456789012".to_string())
        );
        assert_eq!(manifest.metadata.language, Some(vec!["en".to_string()]));
        assert_eq!(
            manifest.metadata.modified,
            Some("2024-01-01T00:00:00Z".to_string())
        );

        // Check authors
        assert!(manifest.metadata.author.is_some());

        // Check publishers
        assert!(manifest.metadata.publisher.is_some());

        // Check reading order
        assert_eq!(manifest.reading_order.len(), 2);
        assert_eq!(manifest.reading_order[0].href, "OEBPS/chapter1.xhtml");
        assert_eq!(manifest.reading_order[1].href, "OEBPS/chapter2.xhtml");
        assert_eq!(
            manifest.reading_order[0].media_type,
            Some("application/xhtml+xml".to_string())
        );

        // Check resources (CSS should be here, not in reading order)
        let resources = manifest.resources.as_ref().unwrap();
        assert!(resources.iter().any(|r| r.href == "OEBPS/style.css"));
    }

    #[test]
    fn test_read_file_from_epub() {
        let epub_file = create_test_epub();
        let mut parser = EpubParser::open(epub_file.path()).unwrap();

        let content = parser.read_file("OEBPS/style.css").unwrap();
        assert_eq!(content, b"body { font-family: serif; }");
    }

    #[test]
    fn test_invalid_epub_missing_container() {
        let file = create_invalid_epub_missing_container();
        let mut parser = EpubParser::open(file.path()).unwrap();
        assert!(parser.parse().is_err());
    }

    // =========================================================================
    // Manifest serialization tests
    // =========================================================================

    #[test]
    fn test_manifest_serialization() {
        let metadata = Metadata::new("Test Title");
        let manifest = WebpubManifest::new(metadata);

        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("\"@context\""));
        assert!(json.contains("Test Title"));
        assert!(json.contains("readingOrder"));
    }

    #[test]
    fn test_link_builder() {
        let link = Link::new("chapter1.xhtml")
            .with_type("application/xhtml+xml")
            .with_title("Chapter 1");

        assert_eq!(link.href, "chapter1.xhtml");
        assert_eq!(link.media_type, Some("application/xhtml+xml".to_string()));
        assert_eq!(link.title, Some("Chapter 1".to_string()));
    }

    #[test]
    fn test_toc_entry_with_children() {
        let child = TocEntry::new("section1.xhtml", "Section 1");
        let parent = TocEntry::new("chapter1.xhtml", "Chapter 1").with_children(vec![child]);

        assert!(parent.children.is_some());
        assert_eq!(parent.children.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_localized_string_simple() {
        let s: LocalizedString = "Hello".into();
        assert!(matches!(s, LocalizedString::Simple(ref v) if v == "Hello"));
    }

    #[test]
    fn test_metadata_json_skips_none() {
        let metadata = Metadata::new("Title Only");
        let json = serde_json::to_string(&metadata).unwrap();

        // Should not contain optional fields that are None
        assert!(!json.contains("\"author\""));
        assert!(!json.contains("\"language\""));
        assert!(!json.contains("\"identifier\""));
    }
}
