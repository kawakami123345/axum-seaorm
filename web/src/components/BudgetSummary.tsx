const BudgetSummary = () => {
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
                        <div>
                            <div className="flex justify-between items-end mb-2">
                                <span className="text-sm font-bold text-text-main dark:text-white">2024年度</span>
                                <span className="text-xs font-medium text-text-sub">
                                    残高: <span className="text-emerald-700 dark:text-emerald-400 font-bold">¥74,800</span>
                                </span>
                            </div>
                            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
                                <div className="bg-primary h-3 rounded-full" style={{ width: '37.6%' }}></div>
                            </div>
                            <div className="flex justify-between items-center mt-1 text-xs">
                                <span className="text-text-main dark:text-gray-200 font-bold">
                                    ¥45,200 <span className="font-normal text-text-sub">使用済</span>
                                </span>
                                <span className="text-text-sub">目標: ¥120,000</span>
                            </div>
                        </div>
                        <div>
                            <div className="flex justify-between items-end mb-2">
                                <span className="text-sm font-bold text-text-main dark:text-white">2023年度</span>
                                <span className="text-xs font-medium text-text-sub">
                                    残高: <span className="text-emerald-700 dark:text-emerald-400 font-bold">¥2,100</span>
                                </span>
                            </div>
                            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
                                <div className="bg-blue-400 dark:bg-blue-500 h-3 rounded-full" style={{ width: '98%' }}></div>
                            </div>
                            <div className="flex justify-between items-center mt-1 text-xs">
                                <span className="text-text-main dark:text-gray-200 font-bold">
                                    ¥97,900 <span className="font-normal text-text-sub">使用済</span>
                                </span>
                                <span className="text-text-sub">目標: ¥100,000</span>
                            </div>
                        </div>
                        <div>
                            <div className="flex justify-between items-end mb-2">
                                <span className="text-sm font-bold text-text-main dark:text-white">2022年度</span>
                                <span className="text-xs font-medium text-text-sub">
                                    超過: <span className="text-accent-red font-bold">¥5,000</span>
                                </span>
                            </div>
                            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
                                <div className="bg-accent-red h-3 rounded-full" style={{ width: '100%' }}></div>
                            </div>
                            <div className="flex justify-between items-center mt-1 text-xs">
                                <span className="text-text-main dark:text-gray-200 font-bold">
                                    ¥85,000 <span className="font-normal text-text-sub">使用済</span>
                                </span>
                                <span className="text-text-sub">目標: ¥80,000</span>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="grid grid-cols-2 gap-4">
                    <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 p-5 flex flex-col justify-between h-28 shadow-sm">
                        <div className="flex items-center gap-2 text-text-sub dark:text-gray-400">
                            <span className="material-symbols-outlined text-xl">library_books</span>
                            <p className="text-xs font-bold uppercase tracking-wide">購入数</p>
                        </div>
                        <p className="text-3xl font-bold text-text-main dark:text-white">
                            24<span className="text-sm font-medium text-text-sub ml-1">冊</span>
                        </p>
                    </div>
                    <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 p-5 flex flex-col justify-between h-28 shadow-sm">
                        <div className="flex items-center gap-2 text-text-sub dark:text-gray-400">
                            <span className="material-symbols-outlined text-xl">payments</span>
                            <p className="text-xs font-bold uppercase tracking-wide">平均価格</p>
                        </div>
                        <p className="text-3xl font-bold text-text-main dark:text-white">¥1,883</p>
                    </div>
                </div>
            </div>
        </aside>
    );
};

export default BudgetSummary;
