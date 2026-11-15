import { useQueryClient } from "@tanstack/react-query";
import {
  useCreatePublisher,
  useUpdatePublisher,
} from "../../api/endpoints/publisher/publisher";
import { invalidatePublisherViews } from "../../lib/query-cache";
import { ApiError } from "../../lib/custom-fetch";

export function usePublisherMutations() {
  const queryClient = useQueryClient();
  const onSuccess = () => invalidatePublisherViews(queryClient);
  const onError = (error: Error) => {
    if (error instanceof ApiError && error.status === 409) return onSuccess();
  };
  const create = useCreatePublisher<Error>({ mutation: { onSuccess } });
  const update = useUpdatePublisher<Error>({
    mutation: { onSuccess, onError },
  });

  return { create, update, isSaving: create.isPending || update.isPending };
}
