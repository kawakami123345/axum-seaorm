import { useState } from 'react';
import { useGetAllPublishers, useCreatePublisher, useUpdatePublisher } from '../api/endpoints/publisher';
import BudgetSummary from '../components/BudgetSummary';
import PublisherModal from '../components/PublisherModal';
import type { PublisherResponseDto, PublisherCreateDto, PublisherUpdateDto } from '../api/model';

const Publishers = () => {
    const { data: publishers = [], isLoading, error } = useGetAllPublishers();
    const createMutation = useCreatePublisher();
    const updateMutation = useUpdatePublisher();

    const [isModalOpen, setIsModalOpen] = useState(false);
    const [selectedPublisher, setSelectedPublisher] = useState<PublisherResponseDto | undefined>(undefined);

    const handleAddClick = () => {
        setSelectedPublisher(undefined);
        setIsModalOpen(true);
    };

    const handlePublisherClick = (publisher: PublisherResponseDto) => {
        setSelectedPublisher(publisher);
        setIsModalOpen(true);
    };

    const handleModalClose = () => {
        setIsModalOpen(false);
        setSelectedPublisher(undefined);
    };

    const handleModalSubmit = (data: PublisherCreateDto | PublisherUpdateDto) => {
        if (selectedPublisher) {
            updateMutation.mutate(
                { pubId: selectedPublisher.pub_id, data: data as PublisherUpdateDto },
                {
                    onSuccess: () => handleModalClose(),
                }
            );
        } else {
            createMutation.mutate(data as PublisherCreateDto, {
                onSuccess: () => handleModalClose(),
            });
        }
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
                            <h1 className="text-2xl font-bold text-text-main dark:text-white leading-tight">出版社管理</h1>
                            <p className="text-text-sub dark:text-gray-400 text-sm mt-1 text-[13px]">取引先出版社の登録と管理</p>
                        </div>
                        <button
                            onClick={handleAddClick}
                            className="flex items-center justify-center gap-2 bg-primary hover:bg-primary-dark text-white text-[13px] font-bold px-5 py-2.5 rounded-md shadow-sm transition-all hover:scale-[1.02] active:scale-95 focus:ring-2 focus:ring-offset-2 focus:ring-primary"
                        >
                            <span className="material-symbols-outlined text-lg">add</span>
                            <span>出版社を追加</span>
                        </button>
                    </div>

                    <div className="bg-white dark:bg-card-dark rounded-lg border border-border-color dark:border-gray-700 shadow-sm overflow-hidden">
                        <div className="overflow-x-auto">
                            <table className="w-full text-left border-collapse">
                                <thead>
                                    <tr className="bg-gray-100 dark:bg-gray-800 border-b border-border-color dark:border-gray-700">
                                        <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[70%]">
                                            出版社名
                                        </th>
                                        <th className="px-6 py-4 text-text-sub dark:text-gray-400 text-xs font-bold uppercase tracking-wide w-[30%] text-right">
                                            ID (短縮)
                                        </th>
                                    </tr>
                                </thead>
                                <tbody className="divide-y divide-border-color dark:divide-gray-700">
                                    {publishers.map((publisher, index) => {
                                        return (
                                            <tr
                                                key={publisher.pub_id || index}
                                                onClick={() => handlePublisherClick(publisher)}
                                                className="cursor-pointer transition-all group hover:bg-gray-50 dark:hover:bg-gray-800/50"
                                            >
                                                <td className="px-6 py-4 text-sm font-bold text-text-main dark:text-white group-hover:text-primary transition-colors">
                                                    {publisher.name}
                                                </td>
                                                <td className="px-6 py-4 text-xs font-mono text-text-sub dark:text-gray-400 text-right">
                                                    {publisher.pub_id.slice(0, 8)}...
                                                </td>
                                            </tr>
                                        );
                                    })}
                                    {publishers.length === 0 && (
                                        <tr>
                                            <td colSpan={2} className="px-6 py-10 text-center text-text-sub">
                                                出版社が登録されていません。右上のボタンから追加してください。
                                            </td>
                                        </tr>
                                    )}
                                </tbody>
                            </table>
                        </div>
                        <div className="flex items-center justify-between px-6 py-4 border-t border-border-color dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
                            <p className="text-xs text-text-sub dark:text-gray-400">
                                計 <span className="font-bold text-text-main dark:text-white">{publishers.length}</span> 件の出版社
                            </p>
                        </div>
                    </div>
                </div>
            </main>
            <BudgetSummary />

            <PublisherModal
                isOpen={isModalOpen}
                onClose={handleModalClose}
                onSubmit={handleModalSubmit}
                publisher={selectedPublisher}
                isSubmitting={createMutation.isPending || updateMutation.isPending}
            />
        </>
    );
};

export default Publishers;
