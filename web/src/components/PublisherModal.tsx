import { useId, useState, type FormEvent } from "react";
import type { PublisherResponseDto, PublisherCreateDto } from "../api/model";
import { getErrorMessage } from "../lib/custom-fetch";
import Modal, { FormActions, FormError } from "./Modal";

interface PublisherModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: PublisherCreateDto) => Promise<void>;
  publisher?: PublisherResponseDto;
  isSubmitting?: boolean;
}

export default function PublisherModal({
  isOpen,
  ...props
}: PublisherModalProps) {
  if (!isOpen) return null;
  return (
    <PublisherModalForm key={props.publisher?.pub_id ?? "new"} {...props} />
  );
}

function PublisherModalForm({
  onClose,
  onSubmit,
  publisher,
  isSubmitting,
}: Omit<PublisherModalProps, "isOpen">) {
  const id = useId();
  const [error, setError] = useState<string | null>(null);
  const [formData, setFormData] = useState<PublisherCreateDto>(() => ({
    name: publisher?.name ?? "",
  }));

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (isSubmitting) return;
    setError(null);
    try {
      await onSubmit(formData);
    } catch (error) {
      setError(`保存に失敗しました: ${getErrorMessage(error)}`);
    }
  };

  return (
    <Modal
      title={publisher ? "出版社情報を編集" : "新しい出版社を追加"}
      onClose={onClose}
      isSubmitting={isSubmitting}
    >
      <form onSubmit={handleSubmit} className="p-6">
        <fieldset disabled={isSubmitting} className="flex flex-col gap-4">
          <FormError message={error} />
          <div>
            <label
              htmlFor={id}
              className="block text-xs font-bold text-text-sub uppercase mb-1"
            >
              出版社名
            </label>
            <input
              id={id}
              required
              placeholder="例: デジタル出版株式会社"
              className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
              value={formData.name}
              onChange={(event) =>
                setFormData({ ...formData, name: event.target.value })
              }
            />
          </div>
          <FormActions
            onClose={onClose}
            isSubmitting={isSubmitting}
            isEditing={Boolean(publisher)}
          />
        </fieldset>
      </form>
    </Modal>
  );
}
