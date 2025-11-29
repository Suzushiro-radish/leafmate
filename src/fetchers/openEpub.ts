/**
 * Helper functions to open EPUB files via Tauri commands.
 */

import { invoke } from "@tauri-apps/api/core";
import { Manifest, Publication } from "@readium/shared";
import { TauriFetcher } from "./TauriFetcher";

interface OpenEpubResult {
  id: string;
}

/**
 * Open an EPUB file and create a Publication object.
 *
 * @param path - Absolute path to the EPUB file
 * @returns Publication object ready for use with EpubNavigator
 */
export async function openEpub(path: string): Promise<Publication> {
  // Open EPUB via Tauri command
  const { id } = await invoke<OpenEpubResult>("open_epub", { path });

  // Get manifest JSON
  const manifestJson = await invoke<string>("get_manifest", { id });
  const manifestData = JSON.parse(manifestJson);

  // Deserialize manifest
  const manifest = Manifest.deserialize(manifestData);
  if (!manifest) {
    throw new Error("Failed to deserialize manifest");
  }

  // Create fetcher for this publication
  const fetcher = new TauriFetcher(id);

  // Create Publication
  const publication = new Publication({
    manifest,
    fetcher,
  });

  return publication;
}

/**
 * Close an opened EPUB publication.
 *
 * @param publicationId - The publication ID returned from openEpub
 */
export async function closeEpub(publicationId: string): Promise<void> {
  await invoke("close_epub", { id: publicationId });
}
