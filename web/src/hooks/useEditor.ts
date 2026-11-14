import { useState } from "react";

export function useEditor<T>() {
  const [editor, setEditor] = useState<{ item?: T } | null>(null);

  return {
    isOpen: editor !== null,
    item: editor?.item,
    openCreate: () => setEditor({}),
    openEdit: (item: T) => setEditor({ item }),
    close: () => setEditor(null),
  };
}
