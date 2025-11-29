/**
 * Helper functions to open EPUB files via Tauri commands.
 */

import { invoke } from "@tauri-apps/api/core";
import { Locator, Manifest, Publication } from "@readium/shared";
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

  // Set self link (required for locator generation)
  const selfLink = `tauri://epub/${id}/manifest.json`;
  manifest.setSelfLink(selfLink);

  // Create fetcher for this publication
  const fetcher = new TauriFetcher(id);

  // Create Publication
  const publication = new Publication({
    manifest,
    fetcher,
  });

  // Generate positions from readingOrder
  // Each item in readingOrder becomes a position with location metadata
  const positions = manifestData.readingOrder.map((link: any, index: number) => ({
    href: link.href,
    type: link.type,
    locations: {
      position: index + 1,
      progression: index / manifestData.readingOrder.length,
    },
  }));

  // Create Locator instances for positions
  const locators = positions.map((pos: any) => new Locator({
    href: pos.href,
    type: pos.type,
    locations: pos.locations,
  }));

  // Set positions on Publication object for EpubNavigator
  (publication as any).positions = locators;

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
