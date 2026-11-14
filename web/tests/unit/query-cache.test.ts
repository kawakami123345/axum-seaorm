import { expect, test } from "bun:test";
import { QueryClient } from "@tanstack/react-query";
import { getGetAllBooksQueryKey } from "../../src/api/endpoints/book/book";
import { getGetAnnualSummaryQueryKey } from "../../src/api/endpoints/dashboard/dashboard";
import { getGetAllPublishersQueryKey } from "../../src/api/endpoints/publisher/publisher";
import {
  invalidateBookViews,
  invalidatePublisherViews,
} from "../../src/lib/query-cache";

const keys = {
  books: getGetAllBooksQueryKey(),
  book: ["/books/book-1"],
  year: ["/books/applied/2024"],
  summary: getGetAnnualSummaryQueryKey(),
  publishers: getGetAllPublishersQueryKey(),
  publisher: ["/publishers/publisher-1"],
  unrelated: ["/bookshelf"],
};

function createCache() {
  const client = new QueryClient();
  for (const key of Object.values(keys)) client.setQueryData(key, []);
  return client;
}

test("book changes refresh lists, details, annual lists and summaries", async () => {
  const client = createCache();
  try {
    await invalidateBookViews(client);
    for (const key of [keys.books, keys.book, keys.year, keys.summary]) {
      expect(client.getQueryState(key)?.isInvalidated).toBe(true);
    }
    expect(client.getQueryState(keys.publishers)?.isInvalidated).toBe(false);
    expect(client.getQueryState(keys.unrelated)?.isInvalidated).toBe(false);
  } finally {
    client.clear();
  }
});

test("publisher changes refresh nested book data while preserving unrelated cache", async () => {
  const client = createCache();
  try {
    await invalidatePublisherViews(client);
    for (const key of [
      keys.publishers,
      keys.publisher,
      keys.books,
      keys.book,
      keys.year,
    ]) {
      expect(client.getQueryState(key)?.isInvalidated).toBe(true);
    }
    expect(client.getQueryState(keys.summary)?.isInvalidated).toBe(false);
    expect(client.getQueryState(keys.unrelated)?.isInvalidated).toBe(false);
  } finally {
    client.clear();
  }
});
