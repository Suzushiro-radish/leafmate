import { useCallback, useEffect, useRef, useState } from "react";
import {
  EpubNavigator,
  EpubNavigatorListeners,
  FrameManager,
  FXLFrameManager,
} from "@readium/navigator";
import { Link, Locator, Publication } from "@readium/shared";
import Peripherals from "../peripherals";

const setupPeripherals = (navRef: React.RefObject<EpubNavigator | null>) =>
  new Peripherals({
    moveTo: (direction) => {
      if (direction === "right") navRef.current?.goRight(true, () => {});
      else navRef.current?.goLeft(true, () => {});
    },
    goForward: () => {
      navRef.current?.goForward(true, () => {});
    },
  });

const fetchHtmlFromPublication = async (
  pub: Publication | null,
  locator: Locator,
) => {
  if (!pub) return "";
  if (!locator?.href) return "";

  try {
    const link = new Link({ href: locator.href });
    const resource = pub.get(link);
    const content = await resource.readAsString();
    resource.close();
    return content ?? "";
  } catch (err) {
    console.error("Failed to read HTML for locator", locator, err);
    return "";
  }
};

const buildNavigatorListeners = (
  navRef: React.RefObject<EpubNavigator | null>,
  peripherals: Peripherals,
  onLocationChange: (locator: Locator) => void,
): EpubNavigatorListeners => ({
  frameLoaded: () => {
    navRef.current?._cframes.forEach(
      (frameManager: FrameManager | FXLFrameManager | undefined) => {
        if (frameManager?.window) peripherals.observe(frameManager.window);
      },
    );
    peripherals.observe(window);
  },
  positionChanged: (locator) => {
    onLocationChange(locator);
  },
  tap: () => false,
  click: () => false,
  zoom: () => {},
  miscPointer: () => {},
  scroll: () => {},
  customEvent: () => {},
  handleLocator: () => false,
  textSelected: () => {},
});

export type VisibleHtmlPayload = {
  locator: Locator | null;
  html: string;
};

/**
 * Hook to display an EPUB publication with EpubNavigator.
 *
 * @param containerRef - Ref to the container div for the navigator
 * @param publication - Publication object to display
 */
export const useEpubNavigator = (
  containerRef: React.RefObject<HTMLDivElement | null>,
  publication: Publication | null,
) => {
  const navRef = useRef<EpubNavigator | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const currentLocatorRef = useRef<Locator | null>(null);

  useEffect(() => {
    if (!publication) return;

    let disposed = false;
    let peripherals: Peripherals | null = null;

    const loadPublication = async () => {
      try {
        setIsLoading(true);
        setError(null);

        if (!containerRef.current) {
          throw new Error("Reader container not mounted");
        }

        // Set up keyboard/controller peripherals and hook locator updates.
        peripherals = setupPeripherals(navRef);
        const listeners = buildNavigatorListeners(
          navRef,
          peripherals,
          (locator) => {
            currentLocatorRef.current = locator;
          },
        );

        // Initialize positions
        // How can I get positions from the publication without using "any"?
        const positions = (publication as any).positions;

        // Instantiate navigator with precomputed positions and load initial page.
        navRef.current = new EpubNavigator(
          containerRef.current,
          publication,
          listeners,
          positions,
        );

        await navRef.current.load();

        if (disposed) {
          (navRef.current as any)?.destroy?.();
          return;
        }

        // Attach peripherals to the host window once frames are ready.
        peripherals?.observe(window);
        setIsLoading(false);
      } catch (err) {
        console.error("Error loading publication", err);
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
      if (
        navRef.current &&
        typeof (navRef.current as any).destroy === "function"
      ) {
        try {
          (navRef.current as any).destroy();
        } catch (err) {
          console.warn("Failed to destroy navigator", err);
        }
      }
      navRef.current = null;
    };
  }, [containerRef, publication]);

  const getVisibleHtml = useCallback(async (): Promise<VisibleHtmlPayload> => {
    const locator = currentLocatorRef.current;
    if (!locator || !publication) {
      return { locator: null, html: "" };
    }

    const html = await fetchHtmlFromPublication(publication, locator);
    return { locator, html };
  }, [publication]);

  return { navRef, isLoading, error, getVisibleHtml };
};
