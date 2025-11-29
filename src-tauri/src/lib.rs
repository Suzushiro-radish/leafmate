pub mod commands;
pub mod epub;

use commands::{close_epub, get_epub_resource, get_manifest, open_epub, EpubState};
use epub::EpubParser;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(std::sync::Arc::new(EpubState::new()))
        .register_asynchronous_uri_scheme_protocol("tauri", move |ctx, request, responder| {
            let state = ctx.app_handle().state::<std::sync::Arc<EpubState>>();
            let state_clone = state.inner().clone();
            
            std::thread::spawn(move || {
                handle_epub_protocol(request, responder, state_clone);
            });
        })
        .invoke_handler(tauri::generate_handler![
            open_epub,
            get_manifest,
            get_epub_resource,
            close_epub,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Handle custom protocol requests for EPUB resources.
fn handle_epub_protocol(
    request: tauri::http::Request<Vec<u8>>,
    responder: tauri::UriSchemeResponder,
    state: std::sync::Arc<EpubState>,
) {
    let uri = request.uri();
    let path = uri.path();
    
    // Expected format: /{id}/{resource_path}
    // Base tag creates URLs like tauri://epub/{id}/OEBPS/file.xhtml
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    
    if parts.len() < 2 {
        responder.respond(
            tauri::http::Response::builder()
                .status(404)
                .body(Vec::new())
                .unwrap(),
        );
        return;
    }
    
    let id = parts[0];
    let resource_path = parts[1..].join("/");
    
    // Get publication from state
    let publications = state.publications.lock().unwrap();
    let publication = match publications.get(id) {
        Some(publication) => publication,
        None => {
            responder.respond(
                tauri::http::Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .unwrap(),
            );
            return;
        }
    };
    
    // Read resource from EPUB
    let mut parser = match EpubParser::open(&publication.path) {
        Ok(p) => p,
        Err(e) => {
            responder.respond(
                tauri::http::Response::builder()
                    .status(500)
                    .body(Vec::new())
                    .unwrap(),
            );
            return;
        }
    };
    
    match parser.read_file(&resource_path) {
        Ok(data) => {
            let content_type = determine_content_type(&resource_path);
            
            responder.respond(
                tauri::http::Response::builder()
                    .status(200)
                    .header("Content-Type", content_type)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(data)
                    .unwrap(),
            );
        }
        Err(_) => {
            responder.respond(
                tauri::http::Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .unwrap(),
            );
        }
    }
}

/// Determine MIME type from file extension.
fn determine_content_type(path: &str) -> &'static str {
    if path.ends_with(".xhtml") || path.ends_with(".html") {
        "application/xhtml+xml"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else {
        "application/octet-stream"
    }
}
