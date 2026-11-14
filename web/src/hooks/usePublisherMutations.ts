import { useQueryClient } from "@tanstack/react-query";
import {
  useCreatePublisher,
  useUpdatePublisher,
} from "../api/endpoints/publisher/publisher";
import { invalidatePublisherViews } from "../lib/query-cache";

export function usePublisherMutations() {
  const queryClient = useQueryClient();
  const onSuccess = () => invalidatePublisherViews(queryClient);
  const create = useCreatePublisher<Error>({ mutation: { onSuccess } });
  const update = useUpdatePublisher<Error>({ mutation: { onSuccess } });

  return { create, update, isSaving: create.isPending || update.isPending };
}
