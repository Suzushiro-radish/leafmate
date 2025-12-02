# Leafmate Architecture Document

Leafmate is a desktop EPUB reader that combines Tauri and React. It leverages Readium Navigator to bridge native file access and web-based rendering.

## System Overview

```
┌─────────────────────────────────────┐
│         Tauri Desktop Shell         │
├─────────────────────────────────────┤
│         Rust Backend                │
│  • EPUB parser (ZIP + XML)          │
│  • Webpub Manifest generator        │
│  • Custom protocol handler          │
│    (tauri://epub/{id}/)             │
├─────────────────────────────────────┤
│         React Frontend              │
│  • Readium Navigator                │
│  • Keyboard interactions            │
│  • UI                               │
└─────────────────────────────────────┘
```

## Core Concepts

### 1. EPUB → Webpub Manifest Conversion
The Rust backend parses an EPUB archive (ZIP) and converts it into the Readium Webpub Manifest format so that Readium Navigator can consume a standardized data structure.

### 2. Custom Protocol (`tauri://epub/`)
Resources inside the EPUB (HTML, CSS, images, etc.) are exposed via the custom URL scheme `tauri://epub/{id}/{path}`, which resolves relative paths automatically within iframes.

### 3. TauriFetcher
We implement Readium’s `Fetcher` interface and retrieve resources from Rust through Tauri commands, accessing the native filesystem directly instead of using the browser’s fetch API.

## Main Processing Flows

### Opening an EPUB

1. **User selects a file** → Tauri file dialog
2. **Rust: `open_epub(path)`**
   - Open the EPUB archive (ZIP)
   - Read `container.xml` to locate the OPF file
   - Parse the OPF to extract metadata, the spine (reading order), and the manifest (all resources)
   - Convert the result to a Webpub Manifest and store it in `EpubState`
3. **TypeScript: `openEpub(path)`**
   - Fetch the manifest and build a Readium `Publication`
   - Attach the `TauriFetcher` for resource access
   - Store the `Publication` in the Viewer state
4. **`useEpubNavigator` detects the `Publication`**
   - Instantiate `EpubNavigator` and start rendering

### Fetching and Rendering Resources

1. **EpubNavigator requests a resource**
   - `TauriFetcher.get(link)` → `TauriResource.read()`
   - Calls `invoke("get_epub_resource", { id, path })`
2. **Rust: `get_epub_resource(id, path)`**
   - Reads the file from the ZIP archive
   - For HTML/XHTML, injects `<base href="tauri://epub/{id}/{dir}/">` into `<head>`
   - Returns the binary payload
3. **Rendering inside the iframe**
   - The HTML is rendered by the navigator
   - Relative paths (CSS, images, etc.) resolve automatically via the `<base>` tag to `tauri://epub/{id}/...`
   - The custom protocol handler serves each resource from the ZIP

### Navigation (Page Turns)

- **Arrow keys or navigation buttons** → `navRef.current.goRight()` / `.goLeft()`
- **EpubNavigator** computes and moves to the next page/section
- Additional resources are fetched as needed
- **Peripherals** registers keyboard handlers on the main window and the embedded iframes

## Key Files

### Frontend (TypeScript / React)

- **`src/Viewer.tsx`**: Main UI, handles file selection, renders the `Publication`
- **`src/hooks/useEpubNavigator.ts`**: Manages the EpubNavigator lifecycle and keyboard bindings
- **`src/fetchers/TauriFetcher.ts`**: Readium Fetcher implementation that talks to Tauri commands
- **`src/fetchers/openEpub.ts`**: Opens an EPUB and builds a `Publication`
- **`src/peripherals.ts`**: Keyboard event wiring (arrow keys, Space)

### Backend (Rust)

- **`src-tauri/src/lib.rs`**: Entry point, registers the custom protocol
- **`src-tauri/src/commands.rs`**: Tauri commands
  - `open_epub(path)`: Open the EPUB, generate a manifest, store it in `EpubState`
  - `get_manifest(id)`: Return the manifest JSON
  - `get_epub_resource(id, path)`: Read resources from the ZIP, inject a `<base>` tag for HTML
  - `close_epub(id)`: Remove the publication from state
- **`src-tauri/src/epub/parser.rs`**: EPUB parser
  - Open ZIP → `container.xml` → OPF → Webpub Manifest
- **`src-tauri/src/epub/manifest.rs`**: Webpub Manifest data structures

**EpubState**: Global map `HashMap<PublicationID, OpenedPublication>` that keeps the currently opened books.

## Summary

Leafmate’s design centers on:

1. **EPUB → Webpub Manifest conversion**: Parse EPUBs in Rust and expose them in Readium’s format
2. **Custom protocol**: Serve resources via `tauri://epub/{id}/` and resolve iframe-relative paths
3. **TauriFetcher**: Implement Readium Fetcher to access native files
4. **Base tag injection**: Inject `<base>` tags into HTML to fix relative references
