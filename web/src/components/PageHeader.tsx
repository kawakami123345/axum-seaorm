import Icon from "./Icon";

interface PageHeaderProps {
  title: string;
  description: string;
  createLabel: string;
  onCreate: () => void;
}

export default function PageHeader({
  title,
  description,
  createLabel,
  onCreate,
}: PageHeaderProps) {
  return (
    <div className="flex flex-wrap items-end justify-between gap-4 border-b border-border-color pb-4">
      <div>
        <h1 className="text-2xl font-bold text-text-main dark:text-white leading-tight">
          {title}
        </h1>
        <p className="text-text-sub dark:text-gray-400 mt-1 text-[13px]">
          {description}
        </p>
      </div>
      <button
        type="button"
        onClick={onCreate}
        className="flex items-center justify-center gap-2 bg-primary hover:bg-primary-dark text-white text-[13px] font-bold px-5 py-2.5 rounded-md shadow-sm transition-all hover:scale-[1.02] active:scale-95 focus:ring-2 focus:ring-offset-2 focus:ring-primary"
      >
        <Icon name="add" className="text-lg" />
        <span>{createLabel}</span>
      </button>
    </div>
  );
}
