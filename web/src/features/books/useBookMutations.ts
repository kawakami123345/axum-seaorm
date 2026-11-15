import { useQueryClient } from "@tanstack/react-query";
import {
  useChangeBookAppliedAt,
  useCreateBook,
  useUpdateBook,
} from "../../api/endpoints/book/book";
import { invalidateBookViews } from "../../lib/query-cache";
import { ApiError } from "../../lib/custom-fetch";

export function useBookMutations() {
  const queryClient = useQueryClient();
  const onSuccess = () => invalidateBookViews(queryClient);
  const onError = (error: Error) => {
    if (error instanceof ApiError && error.status === 409) return onSuccess();
  };
  const create = useCreateBook<Error>({ mutation: { onSuccess } });
  const update = useUpdateBook<Error>({ mutation: { onSuccess, onError } });
  const changeAppliedAt = useChangeBookAppliedAt<Error>({
    mutation: { onSuccess, onError },
  });

  return {
    create,
    update,
    changeAppliedAt,
    isSaving: create.isPending || update.isPending,
  };
}
