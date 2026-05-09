import { BridgeService } from '@/services/bridge';
import type { Connection, GridColumn, GridRow } from '@/types';
import { ref } from 'vue';

export function useTableData(connectionsStore: any) {
  const columns = ref<GridColumn[]>([]);
  const rows = ref<GridRow[]>([]);
  const totalRows = ref(0);
  const currentPage = ref(1);
  const rowsPerPage = ref(100);
  const sortColumn = ref<string | undefined>();
  const sortDirection = ref<'asc' | 'desc' | undefined>();
  const executionTime = ref(0);
  const activeTableName = ref('');
  const activeTableSchema = ref('');
  const filterText = ref('');

  const resolveConnection = (connectionId?: string) =>
    connectionsStore.connections.find(
      (conn: Connection) => conn.id === (connectionId || connectionsStore.activeConnectionId)
    ) ?? null;

  const resolveBackendTableName = (
    tableName: string,
    connectionId?: string,
    schemaName?: string
  ) => {
    const connection = resolveConnection(connectionId);
    const targetSchema = schemaName || activeTableSchema.value;

    if (connection?.type === 'oracle' && targetSchema) {
      return `${targetSchema}.${tableName}`;
    }

    return tableName;
  };

  const loadTable = async (
    tableName: string,
    isLoading: any,
    connectionId?: string,
    schemaName?: string,
    dbName?: string
  ) => {
    if (!tableName || !window.NL_PORT) return;

    activeTableName.value = tableName;
    if (schemaName) {
      activeTableSchema.value = schemaName;
    }
    const startTime = performance.now();
    isLoading.value = true;

    const targetConnectionId = connectionId || connectionsStore.activeConnectionId || undefined;
    const backendTableName = resolveBackendTableName(tableName, targetConnectionId, schemaName);

    try {
      const payload = await BridgeService.request(
        'dbBridge.fetchTableData',
        'dbBridge.fetchTableDataResult',
        {
          connectionId: targetConnectionId,
          tableName: backendTableName,
          limit: rowsPerPage.value,
          offset: (currentPage.value - 1) * rowsPerPage.value,
          sortColumn: sortColumn.value,
          sortDirection: sortDirection.value,
          filter: filterText.value,
          targetDatabase: dbName
        }
      );

      rows.value = payload.rows;
      columns.value = payload.fields.map((f: any) => ({
        name: f.name,
        dataType: String(f.dataTypeID),
        isPrimaryKey: !!f.isPrimaryKey
      }));
      totalRows.value = payload.totalCount;
      executionTime.value = payload.executionTime ?? Math.round(performance.now() - startTime);
    } catch (error: any) {
      console.error('Failed to fetch table data:', error.message);
    } finally {
      isLoading.value = false;
    }
  };

  const toggleSort = (colName: string, isLoading: any) => {
    if (sortColumn.value === colName) {
      if (sortDirection.value === 'asc') {
        sortDirection.value = 'desc';
      } else if (sortDirection.value === 'desc') {
        sortColumn.value = undefined;
        sortDirection.value = undefined;
      }
    } else {
      sortColumn.value = colName;
      sortDirection.value = 'asc';
    }
    currentPage.value = 1;
    loadTable(activeTableName.value, isLoading);
  };

  return {
    columns,
    rows,
    totalRows,
    currentPage,
    rowsPerPage,
    sortColumn,
    sortDirection,
    executionTime,
    activeTableName,
    activeTableSchema,
    filterText,
    loadTable,
    toggleSort,
    resolveBackendTableName
  };
}
