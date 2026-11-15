import { expect, test } from "@playwright/test";
import { mockApi } from "./api-fixture";

test.beforeEach(async ({ context, baseURL }) => {
  if (!baseURL) throw new Error("Browser tests require a base URL");
  await context.addCookies([
    { name: "csrf_token", value: "test-csrf", url: baseURL },
  ]);
});

test("opens book and publisher editors with the keyboard and restores focus", async ({
  page,
}) => {
  await mockApi(page);
  await page.goto("/library");
  const book = page.getByRole("button", {
    name: "書籍「Original Book」を編集",
  });
  await book.focus();
  await page.keyboard.press("Enter");
  const bookDialog = page.getByRole("dialog", { name: "書籍情報を編集" });
  await expect(
    bookDialog.getByLabel("タイトル", { exact: true }),
  ).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(bookDialog).not.toBeVisible();
  await expect(book).toBeFocused();

  await page.getByRole("link", { name: "出版社", exact: true }).click();
  const publisher = page.getByRole("button", {
    name: "出版社「Test Publisher」を編集",
  });
  await publisher.focus();
  await page.keyboard.press("Space");
  const publisherDialog = page.getByRole("dialog", {
    name: "出版社情報を編集",
  });
  await expect(
    publisherDialog.getByLabel("出版社名", { exact: true }),
  ).toHaveValue("Test Publisher");
  await page.keyboard.press("Escape");
  await expect(publisher).toBeFocused();
});

test("preserves edits after a conflict and allows a retry", async ({
  page,
}) => {
  const state = await mockApi(page);
  state.failNextBookUpdate = true;
  await page.goto("/library");
  await page
    .getByRole("button", { name: "書籍「Original Book」を編集" })
    .click();
  const dialog = page.getByRole("dialog", { name: "書籍情報を編集" });
  await dialog.getByLabel("タイトル", { exact: true }).fill("Retried Book");
  await dialog.getByRole("button", { name: "更新する" }).click();
  await expect(dialog.getByRole("alert")).toContainText("再読み込み");
  await expect(dialog.getByLabel("タイトル", { exact: true })).toHaveValue(
    "Retried Book",
  );
  await expect(dialog.getByRole("button", { name: "更新する" })).toBeEnabled();
  await dialog.getByRole("button", { name: "更新する" }).click();
  await expect(dialog).not.toBeVisible();
  await expect(
    page.getByRole("button", { name: "書籍「Retried Book」を編集" }),
  ).toBeVisible();
  expect(state.books[0]!.title).toBe("Retried Book");
});

test("retries a failed book list without treating it as an empty list", async ({
  page,
}) => {
  const state = await mockApi(page);
  state.failBooks = true;
  await page.goto("/library");
  const main = page.getByRole("main");
  await expect(main.getByRole("alert")).toContainText("取得に失敗", {
    timeout: 10_000,
  });
  await expect(
    main.getByText("書籍が登録されていません", { exact: false }),
  ).not.toBeVisible();
  state.failBooks = false;
  await main.getByRole("button", { name: "再試行" }).click();
  await expect(page.getByRole("table", { name: "書籍一覧" })).toBeVisible();
  await expect(
    page.getByRole("button", { name: "書籍「Original Book」を編集" }),
  ).toBeVisible();
});

test("retries failed editor choices without reporting a missing publisher", async ({
  page,
}) => {
  const state = await mockApi(page);
  state.failShops = true;
  await page.goto("/library");
  await page.getByRole("button", { name: "書籍を追加" }).click();
  const dialog = page.getByRole("dialog", { name: "新しい書籍を追加" });
  await expect(dialog.getByRole("alert")).toContainText(
    "出版社・店舗の取得に失敗",
    { timeout: 10_000 },
  );
  await expect(
    dialog.getByText("出版社が登録されていません", { exact: false }),
  ).not.toBeVisible();
  await expect(
    dialog.getByRole("button", { name: "保存する" }),
  ).not.toBeVisible();
  state.failShops = false;
  await dialog.getByRole("button", { name: "再試行" }).click();
  await expect(
    dialog
      .getByLabel("出版社", { exact: true })
      .getByRole("option", { name: "Test Publisher" }),
  ).toHaveCount(1);
  await expect(dialog.getByRole("button", { name: "保存する" })).toBeVisible();
});
