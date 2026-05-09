import type { GridColumn, GridRow } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { useConnectionsStore } from './connections'
import { useTableData } from './grid/useTableData'
import { useSqlQuery } from './grid/useSqlQuery'
import { BridgeService } from '@/services/bridge'

export const useGridStore = defineStore('grid', () => {
  const connectionsStore = useConnectionsStore()
  const isLoading = ref(false)

  // Use sub-composables
  const tableData = useTableData(connectionsStore)
  const sqlQuery = useSqlQuery(connectionsStore)

  // Row selection state
  const selectedRowIndices = ref<Set<number>>(new Set())

  // Cell selection state
  const selectedCell = ref<{ rowIndex: number; column: GridColumn } | null>(null)
  const editingCell = ref<{ rowIndex: number; column: GridColumn; originalValue: any; currentValue: any } | null>(null)

  // Column widths state (column name -> width in px)
  const columnWidths = ref<Record<string, number>>({})

  const totalPages = computed(() => 
    Math.max(1, Math.ceil(tableData.totalRows.value / tableData.rowsPerPage.value))
  )

  const setPage = (page: number) => {
    tableData.currentPage.value = page
    tableData.loadTable(tableData.activeTableName.value, isLoading)
  }

  const setRowsPerPage = (count: number) => {
    tableData.rowsPerPage.value = count
    tableData.currentPage.value = 1
    tableData.loadTable(tableData.activeTableName.value, isLoading)
  }

  const updateCell = async (rowIndex: number, column: GridColumn, newValue: any) => {
    const row = tableData.rows.value[rowIndex]
    if (!row || !window.NL_PORT) return false

    if (row[column.name] === newValue) return true
    
    const pkColumn = tableData.columns.value.find(c => c.isPrimaryKey)
    if (!pkColumn) {
      console.warn('Cannot update cell: No primary key found for table', tableData.activeTableName.value)
      return false
    }

    const pkValue = row[pkColumn.name]
    
    try {
      await BridgeService.request('dbBridge.updateCell', 'dbBridge.updateCellResult', {
        connectionId: connectionsStore.activeConnectionId,
        tableName: tableData.resolveBackendTableName(tableData.activeTableName.value),
        pkColumn: pkColumn.name,
        pkValue,
        targetColumn: column.name,
        newValue
      })
      
      row[column.name] = newValue
      return true
    } catch (error: any) {
      console.error('Failed to update cell:', error.message)
      return false
    }
  }

  const toggleSort = (colName: string) => {
    tableData.toggleSort(colName, isLoading)
  }

  const toggleRowSelection = (rowIdx: number, event: MouseEvent) => {
    const newSet = new Set(selectedRowIndices.value)

    if (event.shiftKey && selectedRowIndices.value.size > 0) {
      const lastIdx = Math.max(...selectedRowIndices.value)
      const start = Math.min(lastIdx, rowIdx)
      const end = Math.max(lastIdx, rowIdx)
      for (let i = start; i <= end; i++) {
        newSet.add(i)
      }
    } else if (event.ctrlKey || event.metaKey) {
      if (newSet.has(rowIdx)) {
        newSet.delete(rowIdx)
      } else {
        newSet.add(rowIdx)
      }
    } else {
      if (newSet.has(rowIdx)) {
        newSet.delete(rowIdx)
      } else {
        newSet.clear()
        newSet.add(rowIdx)
      }
    }

    selectedRowIndices.value = newSet
  }

  const toggleSelectAllRows = () => {
    if (selectedRowIndices.value.size === tableData.rows.value.length) {
      clearSelection()
    } else {
      selectAllRows()
    }
  }

  const clearSelection = () => {
    selectedRowIndices.value = new Set()
  }

  const selectAllRows = () => {
    const newSet = new Set<number>()
    for (let i = 0; i < tableData.rows.value.length; i++) {
      newSet.add(i)
    }
    selectedRowIndices.value = newSet
  }

  const setColumnWidth = (colName: string, width: number) => {
    columnWidths.value = { ...columnWidths.value, [colName]: width }
  }

  const setSelectedCell = (rowIndex: number, column: GridColumn) => {
    selectedCell.value = { rowIndex, column }
  }

  const clearSelectedCell = () => {
    selectedCell.value = null
  }

  const startEditCell = (rowIndex: number, column: GridColumn) => {
    const value = tableData.rows.value[rowIndex]?.[column.name]
    editingCell.value = {
      rowIndex,
      column,
      originalValue: value,
      currentValue: value
    }
  }

  const cancelEditCell = () => {
    editingCell.value = null
  }

  const saveEditCell = async () => {
    if (!editingCell.value) return
    const { rowIndex, column, currentValue } = editingCell.value
    const success = await updateCell(rowIndex, column, currentValue)
    if (success) {
      editingCell.value = null
    }
  }

  const newRowIdx = ref<number | null>(null)
  const newRowData = ref<Record<string, string>>({})

  const createNewRow = () => {
    const placeholder: GridRow = {}
    for (const col of tableData.columns.value) {
      placeholder[col.name] = ''
    }
    tableData.rows.value = [placeholder, ...tableData.rows.value]
    tableData.totalRows.value += 1
    newRowIdx.value = 0
    const data: Record<string, string> = {}
    for (const col of tableData.columns.value) {
      data[col.name] = ''
    }
    newRowData.value = data
  }

  const cancelNewRow = () => {
    if (newRowIdx.value === null) return
    tableData.rows.value.splice(newRowIdx.value, 1)
    tableData.totalRows.value -= 1
    newRowIdx.value = null
    newRowData.value = {}
  }

  const showAlterTableDialog = ref(false)
  const columnVisibility = ref<Record<string, boolean>>({})

  const toggleColumnVisibility = (colName: string) => {
    columnVisibility.value[colName] = !columnVisibility.value[colName]
  }

  const saveNewRow = async () => {
    if (newRowIdx.value === null) return
    try {
      const cleanData: Record<string, string> = {}
      for (const col of tableData.columns.value) {
        const val = newRowData.value[col.name]

        if (!val || val === '') {
          const type = col.dataType ? col.dataType.toLowerCase() : ''
          const isUuid = type.includes('uuid') || type === '2950'
          const isStringPk = col.isPrimaryKey && (['1043', '25', '1042'].includes(type) || type.includes('char') || type.includes('text'))
          if (isUuid || isStringPk) {
            cleanData[col.name] = crypto.randomUUID()
          }
        } else {
          cleanData[col.name] = val
        }
      }

      const savedRow = await insertRowToDB(cleanData)
      tableData.rows.value.splice(newRowIdx.value, 1, savedRow)
      newRowIdx.value = null
      newRowData.value = {}
    } catch (err) {
      console.error('Insert failed:', err)
      throw err
    }
  }

  const insertRowToDB = async (data: Record<string, string> = {}): Promise<GridRow> => {
    if (!window.NL_PORT) return {}
    const payload = await BridgeService.request('dbBridge.insertRow', 'dbBridge.insertRowResult', {
      connectionId: connectionsStore.activeConnectionId,
      tableName: tableData.resolveBackendTableName(tableData.activeTableName.value),
      data,
    })
    return payload.row
  }

  const insertRow = async (data: Record<string, string> = {}): Promise<void> => {
    await insertRowToDB(data)
  }

  const deleteRows = async (indices: number[]): Promise<void> => {
    if (!window.NL_PORT) return
    const pkCol = tableData.columns.value.find(c => c.isPrimaryKey)
    if (!pkCol) throw new Error('No primary key column found')

    const pkValues = indices.map(i => tableData.rows.value[i]?.[pkCol.name]).filter(v => v !== undefined)
    if (pkValues.length === 0) return

    await BridgeService.request('dbBridge.deleteRows', 'dbBridge.deleteRowsResult', {
      connectionId: connectionsStore.activeConnectionId,
      tableName: tableData.resolveBackendTableName(tableData.activeTableName.value),
      pkColumn: pkCol.name,
      pkValues,
    })

    const idxSet = new Set(indices)
    tableData.rows.value = tableData.rows.value.filter((_, i) => !idxSet.has(i))
    tableData.totalRows.value -= indices.length
    clearSelection()
  }

  const getTableColumns = async (tableName: string): Promise<any[]> => {
    if (!window.NL_PORT) return []
    const payload = await BridgeService.request('dbBridge.getTableColumns', 'dbBridge.getTableColumnsResult', {
      connectionId: connectionsStore.activeConnectionId,
      tableName: tableData.resolveBackendTableName(tableName),
    })
    return payload.columns
  }

  const alterTable = async (tableName: string, operations: any[]): Promise<void> => {
    if (!window.NL_PORT) return
    await BridgeService.request('dbBridge.alterTable', 'dbBridge.alterTableResult', {
      connectionId: connectionsStore.activeConnectionId,
      tableName: tableData.resolveBackendTableName(tableName),
      operations,
    })
  }

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
    selectedRowIndices,
    columnWidths,
    setPage,
    setRowsPerPage,
    updateCell,
    toggleSort,
    toggleRowSelection,
    toggleSelectAllRows,
    clearSelection,
    selectAllRows,
    selectedCell,
    editingCell,
    setSelectedCell,
    clearSelectedCell,
    startEditCell,
    cancelEditCell,
    saveEditCell,
    setColumnWidth,
    // New Row addition
    newRowIdx,
    newRowData,
    createNewRow,
    cancelNewRow,
    saveNewRow,
    insertRow,
    deleteRows,
    getTableColumns,
    alterTable,
    showAlterTableDialog,
    columnVisibility,
    toggleColumnVisibility,
  }
})
