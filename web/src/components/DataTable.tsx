import type { ReactNode } from "react";

export interface TableColumn {
  label: string;
  className?: string;
}

interface DataTableProps {
  caption: string;
  columns: readonly TableColumn[];
  count: number;
  emptyMessage: string;
  children: ReactNode;
  footer?: ReactNode;
}

export default function DataTable({
  caption,
  columns,
  count,
  emptyMessage,
  children,
  footer,
}: DataTableProps) {
  return (
    <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 shadow-sm overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <caption className="sr-only">{caption}</caption>
          <thead>
            <tr className="bg-gray-100 dark:bg-gray-800 border-b border-border-color dark:border-gray-700">
              {columns.map(({ label, className = "" }) => (
                <th
                  key={label}
                  scope="col"
                  className={`px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide ${className}`}
                >
                  {label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-border-color dark:divide-gray-700">
            {count > 0 ? (
              children
            ) : (
              <tr>
                <td
                  colSpan={columns.length}
                  className="px-6 py-10 text-center text-text-sub"
                >
                  {emptyMessage}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      {footer && (
        <div className="flex items-center justify-between px-6 py-4 border-t border-border-color dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
          {footer}
        </div>
      )}
    </div>
  );
}
