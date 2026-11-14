interface PageStatusProps {
  status: "loading" | "error";
}

export default function PageStatus({ status }: PageStatusProps) {
  return (
    <main className="flex-1 min-w-0 flex items-center justify-center h-64">
      {status === "loading" ? (
        <div role="status" className="text-text-sub flex items-center gap-2">
          <span
            aria-hidden="true"
            className="material-symbols-outlined animate-spin text-primary"
          >
            sync
          </span>
          読み込み中...
        </div>
      ) : (
        <div
          role="alert"
          className="text-accent-red font-bold flex flex-col items-center gap-2"
        >
          <span
            aria-hidden="true"
            className="material-symbols-outlined text-4xl"
          >
            error
          </span>
          データの取得に失敗しました。
        </div>
      )}
    </main>
  );
}
