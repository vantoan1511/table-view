import { useCellEditing } from './grid/useCellEditing';
import { useNewRow } from './grid/useNewRow';
import { useSelection } from './grid/useSelection';
import { useSqlQuery } from './grid/useSqlQuery';
import { useTableData } from './grid/useTableData';

import { useConnectionsStore } from './connections';
import { useSchemaStore } from './schema';
import { useTabsStore } from './tabs';
import { useToastStore } from './toast';

import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

import { BridgeService } from '@/services/bridge';

export interface TableColumn {
  name: string;
  dataType: string;
  nullable: boolean;
  isPrimaryKey: boolean;
  default?: string;
  foreignKey?: {
    targetTable: string;
    targetColumn: string;
  };
}

export interface TableConstraint {
  name: string;
  constraintType: string;
  definition: string;
}

export const useGridStore = defineStore('grid', () => {
  const connectionsStore = useConnectionsStore();
  const schemaStore = useSchemaStore();
  const tabsStore = useTabsStore();
  const toastStore = useToastStore();
  const isLoading = ref(false);

  // Use sub-composables
  const tableData = useTableData(connectionsStore);
  const sqlQuery = useSqlQuery(connectionsStore);

  // Grid specific state
  const showAlterTableDialog = ref(false);
  const showCreateTableDialog = ref(false);
  const showCreateSchemaDialog = ref(false);
  const showCreateDatabaseDialog = ref(false);
  const createTableTarget = ref<{ connectionId: string; schema: string; db?: string } | null>(null);
  const createSchemaTarget = ref<{ connectionId: string; db?: string } | null>(null);
  const createDatabaseTarget = ref<{ connectionId: string } | null>(null);
  const columnVisibility = ref<Record<string, boolean>>({});

  // Sub-composables for specific features
  const selection = useSelection(tableData.rows);
  const cellEditing = useCellEditing(
    tableData.rows,
    tableData.columns,
    tableData.activeConnectionId,
    tableData.activeTableName,
    tableData.resolveBackendTableName,
    tableData.activeDbName
  );
  const newRow = useNewRow(
    tableData.rows,
    tableData.columns,
    tableData.totalRows,
    tableData.activeConnectionId,
    tableData.activeTableName,
    tableData.resolveBackendTableName,
    tableData.activeDbName
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

    try {
      isLoading.value = true;
      await BridgeService.request('dbBridge.deleteRows', 'dbBridge.deleteRowsResult', {
        connectionId: tableData.activeConnectionId.value || connectionsStore.activeConnectionId,
        tableName: tableData.resolveBackendTableName(tableData.activeTableName.value),
        pkColumn: pkCol.name,
        pkValues,
        targetDatabase: tableData.activeDbName.value
      });

      const idxSet = new Set(indices);
      tableData.rows.value = tableData.rows.value.filter((_, i) => !idxSet.has(i));
      // Use pkValues.length instead of indices.length to avoid out-of-sync count
      tableData.totalRows.value -= pkValues.length;
      selection.clearSelection();

      toastStore.addToast({
        severity: 'success',
        title: 'Rows Deleted',
        message: `Successfully deleted ${pkValues.length} row(s).`
      });
    } catch (err: any) {
      console.error('Failed to delete rows:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Delete Failed',
        message: err.message || 'An unknown error occurred while deleting rows.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const getTableColumns = async (tableName: string): Promise<TableColumn[]> => {
    if (!window.NL_PORT) return [];
    try {
      isLoading.value = true;
      const payload = await BridgeService.request(
        'dbBridge.getTableColumns',
        'dbBridge.getTableColumnsResult',
        {
          connectionId: tableData.activeConnectionId.value || connectionsStore.activeConnectionId,
          tableName: tableData.resolveBackendTableName(tableName),
          targetDatabase: tableData.activeDbName.value
        }
      );
      return payload.columns;
    } catch (err: any) {
      console.error('Failed to get table columns:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Fetch Columns Failed',
        message: err.message || 'An unknown error occurred while fetching table columns.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const getTableConstraints = async (tableName: string): Promise<TableConstraint[]> => {
    if (!window.NL_PORT) return [];
    try {
      isLoading.value = true;
      const payload = await BridgeService.request(
        'dbBridge.getTableConstraints',
        'dbBridge.getTableConstraintsResult',
        {
          connectionId: tableData.activeConnectionId.value || connectionsStore.activeConnectionId,
          tableName: tableData.resolveBackendTableName(tableName),
          targetDatabase: tableData.activeDbName.value
        }
      );
      return payload.constraints || [];
    } catch (err: any) {
      console.error('Failed to get table constraints:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Fetch Constraints Failed',
        message: err.message || 'An unknown error occurred while fetching table constraints.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const alterTable = async (tableName: string, operations: any[]): Promise<void> => {
    if (!window.NL_PORT) return;
    try {
      isLoading.value = true;
      await BridgeService.request('dbBridge.alterTable', 'dbBridge.alterTableResult', {
        connectionId: tableData.activeConnectionId.value || connectionsStore.activeConnectionId,
        tableName: tableData.resolveBackendTableName(tableName),
        operations,
        targetDatabase: tableData.activeDbName.value
      });
    } catch (err: any) {
      console.error('Failed to alter table:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Alter Table Failed',
        message: err.message || 'An unknown error occurred while altering table.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const dropTable = async (
    tableName: string,
    connectionId?: string,
    schemaName?: string,
    dbName?: string,
    cascade?: boolean
  ): Promise<void> => {
    if (!window.NL_PORT) return;
    const targetConnectionId = connectionId || connectionsStore.activeConnectionId;

    try {
      isLoading.value = true;
      await BridgeService.request('dbBridge.dropTable', 'dbBridge.dropTableResult', {
        connectionId: targetConnectionId!,
        tableName: tableData.resolveBackendTableName(
          tableName,
          targetConnectionId!,
          schemaName || undefined
        ),
        targetDatabase: dbName,
        cascade: !!cascade
      });

      // Close associated tabs
      tabsStore.tabs
        .filter(
          (t) =>
            t.type === 'table' &&
            t.tableName === tableName &&
            t.connectionId === targetConnectionId &&
            t.schema === (schemaName || t.schema) &&
            t.dbName === (dbName || t.dbName)
        )
        .forEach((t) => tabsStore.closeTab(t.id));

      // Refresh schema
      if (dbName) {
        await schemaStore.refreshDbSchema(targetConnectionId!, dbName);
      } else if (targetConnectionId) {
        await schemaStore.loadSchema(undefined, targetConnectionId);
      }

      toastStore.addToast({
        severity: 'success',
        title: 'Table Dropped',
        message: `Table ${tableName} was successfully dropped.`
      });
    } catch (err: any) {
      console.error('Failed to drop table:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Drop Failed',
        message: err.message || 'An unknown error occurred while dropping the table.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const createTable = async (
    tableName: string,
    columns: TableColumn[],
    connectionId: string,
    schemaName: string,
    dbName?: string
  ): Promise<void> => {
    if (!window.NL_PORT) return;

    try {
      isLoading.value = true;
      await BridgeService.request('dbBridge.createTable', 'dbBridge.createTableResult', {
        connectionId,
        tableName: tableData.resolveBackendTableName(tableName, connectionId, schemaName),
        targetDatabase: dbName,
        columns
      });

      // Refresh schema
      if (dbName) {
        await schemaStore.refreshDbSchema(connectionId, dbName);
      } else {
        await schemaStore.loadSchema(undefined, connectionId, schemaName);
      }

      toastStore.addToast({
        severity: 'success',
        title: 'Table Created',
        message: `Table ${tableName} was successfully created.`
      });
    } catch (err: any) {
      console.error('Failed to create table:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Creation Failed',
        message: err.message || 'An unknown error occurred while creating the table.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const createSchema = async (
    connectionId: string,
    schemaName: string,
    dbName?: string
  ): Promise<void> => {
    if (!window.NL_PORT) return;

    try {
      isLoading.value = true;
      await BridgeService.request('dbBridge.createSchema', 'dbBridge.createSchemaResult', {
        connectionId,
        schemaName,
        targetDatabase: dbName
      });

      // Refresh schema
      if (dbName) {
        await schemaStore.refreshDbSchema(connectionId, dbName);
      } else {
        await schemaStore.loadSchema(undefined, connectionId);
      }

      toastStore.addToast({
        severity: 'success',
        title: 'Schema Created',
        message: `Schema ${schemaName} was successfully created.`
      });
    } catch (err: any) {
      console.error('Failed to create schema:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Creation Failed',
        message: err.message || 'An unknown error occurred while creating the schema.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const createDatabase = async (
    connectionId: string,
    dbName: string,
    password?: string
  ): Promise<void> => {
    if (!window.NL_PORT) return;

    try {
      isLoading.value = true;
      await BridgeService.request('dbBridge.createDatabase', 'dbBridge.createDatabaseResult', {
        connectionId,
        dbName,
        password
      });

      // Refresh schema
      await schemaStore.loadSchema(undefined, connectionId);

      toastStore.addToast({
        severity: 'success',
        title: 'Database Created',
        message: `Database ${dbName} was successfully created.`
      });
    } catch (err: any) {
      console.error('Failed to create database:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Creation Failed',
        message: err.message || 'An unknown error occurred while creating the database.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const dropSchema = async (
    connectionId: string,
    schemaName: string,
    dbName?: string
  ): Promise<void> => {
    if (!window.NL_PORT) return;

    try {
      isLoading.value = true;
      await BridgeService.request('dbBridge.dropSchema', 'dbBridge.dropSchemaResult', {
        connectionId,
        schemaName,
        targetDatabase: dbName
      });

      // Refresh schema
      if (dbName) {
        await schemaStore.refreshDbSchema(connectionId, dbName);
      } else {
        await schemaStore.loadSchema(undefined, connectionId);
      }

      toastStore.addToast({
        severity: 'success',
        title: 'Schema Dropped',
        message: `Schema ${schemaName} was successfully dropped.`
      });
    } catch (err: any) {
      console.error('Failed to drop schema:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Drop Failed',
        message: err.message || 'An unknown error occurred while dropping the schema.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const dropDatabase = async (connectionId: string, dbName: string): Promise<void> => {
    if (!window.NL_PORT) return;

    try {
      isLoading.value = true;
      await BridgeService.request('dbBridge.dropDatabase', 'dbBridge.dropDatabaseResult', {
        connectionId,
        dbName: dbName
      });

      // Close associated tabs
      tabsStore.tabs
        .filter((t) => t.connectionId === connectionId && t.dbName === dbName)
        .forEach((t) => tabsStore.closeTab(t.id));

      if (tableData.activeDbName.value === dbName) {
        tableData.activeDbName.value = undefined;
      }

      // Explicitly clear database schemas/caches in schemaStore
      schemaStore.clearDbSchema(connectionId, dbName);

      // Check if the database being dropped was the active configured database for the connection
      const conn = connectionsStore.connections.find((c) => c.id === connectionId);
      if (conn && conn.database === dbName) {
        const availableDbs = schemaStore.schemasByConnection[connectionId]?.databases || [];
        const nextDb =
          availableDbs.find((d) => d !== dbName) ||
          (['postgres', 'postgresql'].includes(conn.type) ? 'postgres' : 'mysql');
        // Update connection with fallback database, which automatically triggers reconnection & schema refresh
        await connectionsStore.updateConnection(connectionId, { database: nextDb });
      } else {
        // Refresh schema if not the active configured database
        await schemaStore.loadSchema(undefined, connectionId);
      }

      toastStore.addToast({
        severity: 'success',
        title: 'Database Dropped',
        message: `Database ${dbName} was successfully dropped.`
      });
    } catch (err: any) {
      console.error('Failed to drop database:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Drop Failed',
        message: err.message || 'An unknown error occurred while dropping the database.'
      });
      throw err;
    } finally {
      isLoading.value = false;
    }
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
    activeConnectionId: tableData.activeConnectionId,
    activeDbName: tableData.activeDbName,
    filterText: tableData.filterText,
    loadTable: (
      tableName: string,
      connectionId?: string,
      schemaName?: string,
      dbName?: string,
      forceRefresh?: boolean
    ) => tableData.loadTable(tableName, isLoading, connectionId, schemaName, dbName, forceRefresh),

    // Re-export from sqlQuery
    sqlColumns: sqlQuery.sqlColumns,
    sqlRows: sqlQuery.sqlRows,
    sqlRowCount: sqlQuery.sqlRowCount,
    sqlExecutionTime: sqlQuery.sqlExecutionTime,
    sqlLimit: sqlQuery.sqlLimit,
    sqlMessages: sqlQuery.sqlMessages,
    runQuery: (
      sql: string,
      limit?: number,
      connectionId?: string,
      dbName?: string,
      autoCommit = true
    ) => sqlQuery.runQuery(sql, isLoading, limit, connectionId, dbName, autoCommit),
    commitTransaction: (connectionId?: string, dbName?: string) =>
      sqlQuery.commitTransaction(connectionId, dbName),
    rollbackTransaction: (connectionId?: string, dbName?: string) =>
      sqlQuery.rollbackTransaction(connectionId, dbName),

    // Grid specific
    isLoading,
    totalPages,
    columnWidths: tableData.columnWidths,
    setPage,
    setRowsPerPage,
    toggleSort,
    setColumnWidth: tableData.setColumnWidth,
    autoDistributeColumnWidths: tableData.autoDistributeColumnWidths,
    showAlterTableDialog,
    showCreateTableDialog,
    showCreateSchemaDialog,
    showCreateDatabaseDialog,
    createTableTarget,
    createSchemaTarget,
    createDatabaseTarget,
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
    newRowErrors: newRow.newRowErrors,
    createNewRow: newRow.createNewRow,
    cancelNewRow: newRow.cancelNewRow,
    saveNewRow: newRow.saveNewRow,
    validateNewRowCell: newRow.validateNewRowCell,
    insertRow: newRow.insertRowToDB, // Keep original name for compatibility if needed, or rename

    // Actions
    deleteRows,
    getTableColumns,
    getTableConstraints,
    alterTable,
    dropTable,
    dropSchema,
    dropDatabase,
    createTable,
    createDatabase,
    createSchema
  };
});
