pub mod commands;
pub mod epub;

use commands::{close_epub, get_epub_resource, get_manifest, open_epub, EpubState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(EpubState::new())
        .invoke_handler(tauri::generate_handler![
            open_epub,
            get_manifest,
            get_epub_resource,
            close_epub,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
