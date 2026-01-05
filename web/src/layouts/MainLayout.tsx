import { Outlet, Link, useLocation } from 'react-router-dom';

const MainLayout = () => {
    const location = useLocation();

    const navItems = [
        { name: 'ダッシュボード', path: '/dashboard' },
        { name: 'ライブラリ', path: '/library' },
        { name: '出版社', path: '/publishers' },
        { name: 'レポート', path: '/reports' },
    ];


    return (
        <div className="min-h-screen bg-background-light dark:bg-background-dark">
            <header className="sticky top-0 z-50 bg-white dark:bg-card-dark border-b border-border-color dark:border-gray-700 shadow-sm">
                <div className="px-4 lg:px-8 py-4 flex items-center justify-between">
                    <div className="flex items-center gap-6">
                        <Link to="/" className="flex items-center gap-3">
                            <span className="material-symbols-outlined text-primary text-3xl">local_library</span>
                            <h2 className="text-xl font-bold tracking-tight text-text-main dark:text-white">BookKeeper</h2>
                        </Link>
                        <div className="hidden md:flex relative w-64">
                            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <span className="material-symbols-outlined text-text-sub text-lg">search</span>
                            </div>
                            <input
                                className="block w-full pl-10 pr-3 py-2 border border-border-color rounded-md leading-5 bg-white dark:bg-gray-800 dark:border-gray-600 placeholder-text-sub focus:outline-none focus:ring-2 focus:ring-primary focus:border-primary sm:text-sm transition-shadow"
                                placeholder="書籍を検索..."
                                type="search"
                            />
                        </div>
                    </div>
                    <div className="flex items-center gap-6">
                        <nav className="hidden md:flex gap-6">
                            {navItems.map((item) => {
                                const isActive = location.pathname === item.path;
                                return (
                                    <Link
                                        key={item.path}
                                        to={item.path}
                                        className={`text-sm tracking-wide transition-colors pb-0.5 ${isActive
                                            ? 'font-bold text-primary border-b-2 border-primary'
                                            : 'font-medium text-text-sub hover:text-primary'
                                            }`}
                                    >
                                        {item.name}
                                    </Link>
                                );
                            })}
                        </nav>
                        <div className="flex items-center gap-4 border-l border-gray-200 dark:border-gray-700 pl-4">
                            <button className="text-text-sub hover:text-primary transition-colors p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800">
                                <span className="material-symbols-outlined">notifications</span>
                            </button>
                            <div className="size-9 rounded-full bg-gray-200 overflow-hidden border border-gray-300 dark:border-gray-600">
                                <div
                                    className="bg-center bg-no-repeat bg-cover h-full w-full"
                                    style={{
                                        backgroundImage:
                                            'url("https://lh3.googleusercontent.com/aida-public/AB6AXuD5d8rsvJRju4vTKOk6wiKxtN5IrhWGG2TC1QbYWP6-uGRaA_6JNVvZHZM439OoNTS7b07eH9WkGiuGcQ9qIqsaolXBHNbkA3HaxuMaPNenG2oUwC0wJbrtlWhQakJEyAh6TrBemimIuuf0_XuoWS-F-GYDCSHj18mKce7h3zOcs46KvXvrUxxqmxBs9vNhFtnFR4K4cVnLR2Ta-s8qkzq0SalNy4xsWaNKihQAqeEXH5nBhOlDMnctO8pSHFYTrJ-Iyp2z_puE60Q")',
                                    }}
                                ></div>
                            </div>
                        </div>
                    </div>
                </div>
            </header>

            <div className="flex flex-col lg:flex-row gap-8 max-w-7xl mx-auto w-full px-4 lg:px-8 py-10">
                <Outlet />
            </div>
        </div>
    );
};

export default MainLayout;
