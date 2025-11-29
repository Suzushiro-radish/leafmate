import { describe, it, expect, vi, beforeEach } from "vitest";
import { TauriFetcher } from "./TauriFetcher";
import { Link } from "@readium/shared";

// Mock Tauri's invoke function
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

describe("TauriFetcher", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("constructor", () => {
    it("should create a fetcher with publication ID", () => {
      const fetcher = new TauriFetcher("test-pub-id");
      expect(fetcher).toBeInstanceOf(TauriFetcher);
    });
  });

  describe("links", () => {
    it("should return empty array", () => {
      const fetcher = new TauriFetcher("test-pub-id");
      expect(fetcher.links()).toEqual([]);
    });
  });

  describe("get", () => {
    it("should return a TauriResource", () => {
      const fetcher = new TauriFetcher("test-pub-id");
      const link = new Link({ href: "chapter1.xhtml" });
      const resource = fetcher.get(link);

      expect(resource).toBeDefined();
    });
  });

  describe("TauriResource", () => {
    it("should read resource via Tauri command", async () => {
      const mockData = [72, 101, 108, 108, 111]; // "Hello" in bytes
      vi.mocked(invoke).mockResolvedValue(mockData);

      const fetcher = new TauriFetcher("test-pub-id");
      const link = new Link({ href: "OEBPS/chapter1.xhtml" });
      const resource = fetcher.get(link);

      const data = await resource.read();

      expect(invoke).toHaveBeenCalledWith("get_epub_resource", {
        id: "test-pub-id",
        path: "OEBPS/chapter1.xhtml",
      });
      expect(data).toBeInstanceOf(Uint8Array);
      expect(Array.from(data!)).toEqual(mockData);
    });

    it("should return undefined when Tauri command fails", async () => {
      vi.mocked(invoke).mockRejectedValue(new Error("File not found"));

      const fetcher = new TauriFetcher("test-pub-id");
      const link = new Link({ href: "missing.xhtml" });
      const resource = fetcher.get(link);

      const data = await resource.read();

      expect(data).toBeUndefined();
    });

    it("should throw error for range reads", async () => {
      const fetcher = new TauriFetcher("test-pub-id");
      const link = new Link({ href: "chapter1.xhtml" });
      const resource = fetcher.get(link);

      await expect(resource.read({ start: 0, endInclusive: 100 })).rejects.toThrow(
        "Range reads are not supported by TauriFetcher"
      );
    });

    it("should return link", async () => {
      const fetcher = new TauriFetcher("test-pub-id");
      const link = new Link({ href: "chapter1.xhtml" });
      const resource = fetcher.get(link);

      const resultLink = await resource.link();

      expect(resultLink).toBe(link);
    });

    it("should return undefined for length", async () => {
      const fetcher = new TauriFetcher("test-pub-id");
      const link = new Link({ href: "chapter1.xhtml" });
      const resource = fetcher.get(link);

      const length = await resource.length();

      expect(length).toBeUndefined();
    });

    it("should read as string", async () => {
      const text = "Hello, World!";
      const bytes = new TextEncoder().encode(text);
      vi.mocked(invoke).mockResolvedValue(Array.from(bytes));

      const fetcher = new TauriFetcher("test-pub-id");
      const link = new Link({ href: "text.txt" });
      const resource = fetcher.get(link);

      const result = await resource.readAsString();

      expect(result).toBe(text);
    });

    it("should read as JSON", async () => {
      const json = { title: "Test Book", author: "Test Author" };
      const jsonString = JSON.stringify(json);
      const bytes = new TextEncoder().encode(jsonString);
      vi.mocked(invoke).mockResolvedValue(Array.from(bytes));

      const fetcher = new TauriFetcher("test-pub-id");
      const link = new Link({ href: "manifest.json" });
      const resource = fetcher.get(link);

      const result = await resource.readAsJSON();

      expect(result).toEqual(json);
    });
  });

  describe("close", () => {
    it("should not throw when closed", () => {
      const fetcher = new TauriFetcher("test-pub-id");
      expect(() => fetcher.close()).not.toThrow();
    });
  });
});
