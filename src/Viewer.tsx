import { useRef } from "react";
import { useEpubReader } from "./hooks/useEpubReader";
import { NavigationButtons } from "./components/NavigationButtons";
import { StatusMessage } from "./components/StatusMessage";

const Viewer: React.FC = () => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const { navRef, title, isLoading, error } = useEpubReader(containerRef);

  return (
    <div className="relative flex h-dvh flex-col bg-gradient-to-b from-slate-100 to-slate-200">
      <header
        id="top-bar"
        aria-label="Top Bar"
        className="flex h-12 shrink-0 items-center justify-between gap-2 border-b border-slate-200 bg-white/90 px-4 shadow-sm backdrop-blur"
      >
        <h3 className="text-sm font-semibold text-slate-700 sm:text-base">
          {title}
        </h3>
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
