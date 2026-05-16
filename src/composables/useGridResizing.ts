// src/composables/useGridResizing.ts
import { ref } from 'vue';

export function useGridResizing(options: {
  minWidth?: number;
  maxWidth?: number;
  defaultWidth?: number;
  onResize?: (colName: string, width: number) => void;
}) {
  const MIN_COLUMN_WIDTH = options.minWidth || 80;
  const MAX_COLUMN_WIDTH = options.maxWidth || 320;
  const DEFAULT_COLUMN_WIDTH = options.defaultWidth || 160;

  const resizing = ref<{ colName: string; startX: number; startWidth: number } | null>(null);

  const onResizeStart = (colName: string, event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    const th = (event.target as HTMLElement).closest('th');
    if (!th) return;

    const startWidth = th.offsetWidth;
    resizing.value = { colName, startX: event.clientX, startWidth };

    const onMouseMove = (e: MouseEvent) => {
      if (!resizing.value) return;
      const delta = e.clientX - resizing.value.startX;
      const newWidth = Math.min(
        MAX_COLUMN_WIDTH,
        Math.max(MIN_COLUMN_WIDTH, resizing.value.startWidth + delta)
      );

      if (options.onResize) {
        options.onResize(resizing.value.colName, newWidth);
      }
    };

    const onMouseUp = () => {
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
