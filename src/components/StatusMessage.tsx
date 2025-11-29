interface StatusMessageProps {
  isLoading: boolean;
  error: string | null;
}

export const StatusMessage: React.FC<StatusMessageProps> = ({ isLoading, error }) => {
  if (isLoading && !error) {
    return (
      <div className="absolute right-4 top-16 rounded-md bg-slate-900/80 px-3 py-1.5 text-xs font-medium text-white shadow-lg">
        ドキュメントを読み込み中…
      </div>
    );
  }

  if (error) {
    return (
      <div className="absolute right-4 top-16 rounded-md bg-red-600 px-3 py-1.5 text-xs font-semibold text-white shadow-lg">
        エラー: {error}
      </div>
    );
  }

  return null;
};
