import { useState } from "react";
import type { BookResponseDto, BookCreateDto } from "../../api/model";
import { useGetAllPublishers } from "../../api/endpoints/publisher/publisher";
import { useGetAllShops } from "../../api/endpoints/shop/shop";
import EditorForm from "../../components/EditorForm";
import { InputField, SelectField } from "../../components/FormField";
import Modal from "../../components/Modal";
import RequestStatus from "../../components/RequestStatus";
import { formatDate, formatYen } from "../../lib/format";

interface BookModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: BookCreateDto) => Promise<void>;
  book?: BookResponseDto;
  isSubmitting?: boolean;
}

export default function BookModal({ isOpen, ...props }: BookModalProps) {
  if (!isOpen) return null;
  const book = props.book;
  if (book && (book.applied_at || book.applied_without_date)) {
    const details = [
      ["タイトル", book.title],
      ["著者", book.author],
      ["出版社", book.publisher.name],
      ["購入店舗", book.shop?.name ?? "未指定"],
      ["価格", formatYen(book.price)],
      ["形式", book.format === "EBook" ? "電子書籍" : "単行本/文庫"],
      ["申請日", book.applied_at ? formatDate(book.applied_at) : "日付不明"],
    ];
    return (
      <Modal title="申請済み書籍の詳細" onClose={props.onClose}>
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 p-6">
          {details.map(([label, value]) => (
            <div key={label} className="contents">
              <dt className="text-text-sub">{label}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      </Modal>
    );
  }
  return <BookModalForm key={props.book?.pub_id ?? "new"} {...props} />;
}

function BookModalForm({
  onClose,
  onSubmit,
  book,
  isSubmitting,
}: Omit<BookModalProps, "isOpen">) {
  const publisherQuery = useGetAllPublishers({
    query: { select: (response) => response.data },
  });
  const shopQuery = useGetAllShops({
    query: { select: (response) => response.data },
  });
  const publishers = publisherQuery.data ?? [];
  const shops = shopQuery.data ?? [];
  const hasError = publisherQuery.isError || shopQuery.isError;
  const missingChoices =
    publisherQuery.data === undefined || shopQuery.data === undefined;
  const retryChoices = () => {
    void Promise.all([publisherQuery.refetch(), shopQuery.refetch()]);
  };
  const [formData, setFormData] = useState<BookCreateDto>(() => ({
    title: book?.title ?? "",
    author: book?.author ?? "",
    price: book?.price ?? 0,
    publisher_id: book?.publisher.pub_id ?? "",
    format: book?.format || "Real",
    shop_id: book?.shop?.pub_id ?? null,
  }));

  const setField = <K extends keyof BookCreateDto>(
    field: K,
    value: BookCreateDto[K],
  ) => {
    setFormData((current) => ({ ...current, [field]: value }));
  };

  return (
    <Modal
      title={book ? "書籍情報を編集" : "新しい書籍を追加"}
      onClose={onClose}
      isSubmitting={isSubmitting}
    >
      {missingChoices ? (
        <div className="p-6">
          <RequestStatus
            status={hasError ? "error" : "loading"}
            message={
              hasError
                ? "出版社・店舗の取得に失敗しました。"
                : "選択肢を読み込み中..."
            }
            onRetry={retryChoices}
            isRetrying={publisherQuery.isFetching || shopQuery.isFetching}
          />
        </div>
      ) : (
        <EditorForm
          onSubmit={() => onSubmit(formData)}
          onClose={onClose}
          isSubmitting={isSubmitting}
          isEditing={Boolean(book)}
        >
          {hasError && (
            <RequestStatus
              status="error"
              message="選択肢を更新できませんでした。前回の選択肢を表示しています。"
              onRetry={retryChoices}
              isRetrying={publisherQuery.isFetching || shopQuery.isFetching}
            />
          )}
          {publishers.length === 0 && (
            <div className="p-3 bg-amber-50 dark:bg-amber-900/30 border border-amber-200 dark:border-amber-800 rounded text-amber-700 dark:text-amber-400 text-xs font-bold leading-normal">
              出版社が登録されていません。先に「出版社管理」から出版社を登録してください。
            </div>
          )}
          <InputField
            label="タイトル"
            autoFocus
            required
            value={formData.title}
            onChange={(event) => setField("title", event.target.value)}
          />
          <InputField
            label="著者"
            required
            value={formData.author}
            onChange={(event) => setField("author", event.target.value)}
          />
          <div className="grid grid-cols-2 gap-4">
            <InputField
              label="価格"
              type="number"
              required
              min={0}
              step={1}
              value={formData.price}
              onChange={(event) =>
                setField("price", Number(event.target.value))
              }
            />
            <SelectField
              label="形式"
              value={formData.format || "Real"}
              onChange={(event) => setField("format", event.target.value)}
            >
              <option value="Real">単行本/文庫</option>
              <option value="EBook">Kindle/電子書籍</option>
            </SelectField>
          </div>
          <SelectField
            label="出版社"
            required
            value={formData.publisher_id}
            onChange={(event) => setField("publisher_id", event.target.value)}
          >
            <option value="" disabled>
              出版社を選択してください
            </option>
            {publishers.map((publisher) => (
              <option key={publisher.pub_id} value={publisher.pub_id}>
                {publisher.name}
              </option>
            ))}
          </SelectField>
          <SelectField
            label="購入店舗 (任意)"
            value={formData.shop_id || ""}
            onChange={(event) =>
              setField("shop_id", event.target.value || null)
            }
          >
            <option value="">未指定</option>
            {shops.map((shop) => (
              <option key={shop.pub_id} value={shop.pub_id}>
                {shop.name}
              </option>
            ))}
          </SelectField>
        </EditorForm>
      )}
    </Modal>
  );
}
