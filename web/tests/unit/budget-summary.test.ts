import { describe, expect, test } from "bun:test";
import type { DashboardDto } from "../../src/api/model";
import { getBudgetOverview } from "../../src/features/budget/summary";

function annualSummary(
  year: number,
  count: number,
  total: number,
): DashboardDto {
  return {
    year,
    count,
    total_amount: total,
    limit: 20000,
    balance: 20000 - total,
    average: count > 0 ? total / count : 0,
  };
}

describe("budget overview", () => {
  test("calculates the average from all books rather than averaging yearly averages", () => {
    const result = getBudgetOverview([
      annualSummary(2024, 2, 1000),
      annualSummary(2026, 1, 10000),
    ]);
    expect(result.totalCount).toBe(3);
    expect(result.averagePrice).toBe(3667);
    expect(result.years.map((summary) => summary.year)).toEqual([2026, 2024]);
  });

  test("sorts years without mutating cached query data", () => {
    const summaries = Object.freeze([
      annualSummary(2024, 1, 100),
      annualSummary(2026, 1, 200),
    ]);
    const result = getBudgetOverview(summaries);
    expect(summaries.map((summary) => summary.year)).toEqual([2024, 2026]);
    expect(result.years.map((summary) => summary.year)).toEqual([2026, 2024]);
  });

  test("handles empty data and zero-priced books", () => {
    expect(getBudgetOverview([])).toEqual({
      years: [],
      totalCount: 0,
      averagePrice: 0,
    });
    expect(getBudgetOverview([annualSummary(2026, 2, 0)])).toMatchObject({
      totalCount: 2,
      averagePrice: 0,
    });
  });

  test("keeps totals above the maximum individual book price", () => {
    const result = getBudgetOverview([annualSummary(2026, 2, 4294967294)]);
    expect(result.totalCount).toBe(2);
    expect(result.averagePrice).toBe(2147483647);
  });
});
