import { useId, useState, type FormEvent } from "react";
import type { BookResponseDto, BookCreateDto } from "../api/model";
import { useGetAllPublishers } from "../api/endpoints/publisher/publisher";
import { useGetAllShops } from "../api/endpoints/shop/shop";
import { getErrorMessage } from "../lib/custom-fetch";
import Modal, { FormActions, FormError } from "./Modal";

interface BookModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: BookCreateDto) => Promise<void>;
  book?: BookResponseDto;
  isSubmitting?: boolean;
}

export default function BookModal({ isOpen, ...props }: BookModalProps) {
  if (!isOpen) return null;
  return <BookModalForm key={props.book?.pub_id ?? "new"} {...props} />;
}

function BookModalForm({
  onClose,
  onSubmit,
  book,
  isSubmitting,
}: Omit<BookModalProps, "isOpen">) {
  const id = useId();
  const { data: publishers = [], isLoading: isLoadingPublishers } =
    useGetAllPublishers({
      query: { select: (response) => response.data },
    });
  const { data: shops = [] } = useGetAllShops({
    query: { select: (response) => response.data },
  });
  const [error, setError] = useState<string | null>(null);
  const [formData, setFormData] = useState<BookCreateDto>(() => ({
    title: book?.title ?? "",
    author: book?.author ?? "",
    price: book?.price ?? 0,
    publisher_id: book?.publisher.pub_id ?? "",
    format: book?.format || "Real",
    shop_id: book?.shop?.pub_id ?? null,
  }));

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (isSubmitting) return;
    if (!formData.publisher_id) {
      setError("出版社を選択してください。");
      return;
    }
    setError(null);
    try {
      await onSubmit(formData);
    } catch (error) {
      setError(`保存に失敗しました: ${getErrorMessage(error)}`);
    }
  };

  return (
    <Modal
      title={book ? "書籍情報を編集" : "新しい書籍を追加"}
      onClose={onClose}
      isSubmitting={isSubmitting}
    >
      <form onSubmit={handleSubmit} className="p-6">
        <fieldset disabled={isSubmitting} className="flex flex-col gap-4">
          <FormError message={error} />
          {!isLoadingPublishers && publishers.length === 0 && (
            <div className="p-3 bg-amber-50 dark:bg-amber-900/30 border border-amber-200 dark:border-amber-800 rounded text-amber-700 dark:text-amber-400 text-xs font-bold leading-normal">
              出版社が登録されていません。先に「出版社管理」から出版社を登録してください。
            </div>
          )}
          <div>
            <label
              htmlFor={`${id}-title`}
              className="block text-xs font-bold text-text-sub uppercase mb-1"
            >
              タイトル
            </label>
            <input
              id={`${id}-title`}
              required
              className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
              value={formData.title}
              onChange={(event) =>
                setFormData({ ...formData, title: event.target.value })
              }
            />
          </div>
          <div>
            <label
              htmlFor={`${id}-author`}
              className="block text-xs font-bold text-text-sub uppercase mb-1"
            >
              著者
            </label>
            <input
              id={`${id}-author`}
              required
              className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
              value={formData.author}
              onChange={(event) =>
                setFormData({ ...formData, author: event.target.value })
              }
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label
                htmlFor={`${id}-price`}
                className="block text-xs font-bold text-text-sub uppercase mb-1"
              >
                価格
              </label>
              <input
                id={`${id}-price`}
                type="number"
                required
                min={0}
                step={1}
                className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                value={formData.price}
                onChange={(event) =>
                  setFormData({
                    ...formData,
                    price: Number(event.target.value),
                  })
                }
              />
            </div>
            <div>
              <label
                htmlFor={`${id}-format`}
                className="block text-xs font-bold text-text-sub uppercase mb-1"
              >
                形式
              </label>
              <select
                id={`${id}-format`}
                className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                value={formData.format || "Real"}
                onChange={(event) =>
                  setFormData({ ...formData, format: event.target.value })
                }
              >
                <option value="Real">単行本/文庫</option>
                <option value="EBook">Kindle/電子書籍</option>
              </select>
            </div>
          </div>
          <div>
            <label
              htmlFor={`${id}-publisher`}
              className="block text-xs font-bold text-text-sub uppercase mb-1"
            >
              出版社
            </label>
            <select
              id={`${id}-publisher`}
              required
              className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
              value={formData.publisher_id}
              onChange={(event) =>
                setFormData({ ...formData, publisher_id: event.target.value })
              }
            >
              <option value="" disabled>
                出版社を選択してください
              </option>
              {publishers.map((publisher) => (
                <option key={publisher.pub_id} value={publisher.pub_id}>
                  {publisher.name}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label
              htmlFor={`${id}-shop`}
              className="block text-xs font-bold text-text-sub uppercase mb-1"
            >
              購入店舗 (任意)
            </label>
            <select
              id={`${id}-shop`}
              className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
              value={formData.shop_id || ""}
              onChange={(event) =>
                setFormData({
                  ...formData,
                  shop_id: event.target.value || null,
                })
              }
            >
              <option value="">未指定</option>
              {shops.map((shop) => (
                <option key={shop.pub_id} value={shop.pub_id}>
                  {shop.name}
                </option>
              ))}
            </select>
          </div>
          <FormActions
            onClose={onClose}
            isSubmitting={isSubmitting}
            isEditing={Boolean(book)}
          />
        </fieldset>
      </form>
    </Modal>
  );
}
