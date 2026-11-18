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

for (const resource of ["book", "publisher"] as const) {
  test(`keeps a ${resource} draft when the list refetch after a conflict fails`, async ({
    page,
  }) => {
    const state = await mockApi(page);
    const book = resource === "book";
    await page.goto(book ? "/library" : "/publishers");
    await page
      .getByRole("button", {
        name: book
          ? "書籍「Original Book」を編集"
          : "出版社「Test Publisher」を編集",
      })
      .click();
    const dialog = page.getByRole("dialog");
    const field = dialog.getByLabel(book ? "タイトル" : "出版社名", {
      exact: true,
    });
    await field.fill("My draft");
    (book ? state.books[0]! : state.publishers[0]!).revision += 1;
    if (book) state.failBooks = true;
    else state.failPublishers = true;
    await dialog.getByRole("button", { name: "更新する" }).click();
    await expect(page.getByText(/前回の一覧を表示しています/)).toBeVisible({
      timeout: 12000,
    });
    await expect(field).toHaveValue("My draft");
    state.failBooks = state.failPublishers = false;
    await dialog.getByRole("button", { name: "更新する" }).click();
    await expect(page.getByText(/前回の一覧を表示しています/)).toHaveCount(0);
    await expect(field).toHaveValue("My draft");
    await expect(dialog.getByRole("alert")).toContainText("再読み込み");
  });
}

test("keeps a stale book edit version until the user reopens the latest record", async ({
  page,
}) => {
  const state = await mockApi(page);
  await page.goto("/library");
  await page
    .getByRole("button", { name: "書籍「Original Book」を編集" })
    .click();
  const dialog = page.getByRole("dialog", { name: "書籍情報を編集" });
  await dialog.getByLabel("タイトル", { exact: true }).fill("My draft");

  state.books[0]!.title = "Other edit";
  state.books[0]!.revision = 1;
  for (let attempt = 0; attempt < 2; attempt += 1) {
    await dialog.getByRole("button", { name: "更新する" }).click();
    await expect(dialog.getByRole("alert")).toContainText("再読み込み");
    await expect(dialog.getByLabel("タイトル", { exact: true })).toHaveValue(
      "My draft",
    );
    expect(state.bookUpdates[attempt]!.revision).toBe(0);
    expect(state.bookUpdates[attempt]).not.toHaveProperty("id");
    expect(state.bookUpdates[attempt]).not.toHaveProperty("pub_id");
    expect(state.books[0]!.title).toBe("Other edit");
    expect(state.books[0]!.revision).toBe(1);
  }

  await dialog.getByRole("button", { name: "キャンセル" }).click();
  await page.getByRole("button", { name: "書籍「Other edit」を編集" }).click();
  await expect(dialog.getByLabel("タイトル", { exact: true })).toHaveValue(
    "Other edit",
  );
  await dialog.getByLabel("タイトル", { exact: true }).fill("Reviewed edit");
  await dialog.getByRole("button", { name: "更新する" }).click();
  await expect(dialog).not.toBeVisible();
  expect(state.bookUpdates[2]!.revision).toBe(1);
  expect(state.books[0]!.revision).toBe(2);
  expect(state.books[0]!.title).toBe("Reviewed edit");
});

test("sends the publisher revision and preserves stale edits on conflict", async ({
  page,
}) => {
  const state = await mockApi(page);
  await page.goto("/publishers");
  await page
    .getByRole("button", { name: "出版社「Test Publisher」を編集" })
    .click();
  const dialog = page.getByRole("dialog", { name: "出版社情報を編集" });
  await dialog
    .getByLabel("出版社名", { exact: true })
    .fill("My publisher draft");
  state.publishers[0]!.name = "Other publisher edit";
  state.publishers[0]!.revision = 1;
  await dialog.getByRole("button", { name: "更新する" }).click();
  await expect(dialog.getByRole("alert")).toContainText("再読み込み");
  await expect(dialog.getByLabel("出版社名", { exact: true })).toHaveValue(
    "My publisher draft",
  );
  expect(state.publisherUpdates[0]).toEqual({
    name: "My publisher draft",
    revision: 0,
  });
  expect(state.publishers[0]!.name).toBe("Other publisher edit");
  await dialog.getByRole("button", { name: "キャンセル" }).click();
  await page
    .getByRole("button", { name: "出版社「Other publisher edit」を編集" })
    .click();
  await dialog
    .getByLabel("出版社名", { exact: true })
    .fill("Reviewed publisher");
  await dialog.getByRole("button", { name: "更新する" }).click();
  await expect(dialog).not.toBeVisible();
  expect(state.publisherUpdates[1]!.revision).toBe(1);
  expect(state.publishers[0]!.revision).toBe(2);
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
