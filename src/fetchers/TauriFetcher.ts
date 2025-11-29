/**
 * TauriFetcher - A Fetcher implementation that reads EPUB resources via Tauri commands.
 */

import { invoke } from "@tauri-apps/api/core";
import type { Fetcher, Link, NumberRange } from "@readium/shared";
import { Resource } from "@readium/shared";

export class TauriFetcher implements Fetcher {
  constructor(private readonly publicationId: string) {}

  links(): Link[] {
    // We don't pre-enumerate resources in this implementation
    return [];
  }

  get(link: Link): Resource {
    return new TauriResource(this.publicationId, link);
  }

  close(): void {
    // Resources are cleaned up when the publication is closed
  }
}

class TauriResource extends Resource {
  constructor(
    private readonly publicationId: string,
    private readonly _link: Link,
  ) {
    super();
  }

  async link(): Promise<Link> {
    return this._link;
  }

  async length(): Promise<number | undefined> {
    // Length is not available without reading the entire resource
    // Could be optimized by adding a HEAD-like command
    return undefined;
  }

  async read(range?: NumberRange): Promise<Uint8Array | undefined> {
    if (range) {
      throw new Error("Range reads are not supported by TauriFetcher");
    }

    try {
      const data = await invoke<number[]>("get_epub_resource", {
        id: this.publicationId,
        path: this._link.href,
      });

      return new Uint8Array(data);
    } catch (error) {
      console.error(`Failed to read resource ${this._link.href}:`, error);
      return undefined;
    }
  }

  close(): void {
    // No-op
  }
}
