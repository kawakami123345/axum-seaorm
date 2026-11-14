import { useEffect, useId, useRef, type ReactNode } from "react";

interface ModalProps {
  title: string;
  onClose: () => void;
  isSubmitting?: boolean;
  children: ReactNode;
}

export default function Modal({
  title,
  onClose,
  isSubmitting,
  children,
}: ModalProps) {
  const titleId = useId();
  const ref = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = ref.current;
    const previousFocus = document.activeElement;
    dialog?.showModal();
    dialog?.querySelector<HTMLInputElement>("input:not([disabled])")?.focus();

    return () => {
      dialog?.close();
      if (previousFocus instanceof HTMLElement) previousFocus.focus();
    };
  }, []);

  return (
    <dialog
      ref={ref}
      aria-labelledby={titleId}
      className="m-auto w-[calc(100%_-_2rem)] max-w-md max-h-[calc(100dvh_-_2rem)] overflow-y-auto rounded-lg border-0 bg-white p-0 text-text-main shadow-xl backdrop:bg-black/50 backdrop:backdrop-blur-sm dark:bg-card-dark dark:text-white"
      onCancel={(event) => {
        event.preventDefault();
        if (!isSubmitting) onClose();
      }}
    >
      <div className="px-6 py-4 border-b border-border-color dark:border-gray-700 flex justify-between items-center">
        <h2 id={titleId} className="text-lg font-bold">
          {title}
        </h2>
        <button
          type="button"
          aria-label="閉じる"
          onClick={onClose}
          disabled={isSubmitting}
          className="text-text-sub hover:text-text-main disabled:opacity-50"
        >
          <span aria-hidden="true" className="material-symbols-outlined">
            close
          </span>
        </button>
      </div>
      {children}
    </dialog>
  );
}

export function FormError({ message }: { message: string | null }) {
  if (!message) return null;
  return (
    <div
      role="alert"
      className="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded text-red-600 dark:text-red-400 text-xs font-bold flex items-center gap-2"
    >
      <span aria-hidden="true" className="material-symbols-outlined text-sm">
        error
      </span>
      {message}
    </div>
  );
}

export function FormActions({
  onClose,
  isSubmitting,
  isEditing,
}: {
  onClose: () => void;
  isSubmitting?: boolean;
  isEditing: boolean;
}) {
  return (
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
          <span
            aria-hidden="true"
            className="material-symbols-outlined animate-spin text-sm"
          >
            sync
          </span>
        )}
        {isEditing ? "更新する" : "保存する"}
      </button>
    </div>
  );
}
