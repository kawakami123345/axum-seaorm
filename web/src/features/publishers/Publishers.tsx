import { useGetAllPublishers } from "../../api/endpoints/publisher/publisher";
import type { PublisherResponseDto, PublisherCreateDto } from "../../api/model";
import PageHeader from "../../components/PageHeader";
import PageStatus from "../../components/PageStatus";
import RequestStatus from "../../components/RequestStatus";
import { useEditor } from "../../hooks/useEditor";
import BudgetSummary from "../budget/BudgetSummary";
import PublisherModal from "./PublisherModal";
import PublisherTable from "./PublisherTable";
import { usePublisherMutations } from "./usePublisherMutations";

export default function Publishers() {
  const {
    data: publishers,
    isPending,
    isRefetchError,
    refetch,
    isFetching,
  } = useGetAllPublishers({
    query: { select: (response) => response.data },
  });
  const editor = useEditor<PublisherResponseDto>();
  const { create, update, isSaving } = usePublisherMutations();

  const handleSubmit = async (data: PublisherCreateDto) => {
    if (editor.item) {
      await update.mutateAsync({
        pubId: editor.item.pub_id,
        data: { ...data, revision: editor.item.revision },
      });
    } else {
      await create.mutateAsync({ data });
    }
    editor.close();
  };

  if (publishers === undefined) {
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
            title="出版社管理"
            description="取引先出版社の登録と管理"
            createLabel="出版社を追加"
            onCreate={editor.openCreate}
          />
          {isRefetchError && (
            <RequestStatus
              status="error"
              message="出版社一覧を更新できませんでした。前回の一覧を表示しています。"
              onRetry={() => void refetch()}
              isRetrying={isFetching}
            />
          )}
          <PublisherTable publishers={publishers} onEdit={editor.openEdit} />
        </div>
      </main>
      <BudgetSummary />
      <PublisherModal
        isOpen={editor.isOpen}
        onClose={editor.close}
        onSubmit={handleSubmit}
        publisher={editor.item}
        isSubmitting={isSaving}
      />
    </>
  );
}
