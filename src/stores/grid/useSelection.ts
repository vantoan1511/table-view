// src/stores/grid/useSelection.ts
import type { GridColumn, GridRow } from '@/types';
import { ref, type Ref } from 'vue';

export function useSelection(rows: Ref<GridRow[]>) {
  const selectedRowIndices = ref<Set<number>>(new Set());
  const selectedCell = ref<{ rowIndex: number; column: GridColumn } | null>(null);

  const toggleRowSelection = (rowIdx: number, event: MouseEvent) => {
    const newSet = new Set(selectedRowIndices.value);

    if (event.shiftKey && selectedRowIndices.value.size > 0) {
      const lastIdx = Math.max(...selectedRowIndices.value);
      const start = Math.min(lastIdx, rowIdx);
      const end = Math.max(lastIdx, rowIdx);
      for (let i = start; i <= end; i++) {
        newSet.add(i);
      }
    } else if (event.ctrlKey || event.metaKey) {
      if (newSet.has(rowIdx)) {
        newSet.delete(rowIdx);
      } else {
        newSet.add(rowIdx);
      }
    } else {
      newSet.clear();
      newSet.add(rowIdx);
    }

    selectedRowIndices.value = newSet;
  };

  const toggleSelectAllRows = () => {
    if (selectedRowIndices.value.size === rows.value.length) {
      clearSelection();
    } else {
      selectAllRows();
    }
  };

  const clearSelection = () => {
    selectedRowIndices.value = new Set();
  };

  const selectAllRows = () => {
    const newSet = new Set<number>();
    for (let i = 0; i < rows.value.length; i++) {
      newSet.add(i);
    }
    selectedRowIndices.value = newSet;
  };

  const setSelectedCell = (rowIndex: number, column: GridColumn) => {
    selectedCell.value = { rowIndex, column };
  };

  const clearSelectedCell = () => {
    selectedCell.value = null;
  };

  return {
    selectedRowIndices,
    selectedCell,
    toggleRowSelection,
    toggleSelectAllRows,
    clearSelection,
    selectAllRows,
    setSelectedCell,
    clearSelectedCell
  };
}
