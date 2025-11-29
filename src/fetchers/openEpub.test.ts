import { describe, it, expect, vi, beforeEach } from "vitest";
import { openEpub, closeEpub } from "./openEpub";
import { Publication } from "@readium/shared";

// Mock Tauri's invoke function
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

describe("openEpub", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should open EPUB and return Publication", async () => {
    const mockId = "test-pub-id";
    const mockManifest = {
      "@context": "https://readium.org/webpub-manifest/context.jsonld",
      metadata: {
        title: "Test Book",
        language: ["en"],
      },
      links: [],
      readingOrder: [
        { href: "chapter1.xhtml", type: "application/xhtml+xml" },
      ],
    };

    // Mock open_epub command
    vi.mocked(invoke).mockResolvedValueOnce({ id: mockId });
    // Mock get_manifest command
    vi.mocked(invoke).mockResolvedValueOnce(JSON.stringify(mockManifest));

    const publication = await openEpub("/path/to/book.epub");

    expect(invoke).toHaveBeenCalledWith("open_epub", {
      path: "/path/to/book.epub",
    });
    expect(invoke).toHaveBeenCalledWith("get_manifest", { id: mockId });
    expect(publication).toBeInstanceOf(Publication);
    expect(publication.metadata.title).toBeDefined();
  });

  it("should throw error when manifest is invalid", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ id: "test-id" });
    vi.mocked(invoke).mockResolvedValueOnce("invalid json");

    await expect(openEpub("/path/to/invalid.epub")).rejects.toThrow();
  });

  it("should throw error when manifest deserialization fails", async () => {
    const invalidManifest = {
      // Missing required fields
      metadata: {},
    };

    vi.mocked(invoke).mockResolvedValueOnce({ id: "test-id" });
    vi.mocked(invoke).mockResolvedValueOnce(JSON.stringify(invalidManifest));

    await expect(openEpub("/path/to/broken.epub")).rejects.toThrow(
      "Failed to deserialize manifest"
    );
  });
});

describe("closeEpub", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should call close_epub command", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await closeEpub("test-pub-id");

    expect(invoke).toHaveBeenCalledWith("close_epub", { id: "test-pub-id" });
  });
});
