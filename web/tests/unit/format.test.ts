import { describe, expect, test } from "bun:test";
import { formatYen, parseAppliedDate } from "../../src/lib/format";

describe("expense date input", () => {
  test("accepts a leap day and produces UTC midnight", () => {
    expect(parseAppliedDate("2024-02-29")).toBe("2024-02-29T00:00:00.000Z");
  });

  test("trims surrounding whitespace", () => {
    expect(parseAppliedDate(" 2026-11-14 ")).toBe("2026-11-14T00:00:00.000Z");
  });

  test.each([
    "2025-02-29",
    "2024-04-31",
    "2024-13-01",
    "2024-00-12",
    "2024-01-00",
    "2024-01-32",
  ])("rejects nonexistent calendar date %s", (value) =>
    expect(parseAppliedDate(value)).toBeNull(),
  );

  test.each([
    "2024-1-01",
    "01/01/2024",
    "2024-01-01T00:00:00Z",
    "",
    "not a date",
  ])("rejects non-date input %s", (value) =>
    expect(parseAppliedDate(value)).toBeNull(),
  );
});

test("formats zero and negative JPY amounts consistently", () => {
  expect(formatYen(0)).toBe("￥0");
  expect(formatYen(-2500)).toBe("-￥2,500");
});
