import { expect, test, type Dialog } from "@playwright/test";
import { mockApi } from "./api-fixture";

test("creates the first publisher and book through mobile navigation", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const state = await mockApi(page);
  state.publishers = [];
  state.books = [];
  await page.goto("/library");
  await page.getByRole("link", { name: "出版社", exact: true }).click();
  await page.getByRole("button", { name: "出版社を追加" }).click();
  const publisherDialog = page.getByRole("dialog");
  await publisherDialog
    .getByLabel("出版社名", { exact: true })
    .fill("First Publisher");
  await publisherDialog.getByRole("button", { name: "保存する" }).click();
  await expect(publisherDialog).not.toBeVisible();

  await page.getByRole("link", { name: "ライブラリ", exact: true }).click();
  await page.getByRole("button", { name: "書籍を追加" }).click();
  const bookDialog = page.getByRole("dialog");
  await bookDialog.getByLabel("タイトル", { exact: true }).fill("First Book");
  await bookDialog.getByLabel("著者", { exact: true }).fill("First Author");
  await bookDialog.getByLabel("価格", { exact: true }).fill("1000");
  await bookDialog
    .getByLabel("出版社", { exact: true })
    .selectOption({ label: "First Publisher" });
  await bookDialog.getByRole("button", { name: "保存する" }).click();
  await expect(bookDialog).not.toBeVisible();
  await expect(page.getByText("First Book", { exact: true })).toBeVisible();
  expect(state.bookWrites).toHaveLength(1);
});

test("shows summary loading and failure states and retries without showing false zeros", async ({
  page,
}) => {
  const state = await mockApi(page);
  state.books[0]!.applied_at = "2026-01-01T00:00:00Z";
  state.failSummary = true;
  let releaseSummary = () => {};
  state.holdSummary = new Promise<void>((resolve) => {
    releaseSummary = resolve;
  });
  await page.goto("/library");
  const summary = page.getByRole("complementary", { name: "予算概要" });
  await expect(summary.getByRole("status")).toContainText("読み込み中");
  await expect(summary.getByText("￥0", { exact: true })).not.toBeVisible();

  releaseSummary();
  await expect(summary.getByRole("alert")).toContainText(
    "取得できませんでした",
    { timeout: 10_000 },
  );
  await expect(
    summary.getByText("データがありません", { exact: true }),
  ).not.toBeVisible();
  await expect(summary.getByText("￥0", { exact: true })).not.toBeVisible();
  state.failSummary = false;
  await summary.getByRole("button", { name: "再試行" }).click();
  await expect(summary.getByRole("alert")).not.toBeVisible();
  await expect(summary).toContainText("1冊");
  await expect(summary.getByText("￥1,000", { exact: true })).toBeVisible();
  await expect(summary).toContainText("￥1,000 使用済");
});

test.describe("UTC expense dates", () => {
  test.use({ timezoneId: "America/Los_Angeles" });

  test("keeps the date and year in a timezone west of UTC", async ({
    page,
  }) => {
    const state = await mockApi(page);
    state.books[0]!.applied_at = "2026-01-01T00:00:00Z";
    await page.goto("/library");
    const row = page.getByRole("row").filter({ hasText: "Original Book" });
    await expect(row).toContainText("2026/1/1");
    await expect(row).not.toContainText("2025/12/31");
    await expect(
      page.getByRole("complementary", { name: "予算概要" }),
    ).toContainText("2026年度");
  });
});

test.beforeEach(async ({ context, baseURL }) => {
  if (!baseURL) throw new Error("Browser tests require a base URL");
  await context.addCookies([
    { name: "csrf_token", value: "test-csrf", url: baseURL },
  ]);
});

test("navigates between the library and publishers using React Router", async ({
  page,
}) => {
  await mockApi(page);
  await page.goto("/");
  await expect(page).toHaveURL(/\/library$/);
  await expect(page.getByText("Original Book", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("link", { name: "ライブラリ", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  await page.getByRole("link", { name: "出版社", exact: true }).click();
  await expect(page).toHaveURL(/\/publishers$/);
  await expect(page.getByRole("heading", { name: "出版社管理" })).toBeVisible();
  await expect(page.getByText("Test Publisher", { exact: true })).toBeVisible();
});

test("creates an EBook, sends CSRF, and keeps the editor pending until saved", async ({
  page,
}) => {
  const state = await mockApi(page);
  let releaseSave = () => {};
  state.holdBookSave = new Promise<void>((resolve) => {
    releaseSave = resolve;
  });
  await page.goto("/library");
  await page.getByRole("button", { name: "書籍を追加" }).click();
  const dialog = page.getByRole("dialog", { name: "新しい書籍を追加" });
  await dialog.getByLabel("タイトル", { exact: true }).fill("New EBook");
  await dialog.getByLabel("著者", { exact: true }).fill("New Author");
  await dialog.getByLabel("価格", { exact: true }).fill("1500");
  await dialog.getByLabel("形式", { exact: true }).selectOption("EBook");
  await dialog
    .getByLabel("出版社", { exact: true })
    .selectOption(state.publishers[0]!.pub_id);
  await dialog
    .getByLabel("購入店舗 (任意)", { exact: true })
    .selectOption("22222222-2222-4222-8222-222222222222");
  await dialog.getByRole("button", { name: "保存する" }).click();
  await expect(dialog.getByRole("button", { name: "保存する" })).toBeDisabled();
  await expect(dialog.getByRole("button", { name: "閉じる" })).toBeDisabled();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeVisible();
  releaseSave();
  await expect(dialog).not.toBeVisible();
  await expect(page.getByText("New EBook", { exact: true })).toBeVisible();
  expect(state.bookWrites).toHaveLength(1);
  expect(state.bookWrites[0]).toMatchObject({
    csrfToken: "test-csrf",
    data: {
      title: "New EBook",
      author: "New Author",
      price: 1500,
      format: "EBook",
      shop_id: "22222222-2222-4222-8222-222222222222",
    },
  });
});

test("resets editor fields after cancelling and restores keyboard focus", async ({
  page,
}) => {
  await mockApi(page);
  await page.goto("/library");
  await page.getByText("Original Book", { exact: true }).click();
  const edit = page.getByRole("dialog", { name: "書籍情報を編集" });
  await expect(edit.getByLabel("タイトル", { exact: true })).toHaveValue(
    "Original Book",
  );
  await edit.getByLabel("タイトル", { exact: true }).fill("Discarded title");
  await page.keyboard.press("Escape");
  await expect(edit).not.toBeVisible();
  const add = page.getByRole("button", { name: "書籍を追加" });
  await add.click();
  const create = page.getByRole("dialog", { name: "新しい書籍を追加" });
  await expect(create.getByLabel("タイトル", { exact: true })).toHaveValue("");
  await expect(create.getByLabel("タイトル", { exact: true })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(add).toBeFocused();
  await page.getByText("Original Book", { exact: true }).click();
  await expect(edit.getByLabel("タイトル", { exact: true })).toHaveValue(
    "Original Book",
  );
});

test("shows publisher save errors inline, preserves input, and supports retry", async ({
  page,
}) => {
  await mockApi(page);
  await page.goto("/publishers");
  await page.getByRole("button", { name: "出版社を追加" }).click();
  const dialog = page.getByRole("dialog", { name: "新しい出版社を追加" });
  await dialog.getByLabel("出版社名", { exact: true }).fill("Test Publisher");
  await dialog.getByRole("button", { name: "保存する" }).click();
  await expect(dialog.getByRole("alert")).toContainText(
    "同じ名前の出版社がすでに登録されています。",
  );
  await expect(dialog.getByLabel("出版社名", { exact: true })).toHaveValue(
    "Test Publisher",
  );
  await dialog.getByLabel("出版社名", { exact: true }).fill("Retry Publisher");
  await dialog.getByRole("button", { name: "保存する" }).click();
  await expect(dialog).not.toBeVisible();
  await expect(
    page.getByText("Retry Publisher", { exact: true }),
  ).toBeVisible();
});

test("rejects impossible expense dates and refreshes the annual summary after applying", async ({
  page,
}) => {
  const state = await mockApi(page);
  const alerts: string[] = [];
  const invalidDate = (dialog: Dialog) => {
    if (dialog.type() === "prompt") return dialog.accept("2025-02-29");
    alerts.push(dialog.message());
    return dialog.dismiss();
  };
  await page.goto("/library");
  page.on("dialog", invalidDate);
  await page.getByRole("button", { name: "申請", exact: true }).click();
  await expect.poll(() => alerts.length).toBe(1);
  page.off("dialog", invalidDate);
  expect(state.appliedWrites).toHaveLength(0);
  page.once("dialog", (dialog) => dialog.accept("2024-12-31"));
  await page.getByRole("button", { name: "申請", exact: true }).click();
  await expect(page.getByText("申請済", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("columnheader", { name: "申請日" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "書籍「Original Book」を編集" }),
  ).toHaveCount(0);
  await page
    .getByRole("button", { name: "書籍「Original Book」の詳細を表示" })
    .click();
  const details = page.getByRole("dialog", { name: "申請済み書籍の詳細" });
  await expect(details.getByText("2024/12/31")).toBeVisible();
  await expect(details.getByRole("button", { name: "更新する" })).toHaveCount(
    0,
  );
  await details.getByRole("button", { name: "閉じる" }).click();
  await expect(page.getByText("2024年度", { exact: true })).toBeVisible();
  expect(state.appliedWrites).toEqual([
    { applied_at: "2024-12-31T00:00:00.000Z", revision: 0 },
  ]);
});

test("shows an undated legacy application as read-only in the library", async ({
  page,
}) => {
  const state = await mockApi(page);
  state.books[0]!.applied_without_date = true;
  await page.goto("/library");
  const row = page.getByRole("row").filter({ hasText: "Original Book" });
  await expect(row).toContainText("日付不明");
  await expect(
    row.getByRole("button", { name: "申請", exact: true }),
  ).toHaveCount(0);
  await row
    .getByRole("button", { name: "書籍「Original Book」の詳細を表示" })
    .click();
  const details = page.getByRole("dialog", { name: "申請済み書籍の詳細" });
  await expect(details.getByText("日付不明")).toBeVisible();
  await expect(details.getByRole("textbox")).toHaveCount(0);
  expect(state.appliedWrites).toHaveLength(0);
});

test("redirects expired sessions to the login page", async ({ page }) => {
  await page.route("https://**/*", (route) => route.abort());
  await page.route(
    (url) => url.pathname.startsWith("/api/"),
    (route) => route.fulfill({ status: 401, body: "Unauthorized" }),
  );
  await page.goto("/library");
  await expect(page).toHaveURL(/\/login$/);
  await expect(
    page.getByRole("button", { name: "ログイン", exact: true }),
  ).toBeVisible();
});

test("shows callback errors and starts login again only on request", async ({
  page,
}) => {
  await page.route("https://**/*", (route) => route.abort());
  await page.route("**/api/login", (route) =>
    route.fulfill({ body: "OIDC login" }),
  );
  await page.goto("/login?error=token_exchange_failed");
  await expect(page.getByRole("alert")).toContainText(
    "認証サービスとの通信に失敗",
  );
  await page.getByRole("button", { name: "ログインをやり直す" }).click();
  await expect(page).toHaveURL(/\/api\/login$/);
});

test("posts logout with CSRF before navigating to the provider", async ({
  page,
}) => {
  await mockApi(page);
  const methods: string[] = [];
  await page.route("**/api/logout", async (route) => {
    methods.push(route.request().method());
    expect(route.request().headers()["x-csrf-token"]).toBe("test-csrf");
    if (methods.length === 1)
      return route.fulfill({ status: 500, json: { error: "一時エラー" } });
    return route.fulfill({
      json: { redirect_url: "https://issuer.test/logout" },
    });
  });
  await page.route("https://issuer.test/logout", (route) =>
    route.fulfill({ body: "Signed out" }),
  );
  await page.goto("/library");
  await page.getByRole("button", { name: "ログアウト" }).click();
  await expect(page.getByRole("alert")).toContainText("一時エラー");
  await page.getByRole("button", { name: "ログアウト" }).click();
  await expect(page).toHaveURL("https://issuer.test/logout");
  expect(methods).toEqual(["POST", "POST"]);
});
