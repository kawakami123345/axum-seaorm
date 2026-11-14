import type { QueryClient } from "@tanstack/react-query";
import { getGetAllBooksQueryKey } from "../api/endpoints/book/book";
import { getGetAnnualSummaryQueryKey } from "../api/endpoints/dashboard/dashboard";
import { getGetAllPublishersQueryKey } from "../api/endpoints/publisher/publisher";

function invalidateResource(client: QueryClient, path: string) {
  return client.invalidateQueries({
    predicate: ({ queryKey }) => {
      const [key] = queryKey;
      return (
        typeof key === "string" && (key === path || key.startsWith(`${path}/`))
      );
    },
  });
}

export async function invalidateBookViews(client: QueryClient) {
  await Promise.all([
    invalidateResource(client, getGetAllBooksQueryKey()[0]),
    client.invalidateQueries({ queryKey: getGetAnnualSummaryQueryKey() }),
  ]);
}

export async function invalidatePublisherViews(client: QueryClient) {
  await Promise.all([
    invalidateResource(client, getGetAllPublishersQueryKey()[0]),
    invalidateResource(client, getGetAllBooksQueryKey()[0]),
  ]);
}
