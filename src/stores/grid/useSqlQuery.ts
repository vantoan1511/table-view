import { useHistoryStore } from '../history';
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
    dbName?: string,
    autoCommit = true
  ) => {
    if (!sql || !window.NL_PORT) return;

    const startTime = performance.now();
    sqlMessages.value = [];
    isLoading.value = true;

    try {
      const eventName = autoCommit ? 'dbBridge.executeQuery' : 'dbBridge.executeTransaction';
      const resultEventName = autoCommit
        ? 'dbBridge.executeQueryResult'
        : 'dbBridge.executeTransactionResult';

      const payload = await BridgeService.request(eventName, resultEventName, {
        connectionId: connectionId || connectionsStore.activeConnectionId,
        sql,
        limit: limit || sqlLimit.value,
        targetDatabase: dbName
      });

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

      const historyStore = useHistoryStore();
      historyStore.addSqlEntry({
        query: sql,
        connectionId: connectionId || connectionsStore.activeConnectionId,
        dbName,
        success: true,
        executionTime: sqlExecutionTime.value,
        rowCount: sqlRowCount.value
      });
      return true;
    } catch (error: any) {
      sqlExecutionTime.value = Math.round(performance.now() - startTime);
      sqlMessages.value.push({
        type: 'error',
        text: `Error: ${error.message}`,
        timestamp: new Date().toISOString()
      });

      const historyStore = useHistoryStore();
      historyStore.addSqlEntry({
        query: sql,
        connectionId: connectionId || connectionsStore.activeConnectionId,
        dbName,
        success: false,
        executionTime: sqlExecutionTime.value,
        error: error.message || String(error)
      });
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  const commitTransaction = async (connectionId?: string, dbName?: string) => {
    if (!window.NL_PORT) return;
    try {
      await BridgeService.request(
        'dbBridge.commitTransaction',
        'dbBridge.commitTransactionResult',
        {
          connectionId: connectionId || connectionsStore.activeConnectionId,
          targetDatabase: dbName
        }
      );
      sqlMessages.value.push({
        type: 'info',
        text: 'Transaction committed successfully.',
        timestamp: new Date().toISOString()
      });
    } catch (error: any) {
      sqlMessages.value.push({
        type: 'error',
        text: `Commit Error: ${error.message}`,
        timestamp: new Date().toISOString()
      });
      throw error;
    }
  };

  const rollbackTransaction = async (connectionId?: string, dbName?: string) => {
    if (!window.NL_PORT) return;
    try {
      await BridgeService.request(
        'dbBridge.rollbackTransaction',
        'dbBridge.rollbackTransactionResult',
        {
          connectionId: connectionId || connectionsStore.activeConnectionId,
          targetDatabase: dbName
        }
      );
      sqlMessages.value.push({
        type: 'info',
        text: 'Transaction rolled back successfully.',
        timestamp: new Date().toISOString()
      });
    } catch (error: any) {
      sqlMessages.value.push({
        type: 'error',
        text: `Rollback Error: ${error.message}`,
        timestamp: new Date().toISOString()
      });
      throw error;
    }
  };

  return {
    sqlColumns,
    sqlRows,
    sqlRowCount,
    sqlExecutionTime,
    sqlLimit,
    sqlMessages,
    runQuery,
    commitTransaction,
    rollbackTransaction
  };
}
