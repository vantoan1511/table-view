import { defineStore } from 'pinia';
import { computed, ref, toRef } from 'vue';

import { BridgeService } from '@/services/bridge';
import { useConnectionsStore } from './connections';
import { useCellEditing } from './grid/useCellEditing';
import { useNewRow } from './grid/useNewRow';
import { useSelection } from './grid/useSelection';
import { useSqlQuery } from './grid/useSqlQuery';
import { useTableData } from './grid/useTableData';

export const useGridStore = defineStore('grid', () => {
  const connectionsStore = useConnectionsStore();
  const isLoading = ref(false);

  // Use sub-composables
  const tableData = useTableData(connectionsStore);
  const sqlQuery = useSqlQuery(connectionsStore);

  // Grid specific state
  const columnWidths = ref<Record<string, number>>({});
  const showAlterTableDialog = ref(false);
  const columnVisibility = ref<Record<string, boolean>>({});

  // Sub-composables for specific features
  const selection = useSelection(tableData.rows);
  const cellEditing = useCellEditing(
    tableData.rows,
    tableData.columns,
    toRef(connectionsStore, 'activeConnectionId'),
    tableData.activeTableName,
    tableData.resolveBackendTableName
  );
  const newRow = useNewRow(
    tableData.rows,
    tableData.columns,
    tableData.totalRows,
    toRef(connectionsStore, 'activeConnectionId'),
    tableData.activeTableName,
    tableData.resolveBackendTableName
  );

  const totalPages = computed(() =>
    Math.max(1, Math.ceil(tableData.totalRows.value / tableData.rowsPerPage.value))
  );

  const setPage = (page: number) => {
    tableData.currentPage.value = page;
    tableData.loadTable(tableData.activeTableName.value, isLoading);
  };

  const setRowsPerPage = (count: number) => {
    tableData.rowsPerPage.value = count;
    tableData.currentPage.value = 1;
    tableData.loadTable(tableData.activeTableName.value, isLoading);
  };

  const toggleSort = (colName: string) => {
    tableData.toggleSort(colName, isLoading);
  };

  const setColumnWidth = (colName: string, width: number) => {
    columnWidths.value = { ...columnWidths.value, [colName]: width };
  };

  const toggleColumnVisibility = (colName: string) => {
    columnVisibility.value[colName] = !columnVisibility.value[colName];
  };

  const deleteRows = async (indices: number[]): Promise<void> => {
    if (!window.NL_PORT) return;
    const pkCol = tableData.columns.value.find((c) => c.isPrimaryKey);
    if (!pkCol) throw new Error('No primary key column found');

    const pkValues = indices
      .map((i) => tableData.rows.value[i]?.[pkCol.name])
      .filter((v) => v !== undefined);
    if (pkValues.length === 0) return;

    await BridgeService.request('dbBridge.deleteRows', 'dbBridge.deleteRowsResult', {
      connectionId: connectionsStore.activeConnectionId,
      tableName: tableData.resolveBackendTableName(tableData.activeTableName.value),
      pkColumn: pkCol.name,
      pkValues
    });

    const idxSet = new Set(indices);
    tableData.rows.value = tableData.rows.value.filter((_, i) => !idxSet.has(i));
    tableData.totalRows.value -= indices.length;
    selection.clearSelection();
  };

  const getTableColumns = async (tableName: string): Promise<any[]> => {
    if (!window.NL_PORT) return [];
    const payload = await BridgeService.request(
      'dbBridge.getTableColumns',
      'dbBridge.getTableColumnsResult',
      {
        connectionId: connectionsStore.activeConnectionId,
        tableName: tableData.resolveBackendTableName(tableName)
      }
    );
    return payload.columns;
  };

  const alterTable = async (tableName: string, operations: any[]): Promise<void> => {
    if (!window.NL_PORT) return;
    await BridgeService.request('dbBridge.alterTable', 'dbBridge.alterTableResult', {
      connectionId: connectionsStore.activeConnectionId,
      tableName: tableData.resolveBackendTableName(tableName),
      operations
    });
  };

  return {
    // Re-export from tableData
    columns: tableData.columns,
    rows: tableData.rows,
    totalRows: tableData.totalRows,
    currentPage: tableData.currentPage,
    rowsPerPage: tableData.rowsPerPage,
    sortColumn: tableData.sortColumn,
    sortDirection: tableData.sortDirection,
    executionTime: tableData.executionTime,
    activeTableName: tableData.activeTableName,
    activeTableSchema: tableData.activeTableSchema,
    filterText: tableData.filterText,
    loadTable: (tableName: string, connectionId?: string, schemaName?: string, dbName?: string) =>
      tableData.loadTable(tableName, isLoading, connectionId, schemaName, dbName),

    // Re-export from sqlQuery
    sqlColumns: sqlQuery.sqlColumns,
    sqlRows: sqlQuery.sqlRows,
    sqlRowCount: sqlQuery.sqlRowCount,
    sqlExecutionTime: sqlQuery.sqlExecutionTime,
    sqlLimit: sqlQuery.sqlLimit,
    sqlMessages: sqlQuery.sqlMessages,
    runQuery: (sql: string, limit?: number, connectionId?: string, dbName?: string) =>
      sqlQuery.runQuery(sql, isLoading, limit, connectionId, dbName),

    // Grid specific
    isLoading,
    totalPages,
    columnWidths,
    setPage,
    setRowsPerPage,
    toggleSort,
    setColumnWidth,
    showAlterTableDialog,
    columnVisibility,
    toggleColumnVisibility,

    // Selection
    selectedRowIndices: selection.selectedRowIndices,
    selectedCell: selection.selectedCell,
    toggleRowSelection: selection.toggleRowSelection,
    toggleSelectAllRows: selection.toggleSelectAllRows,
    clearSelection: selection.clearSelection,
    selectAllRows: selection.selectAllRows,
    setSelectedCell: selection.setSelectedCell,
    clearSelectedCell: selection.clearSelectedCell,

    // Cell Editing
    editingCell: cellEditing.editingCell,
    updateCell: cellEditing.updateCell,
    startEditCell: cellEditing.startEditCell,
    cancelEditCell: cellEditing.cancelEditCell,
    saveEditCell: cellEditing.saveEditCell,

    // New Row
    newRowIdx: newRow.newRowIdx,
    newRowData: newRow.newRowData,
    createNewRow: newRow.createNewRow,
    cancelNewRow: newRow.cancelNewRow,
    saveNewRow: newRow.saveNewRow,
    insertRow: newRow.insertRowToDB, // Keep original name for compatibility if needed, or rename

    // Actions
    deleteRows,
    getTableColumns,
    alterTable
  };
});
