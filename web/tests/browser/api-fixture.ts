import type { Page } from "@playwright/test";
import type {
  BookCreateDto,
  BookUpdateDto,
  BookChangeAppliedAtDto,
  BookResponseDto,
  PublisherResponseDto,
  PublisherUpdateDto,
} from "../../src/api/model";

export async function mockApi(page: Page) {
  const publisher: PublisherResponseDto = {
    pub_id: "11111111-1111-4111-8111-111111111111",
    name: "Test Publisher",
    revision: 0,
  };
  const shop = {
    pub_id: "22222222-2222-4222-8222-222222222222",
    name: "Test Shop",
    revision: 0,
  };
  const state = {
    books: [
      {
        pub_id: "33333333-3333-4333-8333-333333333333",
        title: "Original Book",
        author: "Original Author",
        price: 1000,
        publisher,
        shop: null,
        format: "Real",
        applied_at: null,
        applied_without_date: false,
        revision: 0,
      },
    ] as BookResponseDto[],
    publishers: [publisher],
    bookWrites: [] as { data: BookCreateDto; csrfToken?: string }[],
    bookUpdates: [] as BookUpdateDto[],
    publisherUpdates: [] as PublisherUpdateDto[],
    appliedWrites: [] as BookChangeAppliedAtDto[],
    holdBookSave: undefined as Promise<void> | undefined,
    failPublishers: false,
    failSummary: false,
    holdSummary: undefined as Promise<void> | undefined,
    failBooks: false,
    failShops: false,
    failNextBookUpdate: false,
  };

  await page.route("https://**/*", (route) => route.abort());
  // Vite also serves source modules from /src/api/, which must reach the dev server.
  await page.route(
    (url) => url.pathname.startsWith("/api/"),
    async (route) => {
      const request = route.request();
      const path = new URL(request.url()).pathname.slice("/api".length);
      const method = request.method();

      if (path === "/books" && method === "GET") {
        if (state.failBooks)
          return route.fulfill({
            status: 500,
            json: { error: "Internal Server Error" },
          });
        return route.fulfill({ json: state.books });
      }
      if (path === "/publishers" && method === "GET") {
        if (state.failPublishers) return route.fulfill({ status: 500 });
        return route.fulfill({ json: state.publishers });
      }
      if (path === "/shops" && method === "GET") {
        if (state.failShops)
          return route.fulfill({
            status: 500,
            json: { error: "Internal Server Error" },
          });
        return route.fulfill({ json: [shop] });
      }
      if (path === "/dashboard/annual-summary") {
        await state.holdSummary;
        if (state.failSummary) {
          return route.fulfill({
            status: 500,
            json: { error: "Internal Server Error" },
          });
        }
        const applied = state.books.filter((book) => book.applied_at);
        const total = applied.reduce((sum, book) => sum + book.price, 0);
        return route.fulfill({
          json: applied.length
            ? [
                {
                  year: new Date(applied[0]!.applied_at!).getUTCFullYear(),
                  limit: 20000,
                  total_amount: total,
                  balance: 20000 - total,
                  count: applied.length,
                  average: total / applied.length,
                },
              ]
            : [],
        });
      }
      if (path === "/books" && method === "POST") {
        const data = request.postDataJSON() as BookCreateDto;
        state.bookWrites.push({
          data,
          csrfToken: request.headers()["x-csrf-token"],
        });
        await state.holdBookSave;
        state.books.push({
          pub_id: `44444444-4444-4444-8444-${String(state.books.length).padStart(12, "0")}`,
          title: data.title,
          author: data.author,
          price: data.price,
          publisher: state.publishers.find(
            (item) => item.pub_id === data.publisher_id,
          )!,
          shop: data.shop_id ? shop : null,
          format: data.format ?? "Real",
          applied_at: null,
          applied_without_date: false,
          revision: 0,
        });
        return route.fulfill({ status: 204 });
      }
      if (path.startsWith("/books/") && path.endsWith("/applied_at")) {
        const data = request.postDataJSON() as BookChangeAppliedAtDto;
        state.appliedWrites.push(data);
        const book = state.books.find(
          (item) => item.pub_id === path.split("/")[2],
        )!;
        if (data.revision !== book.revision) {
          return route.fulfill({
            status: 409,
            json: {
              error: "別の操作で更新されました。再読み込みしてください。",
            },
          });
        }
        book.applied_at = data.applied_at ?? null;
        book.applied_without_date = false;
        book.revision += 1;
        return route.fulfill({ json: book });
      }
      if (path.startsWith("/books/") && method === "PUT") {
        const data = request.postDataJSON() as BookUpdateDto;
        state.bookUpdates.push(data);
        if (state.failNextBookUpdate) {
          state.failNextBookUpdate = false;
          return route.fulfill({
            status: 409,
            json: {
              error:
                "別の操作で書籍が更新されました。再読み込みしてからやり直してください。",
            },
          });
        }
        const book = state.books.find(
          (item) => item.pub_id === path.split("/")[2],
        )!;
        if (data.revision !== book.revision) {
          return route.fulfill({
            status: 409,
            json: {
              error: "別の操作で更新されました。再読み込みしてください。",
            },
          });
        }
        Object.assign(book, {
          title: data.title,
          author: data.author,
          price: data.price,
          format: data.format,
          publisher: state.publishers.find(
            (item) => item.pub_id === data.publisher_id,
          ),
          shop: data.shop_id ? shop : null,
          revision: book.revision + 1,
        });
        return route.fulfill({ status: 204 });
      }
      if (path === "/publishers" && method === "POST") {
        const { name } = request.postDataJSON() as { name: string };
        if (state.publishers.some((item) => item.name === name)) {
          return route.fulfill({
            status: 400,
            json: { error: "同じ名前の出版社がすでに登録されています。" },
          });
        }
        state.publishers.push({
          pub_id: "55555555-5555-4555-8555-555555555555",
          name,
          revision: 0,
        });
        return route.fulfill({ status: 204 });
      }
      if (path.startsWith("/publishers/") && method === "PUT") {
        const data = request.postDataJSON() as PublisherUpdateDto;
        state.publisherUpdates.push(data);
        const publisher = state.publishers.find(
          (item) => item.pub_id === path.split("/")[2],
        )!;
        if (data.revision !== publisher.revision) {
          return route.fulfill({
            status: 409,
            json: {
              error: "別の操作で更新されました。再読み込みしてください。",
            },
          });
        }
        publisher.name = data.name;
        publisher.revision += 1;
        return route.fulfill({ status: 204 });
      }
      return route.fulfill({
        status: 404,
        json: { error: `Unhandled test endpoint: ${method} ${path}` },
      });
    },
  );
  return state;
}
