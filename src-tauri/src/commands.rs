//! Tauri commands for EPUB handling.

use crate::epub::{EpubParser, WebpubManifest};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// State to hold currently opened EPUB files.
pub struct EpubState {
    /// Map of publication ID to parsed EPUB data.
    publications: Mutex<HashMap<String, OpenedPublication>>,
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
pub fn open_epub(path: String, state: State<EpubState>) -> Result<OpenEpubResult, String> {
    let path_buf = PathBuf::from(&path);

    // Validate file exists
    if !path_buf.exists() {
        return Err(format!("File not found: {}", path));
    }

    // Parse EPUB
    let mut parser = EpubParser::open(&path_buf).map_err(|e| e.to_string())?;
    let manifest = parser.parse().map_err(|e| e.to_string())?;

    // Generate unique ID for this publication
    let id = generate_publication_id(&path_buf);

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
pub fn get_manifest(id: String, state: State<EpubState>) -> Result<String, String> {
    let publications = state.publications.lock().unwrap();

    let publication = publications
        .get(&id)
        .ok_or_else(|| format!("Publication not found: {}", id))?;

    serde_json::to_string(&publication.manifest).map_err(|e| e.to_string())
}

/// Get a resource from an opened EPUB.
#[tauri::command]
pub fn get_epub_resource(id: String, path: String, state: State<EpubState>) -> Result<Vec<u8>, String> {
    let publications = state.publications.lock().unwrap();

    let publication = publications
        .get(&id)
        .ok_or_else(|| format!("Publication not found: {}", id))?;

    // Re-open the EPUB to read the resource
    let mut parser = EpubParser::open(&publication.path).map_err(|e| e.to_string())?;

    parser.read_file(&path).map_err(|e| e.to_string())
}

/// Close an opened EPUB publication.
#[tauri::command]
pub fn close_epub(id: String, state: State<EpubState>) -> Result<(), String> {
    let mut publications = state.publications.lock().unwrap();
    publications.remove(&id);
    Ok(())
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
