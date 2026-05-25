import { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { getGetAllBooksQueryKey, useGetAllBooks, useCreateBook, useUpdateBook, useChangeBookAppliedAt, type getAllBooksResponse } from '../api/endpoints/book/book';
import { getGetAnnualSummaryQueryKey } from '../api/endpoints/dashboard/dashboard';
import BudgetSummary from '../components/BudgetSummary';
import BookModal from '../components/BookModal';
import type { BookResponseDto, BookCreateDto, BookUpdateDto } from '../api/model';

const Library = () => {
    const queryClient = useQueryClient();
    const { data: booksData, isLoading, error } = useGetAllBooks<getAllBooksResponse, Error>();
    // The response object contains the data array
    const books = booksData?.data || [];

    // Mutation hooks usually take TError as the first generic argument
    const createMutation = useCreateBook<Error>();
    const updateMutation = useUpdateBook<Error>();
    const changeAppliedAtMutation = useChangeBookAppliedAt<Error>();

    const [isModalOpen, setIsModalOpen] = useState(false);
    const [selectedBook, setSelectedBook] = useState<BookResponseDto | undefined>(undefined);

    const handleAddClick = () => {
        setSelectedBook(undefined);
        setIsModalOpen(true);
    };

    const handleBookClick = (book: BookResponseDto) => {
        setSelectedBook(book);
        setIsModalOpen(true);
    };

    const handleModalClose = () => {
        setIsModalOpen(false);
        setSelectedBook(undefined);
    };

    const invalidateBookViews = () => {
        queryClient.invalidateQueries({ queryKey: getGetAllBooksQueryKey() });
        queryClient.invalidateQueries({ queryKey: getGetAnnualSummaryQueryKey() });
    };

    const handleModalSubmit = (data: BookCreateDto | BookUpdateDto) => {
        if (selectedBook) {
            updateMutation.mutate(
                { pubId: selectedBook.pub_id, data: data as BookUpdateDto },
                {
                    onSuccess: () => {
                        invalidateBookViews();
                        handleModalClose();
                    },
                    onError: (err: Error) => {
                        console.error('Update failed:', err);
                        alert('更新に失敗しました: ' + err.message);
                    },
                }
            );
        } else {
            // Orval with tags-split usually expects { data: body } for mutation arguments if configured that way
            createMutation.mutate({ data: data as BookCreateDto }, {
                onSuccess: () => {
                    invalidateBookViews();
                    handleModalClose();
                },
                onError: (err: Error) => {
                    console.error('Create failed:', err);
                    alert('登録に失敗しました: ' + err.message);
                },
            });
        }
    };

    const handleApply = (book: BookResponseDto, e: React.MouseEvent) => {
        e.stopPropagation();

        const today = new Date().toISOString().split('T')[0];
        const inputDate = prompt(`「${book.title}」の経費申請を行います。\n申請日を入力してください (YYYY-MM-DD):`, today);

        if (inputDate === null) return; // User cancelled

        const date = new Date(inputDate);
        if (isNaN(date.getTime())) {
            alert('無効な日付形式です。YYYY-MM-DD形式で入力してください。');
            return;
        }

        changeAppliedAtMutation.mutate(
            {
                pubId: book.pub_id,
                data: { applied_at: date.toISOString() }
            },
            {
                onSuccess: () => {
                    invalidateBookViews();
                },
                onError: (err: Error) => {
                    console.error('Apply failed:', err);
                    alert('申請に失敗しました: ' + err.message);
                }
            }
        );
    };


    if (isLoading) {
        return (
            <main className="flex-1 min-w-0 flex items-center justify-center h-64">
                <div className="text-text-sub flex items-center gap-2">
                    <span className="material-symbols-outlined animate-spin text-primary">sync</span>
                    読み込み中...
                </div>
            </main>
        );
    }

    if (error) {
        return (
            <main className="flex-1 min-w-0 flex items-center justify-center h-64">
                <div className="text-accent-red font-bold flex flex-col items-center gap-2">
                    <span className="material-symbols-outlined text-4xl">error</span>
                    データの取得に失敗しました。
                </div>
            </main>
        );
    }

    return (
        <>
            <main className="flex-1 min-w-0">
                <div className="flex flex-col gap-6">
                    <div className="flex flex-wrap items-end justify-between gap-4 border-b border-border-color pb-4">
                        <div>
                            <h1 className="text-2xl font-bold text-text-main dark:text-white leading-tight">購入履歴</h1>
                            <p className="text-text-sub dark:text-gray-400 text-sm mt-1 text-[13px]">コレクションの管理と経費の追跡</p>
                        </div>
                        <button
                            onClick={handleAddClick}
                            className="flex items-center justify-center gap-2 bg-primary hover:bg-primary-dark text-white text-[13px] font-bold px-5 py-2.5 rounded-md shadow-sm transition-all hover:scale-[1.02] active:scale-95 focus:ring-2 focus:ring-offset-2 focus:ring-primary"
                        >
                            <span className="material-symbols-outlined text-lg">add</span>
                            <span>書籍を追加</span>
                        </button>
                    </div>

                    <div className="flex gap-4 border-b border-border-color dark:border-gray-700">
                        <button className="px-1 py-3 border-b-2 border-primary text-primary text-sm font-bold">すべての書籍</button>
                        <button className="px-1 py-3 border-b-2 border-transparent text-text-sub hover:text-text-main hover:border-gray-300 text-sm font-medium transition-colors">
                            お気に入り
                        </button>
                        <button className="px-1 py-3 border-b-2 border-transparent text-text-sub hover:text-text-main hover:border-gray-300 text-sm font-medium transition-colors">
                            高額書籍
                        </button>
                    </div>

                    <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 shadow-sm overflow-hidden">
                        <div className="overflow-x-auto">
                            <table className="w-full text-left border-collapse">
                                <thead>
                                    <tr className="bg-gray-100 dark:bg-gray-800 border-b border-border-color dark:border-gray-700">
                                        <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[35%]">
                                            書籍タイトル
                                        </th>
                                        <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[20%] hidden sm:table-cell">
                                            著者
                                        </th>
                                        <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[15%] hidden md:table-cell">
                                            購入日
                                        </th>
                                        <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[15%] text-center">
                                            経費申請
                                        </th>
                                        <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[15%] text-right">
                                            価格
                                        </th>
                                    </tr>
                                </thead>
                                <tbody className="divide-y divide-border-color dark:divide-gray-700">
                                    {books.map((book, index) => {
                                        const isPending = !book.applied_at;
                                        return (
                                            <tr
                                                key={book.pub_id || index}
                                                onClick={() => handleBookClick(book)}
                                                className={`cursor-pointer transition-all ${isPending
                                                    ? 'row-highlight-pending hover:bg-amber-100 dark:hover:bg-amber-900/30'
                                                    : 'group hover:bg-gray-50 dark:hover:bg-gray-800/50'
                                                    }`}
                                            >
                                                <td className="px-6 py-4">
                                                    <div className="flex flex-col">
                                                        <div className="flex items-center gap-2">
                                                            {isPending && (
                                                                <span className="material-symbols-outlined text-accent-yellow text-sm font-bold">
                                                                    warning
                                                                </span>
                                                            )}
                                                            <p className="text-text-main dark:text-white text-sm font-bold leading-normal group-hover:text-primary transition-colors">
                                                                {book.title}
                                                            </p>
                                                        </div>
                                                        <p className="text-text-sub dark:text-gray-400 text-xs sm:hidden mt-1">{book.author}</p>
                                                    </div>
                                                </td>
                                                <td className="px-6 py-4 text-sm text-text-sub dark:text-gray-300 hidden sm:table-cell">
                                                    {book.author}
                                                </td>
                                                <td className="px-6 py-4 text-sm text-text-sub dark:text-gray-400 hidden md:table-cell">
                                                    {book.applied_at ? new Date(book.applied_at).toLocaleDateString() : '-'}
                                                </td>
                                                <td className="px-6 py-4 text-center">
                                                    <div className="flex items-center justify-center gap-2">
                                                        {!isPending ? (
                                                            <>
                                                                <span className="material-symbols-outlined text-xl text-primary">
                                                                    check_box
                                                                </span>
                                                                <span className="text-xs font-bold text-text-main dark:text-gray-300 font-medium">
                                                                    申請済
                                                                </span>
                                                            </>
                                                        ) : (
                                                            <button
                                                                onClick={(e) => handleApply(book, e)}
                                                                className="flex items-center gap-1 bg-white border border-primary text-primary px-3 py-1 rounded text-xs font-bold hover:bg-primary hover:text-white transition-colors shadow-sm disabled:opacity-50 disabled:cursor-not-allowed"
                                                                disabled={changeAppliedAtMutation.isPending}
                                                            >
                                                                <span className="material-symbols-outlined text-base">send</span>
                                                                申請
                                                            </button>
                                                        )}
                                                    </div>
                                                </td>
                                                <td className="px-6 py-4 text-sm font-bold text-text-main dark:text-white text-right">
                                                    ¥{book.price.toLocaleString()}
                                                </td>
                                            </tr>
                                        );
                                    })}
                                </tbody>
                            </table>
                        </div>
                        <div className="flex items-center justify-between px-6 py-4 border-t border-border-color dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
                            <p className="text-xs text-text-sub dark:text-gray-400">
                                表示中: <span className="font-bold text-text-main dark:text-white">1</span> -{' '}
                                <span className="font-bold text-text-main dark:text-white">{books.length}</span> /{' '}
                                <span className="font-bold text-text-main dark:text-white">{books.length}</span> 件
                            </p>
                            <div className="flex gap-1">
                                <button
                                    className="flex items-center justify-center size-8 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors disabled:opacity-50 text-text-main dark:text-white"
                                    disabled
                                >
                                    <span className="material-symbols-outlined text-lg">chevron_left</span>
                                </button>
                                <button className="flex items-center justify-center size-8 rounded bg-primary text-white text-sm font-bold shadow-sm">
                                    1
                                </button>
                                <button className="flex items-center justify-center size-8 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-text-main dark:text-white">
                                    <span className="material-symbols-outlined text-lg">chevron_right</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </main>
            <BudgetSummary />

            <BookModal
                isOpen={isModalOpen}
                onClose={handleModalClose}
                onSubmit={handleModalSubmit}
                book={selectedBook}
                isSubmitting={createMutation.isPending || updateMutation.isPending}
            />
        </>
    );
};

export default Library;
