import type { ReactNode } from "react";
import { useGetAnnualSummary } from "../../api/endpoints/dashboard/dashboard";
import type { DashboardDto } from "../../api/model";
import Icon from "../../components/Icon";
import RequestStatus from "../../components/RequestStatus";
import { formatYen } from "../../lib/format";
import { getBudgetOverview } from "./summary";

export default function BudgetSummary() {
  const {
    data: summaries = [],
    isPending,
    error,
    refetch,
    isFetching,
  } = useGetAnnualSummary({
    query: { select: (response) => response.data },
  });
  const { years, totalCount, averagePrice } = getBudgetOverview(summaries);
  const isReady = !isPending && !error;

  return (
    <aside aria-label="予算概要" className="w-full lg:w-[320px] shrink-0">
      <div className="sticky top-28 flex flex-col gap-6">
        <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 shadow-sm p-6 relative overflow-hidden">
          <div className="flex justify-between items-center mb-6">
            <div className="flex items-center gap-2">
              <Icon
                name="account_balance_wallet"
                className="text-primary text-xl"
              />
              <h3 className="text-text-main dark:text-gray-200 text-sm font-bold uppercase tracking-wide">
                予算概要
              </h3>
            </div>
            {isReady && (
              <div className="flex bg-gray-100 dark:bg-gray-800 rounded p-1">
                <button
                  type="button"
                  className="px-2 py-1 text-xs font-medium rounded text-text-sub dark:text-gray-400"
                >
                  合計
                </button>
                <button
                  type="button"
                  className="px-2 py-1 text-xs font-bold rounded bg-white dark:bg-gray-700 text-primary shadow-sm"
                >
                  年度別
                </button>
              </div>
            )}
          </div>
          {!isReady ? (
            <RequestStatus
              status={isPending ? "loading" : "error"}
              message={
                isPending
                  ? "集計を読み込み中..."
                  : "予算概要を取得できませんでした。"
              }
              onRetry={() => void refetch()}
              isRetrying={isFetching}
            />
          ) : (
            <div className="flex flex-col gap-5">
              {years.map((summary) => (
                <BudgetYear key={summary.year} summary={summary} />
              ))}
              {years.length === 0 && (
                <p className="text-sm text-text-sub text-center py-4">
                  データがありません
                </p>
              )}
            </div>
          )}
        </div>
        {isReady && (
          <div className="grid grid-cols-2 gap-4">
            <BudgetStat icon="library_books" label="購入数">
              {totalCount}
              <span className="text-sm font-medium text-text-sub ml-1">冊</span>
            </BudgetStat>
            <BudgetStat icon="payments" label="平均価格">
              {formatYen(averagePrice)}
            </BudgetStat>
          </div>
        )}
      </div>
    </aside>
  );
}

function BudgetYear({ summary }: { summary: DashboardDto }) {
  const isOverBudget = summary.balance < 0;
  const percentage = Math.min(
    (summary.total_amount / summary.limit) * 100,
    100,
  );
  return (
    <div>
      <div className="flex justify-between items-end mb-2">
        <span className="text-sm font-bold text-text-main dark:text-white">
          {summary.year}年度
        </span>
        <span className="text-xs font-medium text-text-sub">
          {isOverBudget ? "超過: " : "残高: "}
          <span
            className={`font-bold ${isOverBudget ? "text-accent-red" : "text-emerald-700 dark:text-emerald-400"}`}
          >
            {formatYen(Math.abs(summary.balance))}
          </span>
        </span>
      </div>
      <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
        <div
          className={`h-3 rounded-full ${isOverBudget ? "bg-accent-red" : "bg-primary"}`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      <div className="flex justify-between items-center mt-1 text-xs">
        <span className="text-text-main dark:text-gray-200 font-bold">
          {formatYen(summary.total_amount)}{" "}
          <span className="font-normal text-text-sub">使用済</span>
        </span>
        <span className="text-text-sub">目標: {formatYen(summary.limit)}</span>
      </div>
    </div>
  );
}

function BudgetStat({
  icon,
  label,
  children,
}: {
  icon: string;
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 p-5 flex flex-col justify-between h-28 shadow-sm">
      <div className="flex items-center gap-2 text-text-sub dark:text-gray-400">
        <Icon name={icon} className="text-xl" />
        <p className="text-xs font-bold uppercase tracking-wide">{label}</p>
      </div>
      <p className="text-3xl font-bold text-text-main dark:text-white">
        {children}
      </p>
    </div>
  );
}
