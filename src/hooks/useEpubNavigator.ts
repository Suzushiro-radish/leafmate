import { useEffect, useRef, useState } from "react";
import {
  EpubNavigator,
  EpubNavigatorListeners,
  FrameManager,
  FXLFrameManager,
} from "@readium/navigator";
import { Publication } from "@readium/shared";
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
  positionChanged: () => {},
  tap: () => false,
  click: () => false,
  zoom: () => {},
  miscPointer: () => {},
  scroll: () => {},
  customEvent: () => {},
  handleLocator: () => false,
  textSelected: () => {},
});

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

        peripherals = setupPeripherals(navRef);
        const listeners = buildNavigatorListeners(navRef, peripherals);

        navRef.current = new EpubNavigator(
          containerRef.current,
          publication,
          listeners,
        );
        await navRef.current.load();

        if (disposed) {
          (navRef.current as any)?.destroy?.();
          return;
        }

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

  return { navRef, isLoading, error };
};
