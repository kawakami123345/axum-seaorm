import type { PublisherResponseDto } from "../../api/model";
import DataTable, { type TableColumn } from "../../components/DataTable";

const columns: readonly TableColumn[] = [
  { label: "出版社名", className: "w-[70%]" },
  { label: "ID (短縮)", className: "w-[30%] text-right" },
];

interface PublisherTableProps {
  publishers: readonly PublisherResponseDto[];
  onEdit: (publisher: PublisherResponseDto) => void;
}

export default function PublisherTable({
  publishers,
  onEdit,
}: PublisherTableProps) {
  return (
    <DataTable
      caption="出版社一覧"
      columns={columns}
      count={publishers.length}
      emptyMessage="出版社が登録されていません。右上のボタンから追加してください。"
      footer={
        <p className="text-xs text-text-sub dark:text-gray-400">
          計{" "}
          <span className="font-bold text-text-main dark:text-white">
            {publishers.length}
          </span>{" "}
          件の出版社
        </p>
      }
    >
      {publishers.map((publisher) => (
        <tr
          key={publisher.pub_id}
          onClick={() => onEdit(publisher)}
          className="cursor-pointer transition-all group hover:bg-gray-50 dark:hover:bg-gray-800/50"
        >
          <td className="px-6 py-4 text-sm font-bold text-text-main dark:text-white group-hover:text-primary transition-colors">
            <button
              type="button"
              aria-label={`出版社「${publisher.name}」を編集`}
              onClick={(event) => {
                event.stopPropagation();
                onEdit(publisher);
              }}
              className="text-left focus-visible:outline-2 focus-visible:outline-primary focus-visible:outline-offset-4 rounded-sm"
            >
              {publisher.name}
            </button>
          </td>
          <td className="px-6 py-4 text-xs font-mono text-text-sub dark:text-gray-400 text-right">
            {publisher.pub_id.slice(0, 8)}...
          </td>
        </tr>
      ))}
    </DataTable>
  );
}
