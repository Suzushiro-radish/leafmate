//! Tauri commands for EPUB handling.

use crate::epub::{EpubParser, WebpubManifest};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// State to hold currently opened EPUB files.
pub struct EpubState {
    /// Map of publication ID to parsed EPUB data.
    pub publications: Mutex<HashMap<String, OpenedPublication>>,
}

impl EpubState {
    pub fn new() -> Self {
        Self {
            publications: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for EpubState {
    fn default() -> Self {
        Self::new()
    }
}

/// Data for an opened EPUB publication.
pub struct OpenedPublication {
    /// Path to the EPUB file.
    pub path: PathBuf,
    /// Generated Webpub Manifest.
    pub manifest: WebpubManifest,
}

/// Result of opening an EPUB file.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenEpubResult {
    /// Unique identifier for this publication.
    pub id: String,
}

/// Open an EPUB file and return its manifest URL.
#[tauri::command]
pub fn open_epub(path: String, state: State<std::sync::Arc<EpubState>>) -> Result<OpenEpubResult, String> {
    let path_buf = PathBuf::from(&path);

    // Validate file exists
    if !path_buf.exists() {
        return Err(format!("File not found: {}", path));
    }

    // Parse EPUB
    let mut parser = EpubParser::open(&path_buf).map_err(|e| e.to_string())?;
    let mut manifest = parser.parse().map_err(|e| e.to_string())?;

    // Generate unique ID for this publication
    let id = generate_publication_id(&path_buf);

    // Add self link to manifest
    add_self_link(&mut manifest, &id);

    // Store in state
    {
        let mut publications = state.publications.lock().unwrap();
        publications.insert(
            id.clone(),
            OpenedPublication {
                path: path_buf,
                manifest,
            },
        );
    }

    Ok(OpenEpubResult { id })
}

/// Get the manifest JSON for a publication.
#[tauri::command]
pub fn get_manifest(id: String, state: State<std::sync::Arc<EpubState>>) -> Result<String, String> {
    let publications = state.publications.lock().unwrap();

    let publication = publications
        .get(&id)
        .ok_or_else(|| format!("Publication not found: {}", id))?;

    serde_json::to_string(&publication.manifest).map_err(|e| e.to_string())
}

/// Get a resource from an opened EPUB.
#[tauri::command]
pub fn get_epub_resource(id: String, path: String, state: State<std::sync::Arc<EpubState>>) -> Result<Vec<u8>, String> {
    let publications = state.publications.lock().unwrap();

    let publication = publications
        .get(&id)
        .ok_or_else(|| format!("Publication not found: {}", id))?;

    // Re-open the EPUB to read the resource
    let mut parser = EpubParser::open(&publication.path).map_err(|e| e.to_string())?;

    let mut data = parser.read_file(&path).map_err(|e| e.to_string())?;

    // For HTML/XHTML files, inject a base tag to resolve relative paths
    if path.ends_with(".html") || path.ends_with(".xhtml") {
        data = inject_base_tag(data, &id, &path).map_err(|e| e.to_string())?;
    }

    Ok(data)
}

/// Close an opened EPUB publication.
#[tauri::command]
pub fn close_epub(id: String, state: State<std::sync::Arc<EpubState>>) -> Result<(), String> {
    let mut publications = state.publications.lock().unwrap();
    publications.remove(&id);
    Ok(())
}

/// Add self link to manifest for proper locator generation.
fn add_self_link(manifest: &mut WebpubManifest, id: &str) {
    use crate::epub::manifest::Link;

    // Remove any existing self links
    manifest.links.retain(|link| {
        link.rel.as_ref().map_or(true, |rels| !rels.contains(&"self".to_string()))
    });

    // Add new self link
    manifest.links.push(Link {
        href: format!("tauri://epub/{}/manifest.json", id),
        media_type: Some("application/webpub+json".to_string()),
        title: None,
        rel: Some(vec!["self".to_string()]),
        properties: None,
    });
}

/// Inject a <base> tag into HTML/XHTML to enable relative path resolution.
fn inject_base_tag(data: Vec<u8>, id: &str, path: &str) -> Result<Vec<u8>, String> {
    let html = String::from_utf8(data).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    
    // Calculate base URL: tauri://epub/{id}/path/to/directory/
    let dir_path = path.rfind('/').map(|i| &path[..=i]).unwrap_or("");
    let base_url = format!("tauri://epub/{}/{}", id, dir_path);
    
    // Inject base tag after <head>
    let base_tag = format!("<base href=\"{}\"/>\n", base_url);
    
    let modified_html = if let Some(head_pos) = html.find("<head>") {
        let insert_pos = head_pos + "<head>".len();
        format!(
            "{}{}{}",
            &html[..insert_pos],
            base_tag,
            &html[insert_pos..]
        )
    } else {
        // No <head> tag, return as-is
        html
    };
    
    Ok(modified_html.into_bytes())
}

/// Generate a unique ID for a publication based on its path.
fn generate_publication_id(path: &PathBuf) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);

    format!("{:x}", hasher.finish())
}
