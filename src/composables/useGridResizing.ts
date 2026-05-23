// src/composables/useGridResizing.ts
import { ref } from 'vue';

export function useGridResizing(options: {
  minWidth?: number;
  maxWidth?: number;
  defaultWidth?: number;
  onResize?: (colName: string, width: number) => void;
}) {
  const MIN_COLUMN_WIDTH = options.minWidth || 64;
  const MAX_COLUMN_WIDTH = options.maxWidth || 512;
  const DEFAULT_COLUMN_WIDTH = options.defaultWidth || 160;

  const resizing = ref<{ colName: string; startX: number; startWidth: number } | null>(null);

  const onResizeStart = (colName: string, event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    const th = (event.target as HTMLElement).closest('th');
    if (!th) return;

    const table = th.closest('table');
    const startWidth = th.offsetWidth;
    const startTableWidth = table ? table.offsetWidth : 0;
    resizing.value = { colName, startX: event.clientX, startWidth };

    let latestWidth = startWidth;

    const onMouseMove = (e: MouseEvent) => {
      if (!resizing.value) return;
      const delta = e.clientX - resizing.value.startX;
      const newWidth = Math.min(
        MAX_COLUMN_WIDTH,
        Math.max(MIN_COLUMN_WIDTH, resizing.value.startWidth + delta)
      );

      const actualDelta = newWidth - resizing.value.startWidth;
      latestWidth = newWidth;

      if (table) {
        table.style.setProperty(`--col-width-${resizing.value.colName}`, `${newWidth}px`);
        table.style.setProperty(`--table-width`, `${startTableWidth + actualDelta}px`);
      }
    };

    const onMouseUp = () => {
      if (resizing.value) {
        if (table) {
          table.style.removeProperty(`--col-width-${resizing.value.colName}`);
          table.style.removeProperty(`--table-width`);
        }
        if (options.onResize) {
          options.onResize(resizing.value.colName, latestWidth);
        }
      }
      resizing.value = null;
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
    };

    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  };

  return {
    onResizeStart,
    MIN_COLUMN_WIDTH,
    MAX_COLUMN_WIDTH,
    DEFAULT_COLUMN_WIDTH
  };
}
