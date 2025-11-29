import { EpubNavigator } from "@readium/navigator";

const iconButtonClasses =
  "inline-flex h-9 w-9 items-center justify-center rounded-full border border-slate-200 bg-white text-base font-medium text-slate-600 shadow-sm transition hover:bg-slate-100 hover:text-slate-800 active:bg-slate-200";

interface NavigationButtonsProps {
  navRef: React.RefObject<EpubNavigator | null>;
}

export const NavigationButtons: React.FC<NavigationButtonsProps> = ({ navRef }) => {
  return (
    <>
      <button
        type="button"
        className={iconButtonClasses}
        title="Go left"
        onClick={() => navRef.current?.goLeft(true, () => {})}
      >
        ←
      </button>
      <button
        type="button"
        className={iconButtonClasses}
        title="Go right"
        onClick={() => navRef.current?.goRight(true, () => {})}
      >
        →
      </button>
    </>
  );
};
