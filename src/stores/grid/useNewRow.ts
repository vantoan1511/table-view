// src/stores/grid/useNewRow.ts
import { BridgeService } from '@/services/bridge';
import type { GridColumn, GridRow } from '@/types';
import { ref, type Ref } from 'vue';
import { useToastStore } from '../toast';
import { parseGridInputValue } from './valueConversion';

export function useNewRow(
  rows: Ref<GridRow[]>,
  columns: Ref<GridColumn[]>,
  totalRows: Ref<number>,
  activeConnectionId: Ref<string | null>,
  activeTableName: Ref<string>,
  resolveBackendTableName: (name: string) => string
) {
  const toastStore = useToastStore();
  const newRowIdx = ref<number | null>(null);
  const newRowData = ref<Record<string, string>>({});

  const createNewRow = () => {
    const placeholder: GridRow = {};
    for (const col of columns.value) {
      placeholder[col.name] = '';
    }
    rows.value = [placeholder, ...rows.value];
    totalRows.value += 1;
    newRowIdx.value = 0;
    const data: Record<string, string> = {};
    for (const col of columns.value) {
      data[col.name] = '';
    }
    newRowData.value = data;
  };

  const cancelNewRow = () => {
    if (newRowIdx.value === null) return;
    rows.value.splice(newRowIdx.value, 1);
    totalRows.value -= 1;
    newRowIdx.value = null;
    newRowData.value = {};
  };

  const saveNewRow = async () => {
    if (newRowIdx.value === null) return;
    try {
      const cleanData: Record<string, any> = {};
      for (const col of columns.value) {
        const val = newRowData.value[col.name];
        const isEmpty = !val || val === '';

        if (isEmpty) {
          const type = col.dataType ? col.dataType.toLowerCase() : '';
          const isUuid = type.includes('uuid') || type === '2950';
          const isStringPk =
            col.isPrimaryKey &&
            (['1043', '25', '1042'].includes(type) ||
              type.includes('char') ||
              type.includes('text'));
          if (isUuid || isStringPk) {
            cleanData[col.name] = crypto.randomUUID();
          } else if (col.isPrimaryKey) {
            continue;
          } else {
            const parsed = parseGridInputValue(val, col);
            if (!parsed.ok) throw new Error(parsed.message);
            cleanData[col.name] = parsed.value;
          }
        } else {
          const parsed = parseGridInputValue(val, col);
          if (!parsed.ok) throw new Error(parsed.message);
          cleanData[col.name] = parsed.value;
        }
      }

      const savedRow = await insertRowToDB(cleanData);
      rows.value.splice(newRowIdx.value, 1, savedRow);
      newRowIdx.value = null;
      newRowData.value = {};
    } catch (err: any) {
      toastStore.addToast({
        severity: 'error',
        title: 'Insert Failed',
        message: err.message
      });
      throw err;
    }
  };

  const insertRowToDB = async (data: Record<string, any> = {}): Promise<GridRow> => {
    if (!window.NL_PORT) return {};
    const payload = await BridgeService.request('dbBridge.insertRow', 'dbBridge.insertRowResult', {
      connectionId: activeConnectionId.value,
      tableName: resolveBackendTableName(activeTableName.value),
      data
    });
    return payload.row;
  };

  return {
    newRowIdx,
    newRowData,
    createNewRow,
    cancelNewRow,
    saveNewRow,
    insertRowToDB
  };
}
