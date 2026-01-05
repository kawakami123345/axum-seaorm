import { useGetAnnualSummary, type getAnnualSummaryResponse } from '../api/endpoints/dashboard/dashboard';

const BudgetSummary = () => {
    const { data: response } = useGetAnnualSummary<getAnnualSummaryResponse, Error>();
    const summaries = response?.data || [];

    // Sort summaries by year descending (newest first)
    const sortedSummaries = [...summaries].sort((a, b) => b.year - a.year);

    // Calculate totals
    const totalCount = sortedSummaries.reduce((acc, curr) => acc + curr.count, 0);
    const totalAmount = sortedSummaries.reduce((acc, curr) => acc + curr.total_amount, 0);
    // Calculate global average price (Total Amount / Total Count)
    const globalAverage = totalCount > 0 ? Math.round(totalAmount / totalCount) : 0;

    const formatCurrency = (amount: number) => {
        return new Intl.NumberFormat('ja-JP', { style: 'currency', currency: 'JPY' }).format(amount);
    };

    return (
        <aside className="w-full lg:w-[320px] shrink-0">
            <div className="sticky top-28 flex flex-col gap-6">
                <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 shadow-sm p-6 relative overflow-hidden">
                    <div className="flex justify-between items-center mb-6">
                        <div className="flex items-center gap-2">
                            <span className="material-symbols-outlined text-primary text-xl">account_balance_wallet</span>
                            <h3 className="text-text-main dark:text-gray-200 text-sm font-bold uppercase tracking-wide">
                                予算概要
                            </h3>
                        </div>
                        <div className="flex bg-gray-100 dark:bg-gray-800 rounded p-1">
                            <button className="px-2 py-1 text-xs font-medium rounded text-text-sub dark:text-gray-400">
                                合計
                            </button>
                            <button className="px-2 py-1 text-xs font-bold rounded bg-white dark:bg-gray-700 text-primary shadow-sm">
                                年度別
                            </button>
                        </div>
                    </div>
                    <div className="flex flex-col gap-5">
                        {sortedSummaries.map((summary) => {
                            const isOverBudget = summary.balance < 0;
                            // Calculate percentage but cap at 100 for visual bar
                            const percentage = Math.min((summary.total_amount / summary.limit) * 100, 100);

                            return (
                                <div key={summary.year}>
                                    <div className="flex justify-between items-end mb-2">
                                        <span className="text-sm font-bold text-text-main dark:text-white">{summary.year}年度</span>
                                        <span className="text-xs font-medium text-text-sub">
                                            {isOverBudget ? (
                                                <>
                                                    超過: <span className="text-accent-red font-bold">{formatCurrency(Math.abs(summary.balance))}</span>
                                                </>
                                            ) : (
                                                <>
                                                    残高: <span className="text-emerald-700 dark:text-emerald-400 font-bold">{formatCurrency(summary.balance)}</span>
                                                </>
                                            )}
                                        </span>
                                    </div>
                                    <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
                                        <div
                                            className={`h-3 rounded-full ${isOverBudget ? 'bg-accent-red' : 'bg-primary'}`}
                                            style={{ width: `${percentage}%` }}
                                        ></div>
                                    </div>
                                    <div className="flex justify-between items-center mt-1 text-xs">
                                        <span className="text-text-main dark:text-gray-200 font-bold">
                                            {formatCurrency(summary.total_amount)} <span className="font-normal text-text-sub">使用済</span>
                                        </span>
                                        <span className="text-text-sub">目標: {formatCurrency(summary.limit)}</span>
                                    </div>
                                </div>
                            );
                        })}

                        {sortedSummaries.length === 0 && (
                            <p className="text-sm text-text-sub text-center py-4">データがありません</p>
                        )}
                    </div>
                </div>
                <div className="grid grid-cols-2 gap-4">
                    <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 p-5 flex flex-col justify-between h-28 shadow-sm">
                        <div className="flex items-center gap-2 text-text-sub dark:text-gray-400">
                            <span className="material-symbols-outlined text-xl">library_books</span>
                            <p className="text-xs font-bold uppercase tracking-wide">購入数</p>
                        </div>
                        <p className="text-3xl font-bold text-text-main dark:text-white">
                            {totalCount}<span className="text-sm font-medium text-text-sub ml-1">冊</span>
                        </p>
                    </div>
                    <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 p-5 flex flex-col justify-between h-28 shadow-sm">
                        <div className="flex items-center gap-2 text-text-sub dark:text-gray-400">
                            <span className="material-symbols-outlined text-xl">payments</span>
                            <p className="text-xs font-bold uppercase tracking-wide">平均価格</p>
                        </div>
                        <p className="text-3xl font-bold text-text-main dark:text-white">{formatCurrency(globalAverage)}</p>
                    </div>
                </div>
            </div>
        </aside>
    );
};

export default BudgetSummary;
