import { useState, useEffect } from 'react';
import type { PublisherResponseDto, PublisherCreateDto, PublisherUpdateDto } from '../api/model';

interface PublisherModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSubmit: (data: PublisherCreateDto | PublisherUpdateDto) => void;
    publisher?: PublisherResponseDto;
    isSubmitting?: boolean;
}

const PublisherModal = ({ isOpen, onClose, onSubmit, publisher, isSubmitting }: PublisherModalProps) => {
    const [formData, setFormData] = useState<PublisherCreateDto>({
        name: '',
    });

    useEffect(() => {
        if (publisher) {
            setFormData({
                name: publisher.name,
            });
        } else {
            setFormData({
                name: '',
            });
        }
    }, [publisher, isOpen]);

    if (!isOpen) return null;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        onSubmit(formData);
    };

    return (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
            <div className="bg-white dark:bg-card-dark w-full max-w-md rounded-lg shadow-xl overflow-hidden animate-in fade-in zoom-in duration-200">
                <div className="px-6 py-4 border-b border-border-color dark:border-gray-700 flex justify-between items-center">
                    <h2 className="text-lg font-bold text-text-main dark:text-white">
                        {publisher ? '出版社情報を編集' : '新しい出版社を追加'}
                    </h2>
                    <button onClick={onClose} className="text-text-sub hover:text-text-main">
                        <span className="material-symbols-outlined">close</span>
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="p-6 flex flex-col gap-4">
                    <div>
                        <label className="block text-xs font-bold text-text-sub uppercase mb-1">出版社名</label>
                        <input
                            required
                            autoFocus
                            placeholder="例: デジタル出版株式会社"
                            className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                            value={formData.name}
                            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        />
                    </div>

                    <div className="mt-4 flex gap-3">
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex-1 px-4 py-2 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-text-main dark:text-white font-bold rounded-md transition-colors"
                        >
                            キャンセル
                        </button>
                        <button
                            type="submit"
                            disabled={isSubmitting}
                            className="flex-1 px-4 py-2 bg-primary hover:bg-primary-dark text-white font-bold rounded-md shadow-sm transition-colors flex items-center justify-center gap-2"
                        >
                            {isSubmitting && <span className="material-symbols-outlined animate-spin text-sm">sync</span>}
                            {publisher ? '更新する' : '保存する'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
};

export default PublisherModal;
