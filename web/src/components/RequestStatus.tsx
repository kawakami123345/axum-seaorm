import Icon from "./Icon";

export interface RequestStatusProps {
  status: "loading" | "error";
  message?: string;
  onRetry?: () => void;
  isRetrying?: boolean;
}

export default function RequestStatus({
  status,
  message,
  onRetry,
  isRetrying = false,
}: RequestStatusProps) {
  if (status === "loading") {
    return (
      <div role="status" className="text-text-sub flex items-center gap-2">
        <Icon name="sync" className="animate-spin text-primary" />
        {message ?? "読み込み中..."}
      </div>
    );
  }

  return (
    <div
      role="alert"
      className="text-accent-red flex flex-col items-center gap-2"
    >
      <Icon name="error" className="text-4xl" />
      <p>{message ?? "データの取得に失敗しました。"}</p>
      {onRetry && (
        <button
          type="button"
          onClick={onRetry}
          disabled={isRetrying}
          className="text-primary underline disabled:opacity-50"
        >
          再試行
        </button>
      )}
    </div>
  );
}
