import { BridgeService } from '@/services/bridge';
import type { GridColumn, GridRow } from '@/types';
import { ref } from 'vue';

export function useSqlQuery(connectionsStore: any) {
  const sqlColumns = ref<GridColumn[]>([]);
  const sqlRows = ref<GridRow[]>([]);
  const sqlRowCount = ref(0);
  const sqlExecutionTime = ref(0);
  const sqlLimit = ref(200);
  const sqlMessages = ref<Array<{ type: string; text: string; timestamp: string }>>([]);

  const runQuery = async (
    sql: string,
    isLoading: any,
    limit?: number,
    connectionId?: string,
    dbName?: string
  ) => {
    if (!sql || !window.NL_PORT) return;

    const startTime = performance.now();
    sqlMessages.value = [];
    isLoading.value = true;

    try {
      const payload = await BridgeService.request(
        'dbBridge.executeQuery',
        'dbBridge.executeQueryResult',
        {
          connectionId: connectionId || connectionsStore.activeConnectionId,
          sql,
          limit: limit || sqlLimit.value,
          targetDatabase: dbName
        }
      );

      sqlExecutionTime.value = payload.executionTime ?? Math.round(performance.now() - startTime);
      sqlRows.value = payload.rows || [];
      sqlColumns.value = (payload.fields || []).map((f: any) => ({
        name: f.name,
        dataType: String(f.dataTypeID),
        isPrimaryKey: !!f.isPrimaryKey
      }));
      sqlRowCount.value = payload.rowCount || 0;

      sqlMessages.value.push({
        type: 'info',
        text: `Query executed successfully. ${sqlRowCount.value} rows affected.`,
        timestamp: new Date().toISOString()
      });
    } catch (error: any) {
      sqlExecutionTime.value = Math.round(performance.now() - startTime);
      sqlMessages.value.push({
        type: 'error',
        text: `Error: ${error.message}`,
        timestamp: new Date().toISOString()
      });
    } finally {
      isLoading.value = false;
    }
  };

  return {
    sqlColumns,
    sqlRows,
    sqlRowCount,
    sqlExecutionTime,
    sqlLimit,
    sqlMessages,
    runQuery
  };
}
