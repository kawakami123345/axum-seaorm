import type { DashboardDto } from "../../api/model";

export function getBudgetOverview(summaries: readonly DashboardDto[]) {
  const years = [...summaries].sort((a, b) => b.year - a.year);
  const totals = summaries.reduce(
    (result, summary) => ({
      count: result.count + summary.count,
      amount: result.amount + summary.total_amount,
    }),
    { count: 0, amount: 0 },
  );

  return {
    years,
    totalCount: totals.count,
    averagePrice:
      totals.count > 0 ? Math.round(totals.amount / totals.count) : 0,
  };
}
