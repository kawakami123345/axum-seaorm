import { useGetAllBooks } from "../../api/endpoints/book/book";
import type { BookResponseDto, BookCreateDto } from "../../api/model";
import PageHeader from "../../components/PageHeader";
import PageStatus from "../../components/PageStatus";
import { useEditor } from "../../hooks/useEditor";
import { getErrorMessage } from "../../lib/custom-fetch";
import { parseAppliedDate } from "../../lib/format";
import BudgetSummary from "../budget/BudgetSummary";
import BookModal from "./BookModal";
import BookTable from "./BookTable";
import { useBookMutations } from "./useBookMutations";

export default function Library() {
  const {
    data: books = [],
    isPending,
    error,
    refetch,
    isFetching,
  } = useGetAllBooks({
    query: { select: (response) => response.data },
  });
  const editor = useEditor<BookResponseDto>();
  const { create, update, changeAppliedAt, isSaving } = useBookMutations();

  const handleSubmit = async (data: BookCreateDto) => {
    if (editor.item) {
      await update.mutateAsync({ pubId: editor.item.pub_id, data });
    } else {
      await create.mutateAsync({ data });
    }
    editor.close();
  };

  const handleApply = (book: BookResponseDto) => {
    const input = window.prompt(
      `「${book.title}」の経費申請を行います。\n申請日を入力してください (YYYY-MM-DD):`,
      new Date().toISOString().slice(0, 10),
    );
    if (input === null) return;
    const appliedAt = parseAppliedDate(input);
    if (!appliedAt) {
      window.alert("無効な日付形式です。YYYY-MM-DD形式で入力してください。");
      return;
    }
    changeAppliedAt.mutate(
      { pubId: book.pub_id, data: { applied_at: appliedAt } },
      {
        onError: (error) =>
          window.alert(`申請に失敗しました: ${getErrorMessage(error)}`),
      },
    );
  };

  if (isPending || error) {
    return (
      <PageStatus
        status={isPending ? "loading" : "error"}
        onRetry={() => void refetch()}
        isRetrying={isFetching}
      />
    );
  }

  return (
    <>
      <main className="flex-1 min-w-0">
        <div className="flex flex-col gap-6">
          <PageHeader
            title="購入履歴"
            description="コレクションの管理と経費の追跡"
            createLabel="書籍を追加"
            onCreate={editor.openCreate}
          />
          <div className="flex gap-4 border-b border-border-color dark:border-gray-700">
            <button
              type="button"
              className="px-1 py-3 border-b-2 border-primary text-primary text-sm font-bold"
            >
              すべての書籍
            </button>
            <button
              type="button"
              className="px-1 py-3 border-b-2 border-transparent text-text-sub hover:text-text-main hover:border-gray-300 text-sm font-medium transition-colors"
            >
              お気に入り
            </button>
            <button
              type="button"
              className="px-1 py-3 border-b-2 border-transparent text-text-sub hover:text-text-main hover:border-gray-300 text-sm font-medium transition-colors"
            >
              高額書籍
            </button>
          </div>
          <BookTable
            books={books}
            onEdit={editor.openEdit}
            onApply={handleApply}
            isApplying={changeAppliedAt.isPending}
          />
        </div>
      </main>
      <BudgetSummary />
      <BookModal
        isOpen={editor.isOpen}
        onClose={editor.close}
        onSubmit={handleSubmit}
        book={editor.item}
        isSubmitting={isSaving}
      />
    </>
  );
}
