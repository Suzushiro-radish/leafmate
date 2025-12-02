import { useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Publication } from "@readium/shared";
import { openEpub } from "./fetchers";
import { useEpubNavigator } from "./hooks/useEpubNavigator";
import { NavigationButtons } from "./components/NavigationButtons";
import { StatusMessage } from "./components/StatusMessage";

const Viewer: React.FC = () => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [publication, setPublication] = useState<Publication | null>(null);
  const [title, setTitle] = useState<string>("No EPUB opened");
  const { navRef, isLoading, error } = useEpubNavigator(
    containerRef,
    publication,
  );

  const handleOpenFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "EPUB",
            extensions: ["epub"],
          },
        ],
      });

      if (!selected) return;

      const pub = await openEpub(selected);
      setPublication(pub);

      // Get title from publication
      const pubTitle = pub.metadata.title?.getTranslation?.("en") || "Untitled";
      setTitle(pubTitle);
    } catch (err) {
      console.error("Failed to open EPUB:", err);
    }
  };

  return (
    <div className="relative flex h-dvh flex-col bg-linear-to-b from-slate-100 to-slate-200">
      <header
        id="top-bar"
        aria-label="Top Bar"
        className="flex h-12 shrink-0 items-center justify-between gap-2 border-b border-slate-200 bg-white/90 px-4 shadow-sm backdrop-blur"
      >
        <h3 className="text-sm font-semibold text-slate-700 sm:text-base">
          {title}
        </h3>
        <button
          type="button"
          onClick={handleOpenFile}
          className="rounded-md bg-blue-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-blue-700 active:bg-blue-800"
        >
          Open EPUB
        </button>
      </header>

      <div id="wrapper" className="relative min-h-0 flex-1 overflow-hidden">
        <main
          id="container"
          aria-label="Publication"
          ref={containerRef}
          className="absolute inset-0 bg-white"
          style={{ contain: "content" }}
        />
      </div>

      <footer
        id="bottom-bar"
        aria-label="Bottom Bar"
        className="flex h-12 shrink-0 items-center justify-center gap-2 border-t border-slate-200 bg-white/90 px-4 shadow-sm backdrop-blur"
      >
        <NavigationButtons navRef={navRef} />
      </footer>

      <StatusMessage isLoading={isLoading} error={error} />
    </div>
  );
};

export default Viewer;
