import { useQueryClient } from "@tanstack/react-query";
import {
  useChangeBookAppliedAt,
  useCreateBook,
  useUpdateBook,
} from "../../api/endpoints/book/book";
import { invalidateBookViews } from "../../lib/query-cache";

export function useBookMutations() {
  const queryClient = useQueryClient();
  const onSuccess = () => invalidateBookViews(queryClient);
  const create = useCreateBook<Error>({ mutation: { onSuccess } });
  const update = useUpdateBook<Error>({ mutation: { onSuccess } });
  const changeAppliedAt = useChangeBookAppliedAt<Error>({
    mutation: { onSuccess },
  });

  return {
    create,
    update,
    changeAppliedAt,
    isSaving: create.isPending || update.isPending,
  };
}
