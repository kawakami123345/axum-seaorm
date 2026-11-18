import type { BookResponseDto } from "../../api/model";
import DataTable, { type TableColumn } from "../../components/DataTable";
import Icon from "../../components/Icon";
import { formatDate, formatYen } from "../../lib/format";

const columns: readonly TableColumn[] = [
  { label: "書籍タイトル", className: "w-[35%]" },
  { label: "著者", className: "w-[20%] hidden sm:table-cell" },
  { label: "申請日", className: "w-[15%] hidden md:table-cell" },
  { label: "経費申請", className: "w-[15%] text-center" },
  { label: "価格", className: "w-[15%] text-right" },
];

interface BookTableProps {
  books: readonly BookResponseDto[];
  onSelect: (book: BookResponseDto) => void;
  onApply: (book: BookResponseDto) => void;
  isApplying: boolean;
}

export default function BookTable({
  books,
  onSelect,
  onApply,
  isApplying,
}: BookTableProps) {
  return (
    <DataTable
      caption="書籍一覧"
      columns={columns}
      count={books.length}
      emptyMessage="書籍が登録されていません。右上のボタンから追加してください。"
      footer={<BookTableFooter count={books.length} />}
    >
      {books.map((book) => (
        <BookRow
          key={book.pub_id}
          book={book}
          onSelect={onSelect}
          onApply={onApply}
          isApplying={isApplying}
        />
      ))}
    </DataTable>
  );
}

function BookRow({
  book,
  onSelect,
  onApply,
  isApplying,
}: Omit<BookTableProps, "books"> & { book: BookResponseDto }) {
  const isPending = !book.applied_at && !book.applied_without_date;
  return (
    <tr
      onClick={() => onSelect(book)}
      className={`group cursor-pointer transition-all ${isPending ? "row-highlight-pending hover:bg-amber-100 dark:hover:bg-amber-900/30" : "hover:bg-gray-50 dark:hover:bg-gray-800/50"}`}
    >
      <td className="px-6 py-4">
        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            {isPending && (
              <Icon
                name="warning"
                className="text-accent-yellow text-sm font-bold"
              />
            )}
            <button
              type="button"
              aria-label={`書籍「${book.title}」${isPending ? "を編集" : "の詳細を表示"}`}
              onClick={(event) => {
                event.stopPropagation();
                onSelect(book);
              }}
              className="text-left text-text-main dark:text-white text-sm font-bold leading-normal group-hover:text-primary focus-visible:outline-2 focus-visible:outline-primary focus-visible:outline-offset-4 rounded-sm"
            >
              {book.title}
            </button>
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
        {book.applied_at
          ? formatDate(book.applied_at)
          : book.applied_without_date
            ? "日付不明"
            : "-"}
      </td>
      <td className="px-6 py-4 text-center">
        <div className="flex items-center justify-center gap-2">
          {isPending ? (
            <button
              type="button"
              onClick={(event) => {
                event.stopPropagation();
                onApply(book);
              }}
              disabled={isApplying}
              className="flex items-center gap-1 bg-white border border-primary text-primary px-3 py-1 rounded text-xs font-bold hover:bg-primary hover:text-white transition-colors shadow-sm disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Icon name="send" className="text-base" />
              申請
            </button>
          ) : (
            <>
              <Icon name="check_box" className="text-xl text-primary" />
              <span className="text-xs font-bold text-text-main dark:text-gray-300">
                申請済
              </span>
              <span className="text-xs text-text-sub">閲覧のみ</span>
            </>
          )}
        </div>
      </td>
      <td className="px-6 py-4 text-sm font-bold text-text-main dark:text-white text-right">
        {formatYen(book.price)}
      </td>
    </tr>
  );
}

function BookTableFooter({ count }: { count: number }) {
  return (
    <>
      <p className="text-xs text-text-sub dark:text-gray-400">
        表示中:{" "}
        <span className="font-bold text-text-main dark:text-white">
          {count === 0 ? 0 : 1}
        </span>
        {" - "}
        <span className="font-bold text-text-main dark:text-white">
          {count}
        </span>
        {" / "}
        <span className="font-bold text-text-main dark:text-white">
          {count}
        </span>{" "}
        件
      </p>
      <div className="flex gap-1">
        <button
          type="button"
          aria-label="前のページ"
          className="flex items-center justify-center size-8 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors disabled:opacity-50 text-text-main dark:text-white"
          disabled
        >
          <Icon name="chevron_left" className="text-lg" />
        </button>
        <button
          type="button"
          aria-current="page"
          className="flex items-center justify-center size-8 rounded bg-primary text-white text-sm font-bold shadow-sm"
        >
          1
        </button>
        <button
          type="button"
          aria-label="次のページ"
          className="flex items-center justify-center size-8 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-text-main dark:text-white"
        >
          <Icon name="chevron_right" className="text-lg" />
        </button>
      </div>
    </>
  );
}
