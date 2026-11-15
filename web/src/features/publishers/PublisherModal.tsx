import { useState } from "react";
import type { PublisherResponseDto, PublisherCreateDto } from "../../api/model";
import EditorForm from "../../components/EditorForm";
import { InputField } from "../../components/FormField";
import Modal from "../../components/Modal";

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
  const [name, setName] = useState(publisher?.name ?? "");

  return (
    <Modal
      title={publisher ? "出版社情報を編集" : "新しい出版社を追加"}
      onClose={onClose}
      isSubmitting={isSubmitting}
    >
      <EditorForm
        onSubmit={() => onSubmit({ name })}
        onClose={onClose}
        isSubmitting={isSubmitting}
        isEditing={Boolean(publisher)}
      >
        <InputField
          label="出版社名"
          required
          placeholder="例: デジタル出版株式会社"
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
      </EditorForm>
    </Modal>
  );
}
