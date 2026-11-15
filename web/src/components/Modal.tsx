import { useEffect, useId, useRef, type ReactNode } from "react";
import Icon from "./Icon";

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
          <Icon name="close" />
        </button>
      </div>
      {children}
    </dialog>
  );
}
