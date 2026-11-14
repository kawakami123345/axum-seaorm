import type { MouseEvent } from "react";
import { useGetAllBooks } from "../api/endpoints/book/book";
import type { BookResponseDto, BookCreateDto } from "../api/model";
import BudgetSummary from "../components/BudgetSummary";
import BookModal from "../components/BookModal";
import PageStatus from "../components/PageStatus";
import { useBookMutations } from "../hooks/useBookMutations";
import { useEditor } from "../hooks/useEditor";
import { getErrorMessage } from "../lib/custom-fetch";
import { formatDate, formatYen, parseAppliedDate } from "../lib/format";

const Library = () => {
  const {
    data: books = [],
    isLoading,
    error,
  } = useGetAllBooks({
    query: { select: (response) => response.data },
  });
  const editor = useEditor<BookResponseDto>();
  const { create, update, changeAppliedAt, isSaving } = useBookMutations();

  const handleModalSubmit = async (data: BookCreateDto) => {
    if (editor.item) {
      await update.mutateAsync({ pubId: editor.item.pub_id, data });
    } else {
      await create.mutateAsync({ data });
    }
    editor.close();
  };

  const handleApply = (
    book: BookResponseDto,
    event: MouseEvent<HTMLButtonElement>,
  ) => {
    event.stopPropagation();
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

  if (isLoading || error)
    return <PageStatus status={isLoading ? "loading" : "error"} />;

  return (
    <>
      <main className="flex-1 min-w-0">
        <div className="flex flex-col gap-6">
          <div className="flex flex-wrap items-end justify-between gap-4 border-b border-border-color pb-4">
            <div>
              <h1 className="text-2xl font-bold text-text-main dark:text-white leading-tight">
                購入履歴
              </h1>
              <p className="text-text-sub dark:text-gray-400 text-sm mt-1 text-[13px]">
                コレクションの管理と経費の追跡
              </p>
            </div>
            <button
              onClick={editor.openCreate}
              className="flex items-center justify-center gap-2 bg-primary hover:bg-primary-dark text-white text-[13px] font-bold px-5 py-2.5 rounded-md shadow-sm transition-all hover:scale-[1.02] active:scale-95 focus:ring-2 focus:ring-offset-2 focus:ring-primary"
            >
              <span
                aria-hidden="true"
                className="material-symbols-outlined text-lg"
              >
                add
              </span>
              <span>書籍を追加</span>
            </button>
          </div>

          <div className="flex gap-4 border-b border-border-color dark:border-gray-700">
            <button className="px-1 py-3 border-b-2 border-primary text-primary text-sm font-bold">
              すべての書籍
            </button>
            <button className="px-1 py-3 border-b-2 border-transparent text-text-sub hover:text-text-main hover:border-gray-300 text-sm font-medium transition-colors">
              お気に入り
            </button>
            <button className="px-1 py-3 border-b-2 border-transparent text-text-sub hover:text-text-main hover:border-gray-300 text-sm font-medium transition-colors">
              高額書籍
            </button>
          </div>

          <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 shadow-sm overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="bg-gray-100 dark:bg-gray-800 border-b border-border-color dark:border-gray-700">
                    <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[35%]">
                      書籍タイトル
                    </th>
                    <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[20%] hidden sm:table-cell">
                      著者
                    </th>
                    <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[15%] hidden md:table-cell">
                      購入日
                    </th>
                    <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[15%] text-center">
                      経費申請
                    </th>
                    <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[15%] text-right">
                      価格
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border-color dark:divide-gray-700">
                  {books.map((book) => {
                    const isPending = !book.applied_at;
                    return (
                      <tr
                        key={book.pub_id}
                        onClick={() => editor.openEdit(book)}
                        className={`cursor-pointer transition-all ${
                          isPending
                            ? "row-highlight-pending hover:bg-amber-100 dark:hover:bg-amber-900/30"
                            : "group hover:bg-gray-50 dark:hover:bg-gray-800/50"
                        }`}
                      >
                        <td className="px-6 py-4">
                          <div className="flex flex-col">
                            <div className="flex items-center gap-2">
                              {isPending && (
                                <span
                                  aria-hidden="true"
                                  className="material-symbols-outlined text-accent-yellow text-sm font-bold"
                                >
                                  warning
                                </span>
                              )}
                              <p className="text-text-main dark:text-white text-sm font-bold leading-normal group-hover:text-primary transition-colors">
                                {book.title}
                              </p>
                            </div>
                            <p className="text-text-sub dark:text-gray-400 text-xs sm:hidden mt-1">
                              {book.author}
                            </p>
                          </div>
                        </td>
                        <td className="px-6 py-4 text-sm text-text-sub dark:text-gray-300 hidden sm:table-cell">
                          {book.author}
                        </td>
                        <td className="px-6 py-4 text-sm text-text-sub dark:text-gray-400 hidden md:table-cell">
                          {book.applied_at ? formatDate(book.applied_at) : "-"}
                        </td>
                        <td className="px-6 py-4 text-center">
                          <div className="flex items-center justify-center gap-2">
                            {!isPending ? (
                              <>
                                <span
                                  aria-hidden="true"
                                  className="material-symbols-outlined text-xl text-primary"
                                >
                                  check_box
                                </span>
                                <span className="text-xs font-bold text-text-main dark:text-gray-300 font-medium">
                                  申請済
                                </span>
                              </>
                            ) : (
                              <button
                                onClick={(e) => handleApply(book, e)}
                                className="flex items-center gap-1 bg-white border border-primary text-primary px-3 py-1 rounded text-xs font-bold hover:bg-primary hover:text-white transition-colors shadow-sm disabled:opacity-50 disabled:cursor-not-allowed"
                                disabled={changeAppliedAt.isPending}
                              >
                                <span
                                  aria-hidden="true"
                                  className="material-symbols-outlined text-base"
                                >
                                  send
                                </span>
                                申請
                              </button>
                            )}
                          </div>
                        </td>
                        <td className="px-6 py-4 text-sm font-bold text-text-main dark:text-white text-right">
                          {formatYen(book.price)}
                        </td>
                      </tr>
                    );
                  })}
                  {books.length === 0 && (
                    <tr>
                      <td
                        colSpan={5}
                        className="px-6 py-10 text-center text-text-sub"
                      >
                        書籍が登録されていません。右上のボタンから追加してください。
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
            <div className="flex items-center justify-between px-6 py-4 border-t border-border-color dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
              <p className="text-xs text-text-sub dark:text-gray-400">
                表示中:{" "}
                <span className="font-bold text-text-main dark:text-white">
                  {books.length === 0 ? 0 : 1}
                </span>{" "}
                -{" "}
                <span className="font-bold text-text-main dark:text-white">
                  {books.length}
                </span>{" "}
                /{" "}
                <span className="font-bold text-text-main dark:text-white">
                  {books.length}
                </span>{" "}
                件
              </p>
              <div className="flex gap-1">
                <button
                  className="flex items-center justify-center size-8 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors disabled:opacity-50 text-text-main dark:text-white"
                  disabled
                >
                  <span
                    aria-hidden="true"
                    className="material-symbols-outlined text-lg"
                  >
                    chevron_left
                  </span>
                </button>
                <button className="flex items-center justify-center size-8 rounded bg-primary text-white text-sm font-bold shadow-sm">
                  1
                </button>
                <button className="flex items-center justify-center size-8 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-text-main dark:text-white">
                  <span
                    aria-hidden="true"
                    className="material-symbols-outlined text-lg"
                  >
                    chevron_right
                  </span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </main>
      <BudgetSummary />

      <BookModal
        isOpen={editor.isOpen}
        onClose={editor.close}
        onSubmit={handleModalSubmit}
        book={editor.item}
        isSubmitting={isSaving}
      />
    </>
  );
};

export default Library;
