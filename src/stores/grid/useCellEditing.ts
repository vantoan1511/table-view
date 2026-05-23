// src/stores/grid/useCellEditing.ts
import { BridgeService } from '@/services/bridge';
import type { GridColumn, GridRow } from '@/types';
import { ref, type Ref } from 'vue';
import { formatGridCellValue, parseGridInputValue } from './valueConversion';

import { useToastStore } from '../toast';

export function useCellEditing(
  rows: Ref<GridRow[]>,
  columns: Ref<GridColumn[]>,
  activeConnectionId: Ref<string | null | undefined>,
  activeTableName: Ref<string>,
  resolveBackendTableName: (name: string) => string,
  activeDbName: Ref<string | undefined>
) {
  const toastStore = useToastStore();
  const editingCell = ref<{
    rowIndex: number;
    column: GridColumn;
    originalValue: any;
    currentValue: any;
  } | null>(null);

  const updateCell = async (rowIndex: number, column: GridColumn, newValue: any) => {
    const row = rows.value[rowIndex];
    if (!row || !window.NL_PORT) return false;

    const parsedValue = parseGridInputValue(newValue, column);
    if (!parsedValue.ok) {
      toastStore.addToast({
        severity: 'error',
        title: 'Input Error',
        message: parsedValue.message
      });
      return false;
    }

    if (row[column.name] === parsedValue.value) return true;

    const pkColumn = columns.value.find((c) => c.isPrimaryKey);
    if (!pkColumn) {
      toastStore.addToast({
        severity: 'error',
        title: 'Update Failed',
        message: 'No primary key found for table ' + activeTableName.value
      });
      return false;
    }

    const pkValue = row[pkColumn.name];

    try {
      await BridgeService.request('dbBridge.updateCell', 'dbBridge.updateCellResult', {
        connectionId: activeConnectionId.value,
        tableName: resolveBackendTableName(activeTableName.value),
        pkColumn: pkColumn.name,
        pkValue,
        targetColumn: column.name,
        newValue: parsedValue.value,
        targetDatabase: activeDbName.value
      });

      row[column.name] = parsedValue.value as GridRow[string];
      return true;
    } catch (error: any) {
      toastStore.addToast({
        severity: 'error',
        title: 'Database Error',
        message: 'Failed to update cell: ' + error.message
      });
      return false;
    }
  };

  const startEditCell = (rowIndex: number, column: GridColumn) => {
    const value = rows.value[rowIndex]?.[column.name];
    editingCell.value = {
      rowIndex,
      column,
      originalValue: value,
      currentValue: formatGridCellValue(value)
    };
  };

  const cancelEditCell = () => {
    editingCell.value = null;
  };

  const saveEditCell = async () => {
    if (!editingCell.value) return;
    const { rowIndex, column, currentValue } = editingCell.value;
    const success = await updateCell(rowIndex, column, currentValue);
    if (success) {
      editingCell.value = null;
    }
  };

  return {
    editingCell,
    updateCell,
    startEditCell,
    cancelEditCell,
    saveEditCell
  };
}
