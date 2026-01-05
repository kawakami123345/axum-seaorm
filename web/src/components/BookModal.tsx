import { useState, useEffect } from 'react';
import type { BookResponseDto, BookCreateDto, BookUpdateDto } from '../api/model';
import { useGetAllPublishers } from '../api/endpoints/publisher';
import { useGetAllShops } from '../api/endpoints/shop';

interface BookModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSubmit: (data: BookCreateDto | BookUpdateDto) => void;
    book?: BookResponseDto;
    isSubmitting?: boolean;
}

const BookModal = ({ isOpen, onClose, onSubmit, book, isSubmitting }: BookModalProps) => {
    const { data: publishers = [], isLoading: isLoadingPublishers } = useGetAllPublishers();
    const { data: shops = [] } = useGetAllShops();
    const [errorMsg, setErrorMsg] = useState<string | null>(null);

    const [formData, setFormData] = useState<BookCreateDto>({
        title: '',
        author: '',
        price: 0,
        publisher_id: '',
        format: 'Real',
        shop_id: null,
    });

    useEffect(() => {
        setErrorMsg(null);
        if (book) {
            setFormData({
                title: book.title,
                author: book.author,
                price: book.price,
                publisher_id: book.publisher.pub_id,
                format: book.format || 'Real',
                shop_id: book.shop?.pub_id || null,
            });
        } else {
            setFormData({
                title: '',
                author: '',
                price: 0,
                publisher_id: '',
                format: 'Real',
                shop_id: null,
            });
        }
    }, [book, isOpen]);

    if (!isOpen) return null;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        console.log('Submitting book data:', formData);
        if (!formData.publisher_id) {
            setErrorMsg('出版社を選択してください。');
            return;
        }
        onSubmit(formData);
    };

    return (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
            <div className="bg-white dark:bg-card-dark w-full max-w-md rounded-lg shadow-xl overflow-hidden animate-in fade-in zoom-in duration-200">
                <div className="px-6 py-4 border-b border-border-color dark:border-gray-700 flex justify-between items-center">
                    <h2 className="text-lg font-bold text-text-main dark:text-white">
                        {book ? '書籍情報を編集' : '新しい書籍を追加'}
                    </h2>
                    <button onClick={onClose} className="text-text-sub hover:text-text-main">
                        <span className="material-symbols-outlined">close</span>
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="p-6 flex flex-col gap-4">
                    {errorMsg && (
                        <div className="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded text-red-600 dark:text-red-400 text-xs font-bold flex items-center gap-2">
                            <span className="material-symbols-outlined text-sm">error</span>
                            {errorMsg}
                        </div>
                    )}

                    {!isLoadingPublishers && publishers.length === 0 && (
                        <div className="p-3 bg-amber-50 dark:bg-amber-900/30 border border-amber-200 dark:border-amber-800 rounded text-amber-700 dark:text-amber-400 text-xs font-bold leading-normal">
                            出版社が登録されていません。先に「出版社管理」から出版社を登録してください。
                        </div>
                    )}
                    <div>
                        <label className="block text-xs font-bold text-text-sub uppercase mb-1">タイトル</label>
                        <input
                            required
                            className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                            value={formData.title}
                            onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                        />
                    </div>

                    <div>
                        <label className="block text-xs font-bold text-text-sub uppercase mb-1">著者</label>
                        <input
                            required
                            className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                            value={formData.author}
                            onChange={(e) => setFormData({ ...formData, author: e.target.value })}
                        />
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <label className="block text-xs font-bold text-text-sub uppercase mb-1">価格</label>
                            <input
                                type="number"
                                required
                                className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                                value={formData.price}
                                onChange={(e) => setFormData({ ...formData, price: parseInt(e.target.value) || 0 })}
                            />
                        </div>
                        <div>
                            <label className="block text-xs font-bold text-text-sub uppercase mb-1">形式</label>
                            <select
                                className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                                value={formData.format || 'Real'}
                                onChange={(e) => setFormData({ ...formData, format: e.target.value })}
                            >
                                <option value="Real">単行本/文庫</option>
                                <option value="Kindle">Kindle/電子書籍</option>
                            </select>
                        </div>
                    </div>

                    <div>
                        <label className="block text-xs font-bold text-text-sub uppercase mb-1">出版社</label>
                        <select
                            required
                            className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                            value={formData.publisher_id}
                            onChange={(e) => setFormData({ ...formData, publisher_id: e.target.value })}
                        >
                            <option value="" disabled>出版社を選択してください</option>
                            {publishers.map((p) => (
                                <option key={p.pub_id} value={p.pub_id}>{p.name}</option>
                            ))}
                        </select>
                    </div>

                    <div>
                        <label className="block text-xs font-bold text-text-sub uppercase mb-1">購入店舗 (任意)</label>
                        <select
                            className="w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none"
                            value={formData.shop_id || ''}
                            onChange={(e) => setFormData({ ...formData, shop_id: e.target.value || null })}
                        >
                            <option value="">未指定</option>
                            {shops.map((s) => (
                                <option key={s.pub_id} value={s.pub_id}>{s.name}</option>
                            ))}
                        </select>
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
                            {book ? '更新する' : '保存する'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
};

export default BookModal;
