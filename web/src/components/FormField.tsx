import { useId, type ComponentPropsWithoutRef } from "react";

const controlClassName =
  "w-full px-3 py-2 border border-border-color rounded-md bg-white dark:bg-gray-800 focus:ring-2 focus:ring-primary outline-none";
const labelClassName = "block text-xs font-bold text-text-sub uppercase mb-1";

type InputFieldProps = Omit<ComponentPropsWithoutRef<"input">, "className"> & {
  label: string;
};
type SelectFieldProps = Omit<
  ComponentPropsWithoutRef<"select">,
  "className"
> & { label: string };

export function InputField({ label, id, ...props }: InputFieldProps) {
  const generatedId = useId();
  const fieldId = id ?? generatedId;
  return (
    <div>
      <label htmlFor={fieldId} className={labelClassName}>
        {label}
      </label>
      <input {...props} id={fieldId} className={controlClassName} />
    </div>
  );
}

export function SelectField({ label, id, ...props }: SelectFieldProps) {
  const generatedId = useId();
  const fieldId = id ?? generatedId;
  return (
    <div>
      <label htmlFor={fieldId} className={labelClassName}>
        {label}
      </label>
      <select {...props} id={fieldId} className={controlClassName} />
    </div>
  );
}
