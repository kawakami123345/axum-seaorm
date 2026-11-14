import { useGetAllPublishers } from "../api/endpoints/publisher/publisher";
import type { PublisherResponseDto, PublisherCreateDto } from "../api/model";
import BudgetSummary from "../components/BudgetSummary";
import PublisherModal from "../components/PublisherModal";
import PageStatus from "../components/PageStatus";
import { useEditor } from "../hooks/useEditor";
import { usePublisherMutations } from "../hooks/usePublisherMutations";

const Publishers = () => {
  const {
    data: publishers = [],
    isLoading,
    error,
  } = useGetAllPublishers({
    query: { select: (response) => response.data },
  });
  const editor = useEditor<PublisherResponseDto>();
  const { create, update, isSaving } = usePublisherMutations();

  const handleModalSubmit = async (data: PublisherCreateDto) => {
    if (editor.item) {
      await update.mutateAsync({ pubId: editor.item.pub_id, data });
    } else {
      await create.mutateAsync({ data });
    }
    editor.close();
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
                出版社管理
              </h1>
              <p className="text-text-sub dark:text-gray-400 text-sm mt-1 text-[13px]">
                取引先出版社の登録と管理
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
              <span>出版社を追加</span>
            </button>
          </div>

          <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 shadow-sm overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="bg-gray-100 dark:bg-gray-800 border-b border-border-color dark:border-gray-700">
                    <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[70%]">
                      出版社名
                    </th>
                    <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[30%] text-right">
                      ID (短縮)
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border-color dark:divide-gray-700">
                  {publishers.map((publisher) => {
                    return (
                      <tr
                        key={publisher.pub_id}
                        onClick={() => editor.openEdit(publisher)}
                        className="cursor-pointer transition-all group hover:bg-gray-50 dark:hover:bg-gray-800/50"
                      >
                        <td className="px-6 py-4 text-sm font-bold text-text-main dark:text-white group-hover:text-primary transition-colors">
                          {publisher.name}
                        </td>
                        <td className="px-6 py-4 text-xs font-mono text-text-sub dark:text-gray-400 text-right">
                          {publisher.pub_id.slice(0, 8)}...
                        </td>
                      </tr>
                    );
                  })}
                  {publishers.length === 0 && (
                    <tr>
                      <td
                        colSpan={2}
                        className="px-6 py-10 text-center text-text-sub"
                      >
                        出版社が登録されていません。右上のボタンから追加してください。
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
            <div className="flex items-center justify-between px-6 py-4 border-t border-border-color dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
              <p className="text-xs text-text-sub dark:text-gray-400">
                計{" "}
                <span className="font-bold text-text-main dark:text-white">
                  {publishers.length}
                </span>{" "}
                件の出版社
              </p>
            </div>
          </div>
        </div>
      </main>
      <BudgetSummary />

      <PublisherModal
        isOpen={editor.isOpen}
        onClose={editor.close}
        onSubmit={handleModalSubmit}
        publisher={editor.item}
        isSubmitting={isSaving}
      />
    </>
  );
};

export default Publishers;
