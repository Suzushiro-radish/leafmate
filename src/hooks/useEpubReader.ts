import { useEffect, useRef, useState } from "react";
import {
  EpubNavigator,
  EpubNavigatorListeners,
  FrameManager,
  FXLFrameManager,
} from "@readium/navigator";
import { Fetcher, HttpFetcher, Link, Manifest, Publication } from "@readium/shared";
import Peripherals from "../peripherals";

const fetchPublication = async (manifestBase: string) => {
  const manifestLink = new Link({ href: "manifest.json" });
  const fetcher: Fetcher = new HttpFetcher(undefined, manifestBase);
  const fetched = fetcher.get(manifestLink);
  const selfLink = (await fetched.link()).toURL(manifestBase)!;
  const response = await fetched.readAsJSON();
  const manifestData =
    typeof response === "string" ? JSON.parse(response) : response;
  const loadedManifest = Manifest.deserialize(manifestData);
  if (!loadedManifest) {
    throw new Error("Failed to deserialize manifest");
  }
  loadedManifest.setSelfLink(selfLink);
  const publication = new Publication({
    manifest: loadedManifest,
    fetcher,
  });
  const title =
    loadedManifest.metadata.title.getTranslation("en") || "Untitled";
  return { publication, title };
};

const setupPeripherals = (navRef: React.RefObject<EpubNavigator | null>) =>
  new Peripherals({
    moveTo: (direction) => {
      if (direction === "right") navRef.current?.goRight(true);
      else navRef.current?.goLeft(true);
    },
    goForward: () => {
      navRef.current?.goForward(true);
    },
  });

const buildNavigatorListeners = (
  navRef: React.RefObject<EpubNavigator | null>,
  peripherals: Peripherals,
): EpubNavigatorListeners => ({
  frameLoaded: () => {
    navRef.current?._cframes.forEach(
      (frameManager: FrameManager | FXLFrameManager | undefined) => {
        if (frameManager?.window) peripherals.observe(frameManager.window);
      },
    );
    peripherals.observe(window);
  },
});

export const useEpubReader = (
  containerRef: React.RefObject<HTMLDivElement | null>,
  manifestBase: string,
) => {
  const navRef = useRef<EpubNavigator | null>(null);
  const [title, setTitle] = useState<string>("Loading…");
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    let peripherals: Peripherals | null = null;

    const loadPublication = async () => {
      try {
        setIsLoading(true);
        setError(null);

        if (!containerRef.current) {
          throw new Error("Reader container not mounted");
        }

        const { publication, title: publicationTitle } =
          await fetchPublication(manifestBase);
        if (disposed) return;

        setTitle(publicationTitle);

        peripherals = setupPeripherals(navRef);
        const listeners = buildNavigatorListeners(navRef, peripherals);

        navRef.current = new EpubNavigator(containerRef.current, publication, listeners);
        await navRef.current.load();

        if (disposed) {
          (navRef.current as any)?.destroy?.();
          return;
        }

        peripherals?.observe(window);
        setIsLoading(false);
      } catch (err) {
        console.error("Error loading manifest", err);
        if (disposed) return;
        const message =
          err instanceof Error
            ? err.message
            : `Unexpected error: ${String(err)}`;
        setError(message);
        setIsLoading(false);
      }
    };

    loadPublication();

    return () => {
      disposed = true;
      peripherals?.destroy();
      peripherals = null;
      if (navRef.current && typeof (navRef.current as any).destroy === "function") {
        try {
          (navRef.current as any).destroy();
        } catch (err) {
          console.warn("Failed to destroy navigator", err);
        }
      }
      navRef.current = null;
    };
  }, [containerRef, manifestBase]);

  return { navRef, title, isLoading, error };
};
