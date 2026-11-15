import { useGetAllPublishers } from "../../api/endpoints/publisher/publisher";
import type { PublisherResponseDto, PublisherCreateDto } from "../../api/model";
import PageHeader from "../../components/PageHeader";
import PageStatus from "../../components/PageStatus";
import { useEditor } from "../../hooks/useEditor";
import BudgetSummary from "../budget/BudgetSummary";
import PublisherModal from "./PublisherModal";
import PublisherTable from "./PublisherTable";
import { usePublisherMutations } from "./usePublisherMutations";

export default function Publishers() {
  const {
    data: publishers = [],
    isPending,
    error,
    refetch,
    isFetching,
  } = useGetAllPublishers({
    query: { select: (response) => response.data },
  });
  const editor = useEditor<PublisherResponseDto>();
  const { create, update, isSaving } = usePublisherMutations();

  const handleSubmit = async (data: PublisherCreateDto) => {
    if (editor.item) {
      await update.mutateAsync({ pubId: editor.item.pub_id, data });
    } else {
      await create.mutateAsync({ data });
    }
    editor.close();
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
            title="出版社管理"
            description="取引先出版社の登録と管理"
            createLabel="出版社を追加"
            onCreate={editor.openCreate}
          />
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
