import {
  afterEach,
  beforeEach,
  describe,
  expect,
  mock,
  spyOn,
  test,
} from "bun:test";
import { ApiError, customFetch } from "../../src/lib/custom-fetch";
import { createQueryClient } from "../../src/lib/query-client";

const originalDocument = Object.getOwnPropertyDescriptor(
  globalThis,
  "document",
);
const originalWindow = Object.getOwnPropertyDescriptor(globalThis, "window");
let redirect = mock((path: string) => path);

beforeEach(() => {
  redirect = mock((path: string) => path);
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    value: { cookie: "csrf_token=test-token" },
  });
  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: { location: { assign: redirect } },
  });
});

afterEach(() => {
  mock.restore();
  if (originalDocument)
    Object.defineProperty(globalThis, "document", originalDocument);
  else Reflect.deleteProperty(globalThis, "document");
  if (originalWindow)
    Object.defineProperty(globalThis, "window", originalWindow);
  else Reflect.deleteProperty(globalThis, "window");
});

describe("customFetch", () => {
  test("returns the generated client response envelope and preserves abort signals", async () => {
    const response = Response.json([{ pub_id: "book-1" }], {
      headers: { "X-Trace": "trace-1" },
    });
    const request = spyOn(globalThis, "fetch").mockResolvedValue(response);
    const signal = new AbortController().signal;
    const result = await customFetch<{
      data: unknown;
      status: number;
      headers: Headers;
    }>("/books", { signal });

    expect(result.data).toEqual([{ pub_id: "book-1" }]);
    expect(result.status).toBe(200);
    expect(result.headers.get("X-Trace")).toBe("trace-1");
    const [url, options] = request.mock.calls[0]!;
    expect(url).toBe("/api/books");
    expect(options?.signal).toBe(signal);
    expect(new Headers(options?.headers).has("X-CSRF-Token")).toBe(false);
  });

  test("sets one CSRF token without mutating the caller headers", async () => {
    document.cookie = "session=abc;csrf_token=new-token; extra=1";
    const request = spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(null, { status: 204 }),
    );
    const headers = new Headers({
      "X-CSRF-Token": "old-token",
      "X-Custom": "custom",
    });
    await customFetch("/books/book-1", { method: "patch", headers });

    const sent = new Headers(request.mock.calls[0]![1]?.headers);
    expect(sent.get("X-CSRF-Token")).toBe("new-token");
    expect(sent.get("X-Custom")).toBe("custom");
    expect(headers.get("X-CSRF-Token")).toBe("old-token");
  });

  test("handles 204 responses without attempting to read JSON", async () => {
    spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(null, { status: 204 }),
    );
    const result = await customFetch<{ data: void; status: number }>("/books", {
      method: "POST",
    });
    expect(result.status).toBe(204);
    expect(result.data).toBeUndefined();
  });

  test("exposes server validation messages and HTTP status", async () => {
    spyOn(globalThis, "fetch").mockResolvedValue(
      Response.json({ error: "Publisher not found" }, { status: 400 }),
    );
    await expect(
      customFetch("/books", { method: "POST" }),
    ).rejects.toMatchObject({
      name: "ApiError",
      status: 400,
      message: "Publisher not found",
    });
  });

  test("uses a readable fallback for non-JSON failures", async () => {
    spyOn(globalThis, "fetch").mockResolvedValue(
      new Response("Bad gateway", { status: 502 }),
    );
    await expect(customFetch("/books")).rejects.toMatchObject({
      status: 502,
      message: "通信に失敗しました (HTTP 502)",
    });
  });

  test("rejects and redirects on 401 instead of leaving a pending promise", async () => {
    spyOn(globalThis, "fetch").mockResolvedValue(
      new Response("Unauthorized", { status: 401 }),
    );
    await expect(customFetch("/books")).rejects.toBeInstanceOf(ApiError);
    expect(redirect).toHaveBeenCalledWith("/login");
  });

  test("preserves native request cancellation", async () => {
    const error = new DOMException("Aborted", "AbortError");
    spyOn(globalThis, "fetch").mockRejectedValue(error);
    await expect(customFetch("/books")).rejects.toBe(error);
  });

  test("does not retry client errors through the application query client", async () => {
    const request = spyOn(globalThis, "fetch").mockResolvedValue(
      Response.json({ error: "Not found" }, { status: 404 }),
    );
    const client = createQueryClient();
    try {
      await expect(
        client.fetchQuery({
          queryKey: ["/missing"],
          queryFn: () => customFetch("/missing"),
        }),
      ).rejects.toBeInstanceOf(ApiError);
      expect(request).toHaveBeenCalledTimes(1);
    } finally {
      client.clear();
    }
  });
});
