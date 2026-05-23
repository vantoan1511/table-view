import { BridgeService } from '@/services/bridge';
import type { Connection, GridColumn, GridRow } from '@/types';
import { ref } from 'vue';

import { useToastStore } from '../toast';
import { useTabsStore } from '../tabs';

export function useTableData(connectionsStore: any) {
  const toastStore = useToastStore();
  const tabsStore = useTabsStore();
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
  const activeConnectionId = ref<string | undefined>();
  const activeDbName = ref<string | undefined>();
  const filterText = ref('');
  const currentTabId = ref<string>('');

  const resolveConnection = (connectionId?: string) =>
    connectionsStore.connections.find(
      (conn: Connection) => conn.id === (connectionId || connectionsStore.activeConnectionId)
    ) ?? null;

  const resolveBackendTableName = (
    tableName: string,
    connectionId?: string,
    schemaName?: string
  ) => {
    const targetConnectionId = connectionId || activeConnectionId.value;
    const connection = resolveConnection(targetConnectionId);
    const targetSchema = schemaName || activeTableSchema.value;

    if (['oracle', 'postgres', 'postgresql'].includes(connection?.type ?? '') && targetSchema) {
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

    const activeTab = tabsStore.activeTab;
    const targetConnectionId =
      connectionId || activeConnectionId.value || connectionsStore.activeConnectionId || undefined;
    const targetDbName = dbName || activeDbName.value || undefined;

    const isNewTab = activeTab ? activeTab.id !== currentTabId.value : currentTabId.value !== '';

    const isSameTab =
      activeTab &&
      activeTab.type === 'table' &&
      activeTab.tableName === tableName &&
      activeTab.connectionId === targetConnectionId &&
      activeTab.schema === schemaName &&
      activeTab.dbName === targetDbName;

    if (isNewTab) {
      if (isSameTab && activeTab.sortColumn !== undefined) {
        sortColumn.value = activeTab.sortColumn;
        sortDirection.value = activeTab.sortDirection;
        currentPage.value = activeTab.currentPage ?? 1;
        filterText.value = activeTab.filterText ?? '';
      } else {
        sortColumn.value = undefined;
        sortDirection.value = undefined;
        currentPage.value = 1;
        filterText.value = '';
      }
      currentTabId.value = activeTab?.id ?? '';
    }

    if (isSameTab) {
      activeTab.sortColumn = sortColumn.value;
      activeTab.sortDirection = sortDirection.value;
      activeTab.currentPage = currentPage.value;
      activeTab.filterText = filterText.value;
    }

    activeTableName.value = tableName;
    if (schemaName) {
      activeTableSchema.value = schemaName;
    }
    if (targetConnectionId) {
      activeConnectionId.value = targetConnectionId;
    }
    if (targetDbName) {
      activeDbName.value = targetDbName;
    }

    const startTime = performance.now();
    isLoading.value = true;

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
          targetDatabase: targetDbName
        }
      );

      rows.value = payload.rows;
      columns.value = payload.fields.map((f: any) => ({
        name: f.name,
        dataType: String(f.dataTypeID),
        isPrimaryKey: !!f.isPrimaryKey,
        ...(f.isNullable !== undefined ? { isNullable: !!f.isNullable } : {})
      }));
      totalRows.value = payload.totalCount;
      executionTime.value = payload.executionTime ?? Math.round(performance.now() - startTime);
    } catch (error: any) {
      console.error('Failed to fetch table data:', error.message);
      toastStore.addToast({
        severity: 'error',
        title: 'Failed to Load Table',
        message: error.message || 'An unknown error occurred while loading table data.'
      });
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
    activeConnectionId,
    activeDbName,
    filterText,
    loadTable,
    toggleSort,
    resolveBackendTableName
  };
}
