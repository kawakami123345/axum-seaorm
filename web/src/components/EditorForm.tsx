import { useState, type FormEvent, type ReactNode } from "react";
import { getErrorMessage } from "../lib/custom-fetch";
import Icon from "./Icon";

interface EditorFormProps {
  onSubmit: () => Promise<void>;
  onClose: () => void;
  isSubmitting?: boolean;
  isEditing: boolean;
  children: ReactNode;
}

export default function EditorForm({
  onSubmit,
  onClose,
  isSubmitting = false,
  isEditing,
  children,
}: EditorFormProps) {
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (isSubmitting) return;
    setError(null);
    try {
      await onSubmit();
    } catch (error) {
      setError(`保存に失敗しました: ${getErrorMessage(error)}`);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="p-6">
      <fieldset disabled={isSubmitting} className="flex flex-col gap-4">
        {error && (
          <div
            role="alert"
            className="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded text-red-600 dark:text-red-400 text-xs font-bold flex items-center gap-2"
          >
            <Icon name="error" className="text-sm" />
            {error}
          </div>
        )}
        {children}
        <div className="mt-4 flex gap-3">
          <button
            type="button"
            onClick={onClose}
            disabled={isSubmitting}
            className="flex-1 px-4 py-2 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-text-main dark:text-white font-bold rounded-md transition-colors disabled:opacity-50"
          >
            キャンセル
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            className="flex-1 px-4 py-2 bg-primary hover:bg-primary-dark text-white font-bold rounded-md shadow-sm transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
          >
            {isSubmitting && (
              <Icon name="sync" className="animate-spin text-sm" />
            )}
            {isEditing ? "更新する" : "保存する"}
          </button>
        </div>
      </fieldset>
    </form>
  );
}
