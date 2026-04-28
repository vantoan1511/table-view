import type { GridColumn, GridRow } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import * as Neutralino from '@neutralinojs/lib'
import { useConnectionsStore } from './connections'

export const useGridStore = defineStore('grid', () => {
  const connectionsStore = useConnectionsStore()
  // Table data grid state
  const columns = ref<GridColumn[]>([])
  const rows = ref<GridRow[]>([])
  const totalRows = ref(0)
  const currentPage = ref(1)
  const rowsPerPage = ref(100)
  const sortColumn = ref<string | undefined>()
  const sortDirection = ref<'asc' | 'desc' | undefined>()
  const executionTime = ref(0)
  const activeTableName = ref('')
  const filterText = ref('')
  const isLoading = ref(false)

  // Row selection state
  const selectedRowIndices = ref<Set<number>>(new Set())

  // Column widths state (column name -> width in px)
  const columnWidths = ref<Record<string, number>>({})

  // SQL result state
  const sqlColumns = ref<GridColumn[]>([])
  const sqlRows = ref<GridRow[]>([])
  const sqlRowCount = ref(0)
  const sqlExecutionTime = ref(0)
  const sqlLimit = ref(100)
  const sqlMessages = ref<Array<{ type: string; text: string; timestamp: string }>>([])

  const totalPages = computed(() => Math.max(1, Math.ceil(totalRows.value / rowsPerPage.value)))

  const setPage = (page: number) => {
    currentPage.value = page
    loadTable(activeTableName.value)
  }

  const setRowsPerPage = (count: number) => {
    rowsPerPage.value = count
    currentPage.value = 1
    loadTable(activeTableName.value)
  }

  const loadTable = async (tableName: string) => {
    if (!tableName || !window.NL_PORT) return

    activeTableName.value = tableName
    const reqId = Date.now().toString()
    const offset = (currentPage.value - 1) * rowsPerPage.value
    const startTime = performance.now()
    isLoading.value = true

    const onResult = (evt: any) => {
      const payload = evt.detail
      if (payload.reqId === reqId) {
        if (payload.success) {
          rows.value = payload.rows
          columns.value = payload.fields.map((f: any) => ({ name: f.name, dataType: String(f.dataTypeID), isPrimaryKey: !!f.isPrimaryKey }))
          totalRows.value = payload.totalCount
          executionTime.value = Math.round(performance.now() - startTime)
        } else {
          console.error("Failed to fetch table data:", payload.error)
        }
        isLoading.value = false
        Neutralino.events.off('dbBridge.fetchTableDataResult', onResult)
      }
    }

    Neutralino.events.on('dbBridge.fetchTableDataResult', onResult)
    Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.fetchTableData', {
      reqId,
      connectionId: connectionsStore.activeConnectionId,
      tableName,
      limit: rowsPerPage.value,
      offset,
      sortColumn: sortColumn.value,
      sortDirection: sortDirection.value,
      filter: filterText.value
    })
  }


  const updateCell = async (rowIndex: number, column: GridColumn, newValue: any) => {
    const row = rows.value[rowIndex]
    if (!row || !window.NL_PORT) return false

    // Find the primary key column
    const pkColumn = columns.value.find(c => c.isPrimaryKey)
    if (!pkColumn) {
      console.warn('Cannot update cell: No primary key found for table', activeTableName.value)
      return false
    }

    const pkValue = row[pkColumn.name]
    const reqId = Date.now().toString()

    return new Promise<boolean>((resolve) => {
      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId === reqId) {
          if (payload.success) {
            // Update the local reactive state
            row[column.name] = newValue
            resolve(true)
          } else {
            console.error('Failed to update cell:', payload.error)
            resolve(false)
          }
          Neutralino.events.off('dbBridge.updateCellResult', onResult)
        }
      }

      Neutralino.events.on('dbBridge.updateCellResult', onResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.updateCell', {
        reqId,
        connectionId: connectionsStore.activeConnectionId,
        tableName: activeTableName.value,
        pkColumn: pkColumn.name,
        pkValue,
        targetColumn: column.name,
        newValue
      })
    })
  }

  const runQuery = async (sql: string) => {
    if (!sql || !window.NL_PORT) return

    const reqId = Date.now().toString()
    const startTime = performance.now()
    sqlMessages.value = [] // clear previous messages
    isLoading.value = true

    const onResult = (evt: any) => {
      const payload = evt.detail
      if (payload.reqId === reqId) {
        sqlExecutionTime.value = Math.round(performance.now() - startTime)
        if (payload.success) {
          sqlRows.value = payload.rows || []
          sqlColumns.value = (payload.fields || []).map((f: any) => ({ name: f.name, dataType: String(f.dataTypeID), isPrimaryKey: !!f.isPrimaryKey }))
          sqlRowCount.value = payload.rowCount || 0

          sqlMessages.value.push({
            type: 'info',
            text: `Query executed successfully. ${sqlRowCount.value} rows affected.`,
            timestamp: new Date().toISOString(),
          })
        } else {
          sqlMessages.value.push({
            type: 'error',
            text: `Error: ${payload.error}`,
            timestamp: new Date().toISOString(),
          })
        }
        isLoading.value = false
        Neutralino.events.off('dbBridge.executeQueryResult', onResult)
      }
    }

    Neutralino.events.on('dbBridge.executeQueryResult', onResult)
    Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.executeQuery', {
      reqId,
      connectionId: connectionsStore.activeConnectionId,
      sql
    })
  }

  const toggleSort = (colName: string) => {
    if (sortColumn.value === colName) {
      if (sortDirection.value === 'asc') {
        sortDirection.value = 'desc'
      } else if (sortDirection.value === 'desc') {
        sortColumn.value = undefined
        sortDirection.value = undefined
      }
    } else {
      sortColumn.value = colName
      sortDirection.value = 'asc'
    }
    currentPage.value = 1
    loadTable(activeTableName.value)
  }

  const toggleRowSelection = (rowIdx: number, event: MouseEvent) => {
    const newSet = new Set(selectedRowIndices.value)

    if (event.shiftKey && selectedRowIndices.value.size > 0) {
      // Range select: find the last selected index and fill between
      const lastIdx = Math.max(...selectedRowIndices.value)
      const start = Math.min(lastIdx, rowIdx)
      const end = Math.max(lastIdx, rowIdx)
      for (let i = start; i <= end; i++) {
        newSet.add(i)
      }
    } else if (event.ctrlKey || event.metaKey) {
      // Toggle individual row
      if (newSet.has(rowIdx)) {
        newSet.delete(rowIdx)
      } else {
        newSet.add(rowIdx)
      }
    } else {
      // Single select
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
    if (selectedRowIndices.value.size === rows.value.length) {
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
    for (let i = 0; i < rows.value.length; i++) {
      newSet.add(i)
    }
    selectedRowIndices.value = newSet
  }

  const setColumnWidth = (colName: string, width: number) => {
    columnWidths.value = { ...columnWidths.value, [colName]: width }
  }

  // ─── New Row Inline Adding ───────────────────────────────────────────────
  const newRowIdx = ref<number | null>(null)
  const newRowData = ref<Record<string, string>>({})

  const createNewRow = () => {
    // Insert a placeholder row at the top
    const placeholder: GridRow = {}
    // Fill placeholder with empty strings for each column
    for (const col of columns.value) {
      placeholder[col.name] = ''
    }
    // Prepend to rows and track its index (0 after prepend)
    rows.value = [placeholder, ...rows.value]
    totalRows.value += 1
    newRowIdx.value = 0
    // Initialize data object for binding
    const data: Record<string, string> = {}
    for (const col of columns.value) {
      data[col.name] = ''
    }
    newRowData.value = data
  }

  const cancelNewRow = () => {
    if (newRowIdx.value === null) return
    // Remove the temporary row
    rows.value.splice(newRowIdx.value, 1)
    totalRows.value -= 1
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
      // Filter out empty strings so the DB can apply default values (e.g. for Serial PKs)
      // For UUID columns without a DB default, we generate one automatically.
      const cleanData: Record<string, string> = {}
      for (const col of columns.value) {
        const val = newRowData.value[col.name]

        if (!val || val === '') {
          // If it's a primary key or UUID column and left blank, auto-generate a UUID.
          // Note: Postgres driver returns OIDs for dataType (e.g. 2950 for uuid, 1043 for varchar, 25 for text)
          const type = col.dataType ? col.dataType.toLowerCase() : ''
          const isUuid = type.includes('uuid') || type === '2950'
          const isStringPk = col.isPrimaryKey && (['1043', '25', '1042'].includes(type) || type.includes('char') || type.includes('text'))
          if (isUuid || isStringPk) {
            cleanData[col.name] = crypto.randomUUID()
          }
          // Otherwise, drop it so DB defaults or NULL apply.
        } else {
          cleanData[col.name] = val
        }
      }

      const savedRow = await insertRowToDB(cleanData)
      // Replace placeholder with actual row returned from DB
      rows.value.splice(newRowIdx.value, 1, savedRow)
      newRowIdx.value = null
      newRowData.value = {}
    } catch (err) {
      console.error('Insert failed:', err)
      throw err
    }
  }

  // Internal DB insertion (used by saveNewRow and legacy calls)
  const insertRowToDB = async (data: Record<string, string> = {}): Promise<GridRow> => {
    return new Promise((resolve, reject) => {
      if (!window.NL_PORT) return resolve({})
      const reqId = Date.now().toString()

      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId !== reqId) return
        Neutralino.events.off('dbBridge.insertRowResult', onResult)
        if (payload.success) {
          resolve(payload.row)
        } else {
          reject(new Error(payload.error))
        }
      }

      Neutralino.events.on('dbBridge.insertRowResult', onResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.insertRow', {
        reqId,
        connectionId: connectionsStore.activeConnectionId,
        tableName: activeTableName.value,
        data,
      })
    })
  }

  // Backwards‑compatible wrapper used by UI (old call)
  const insertRow = async (data: Record<string, string> = {}): Promise<void> => {
    await insertRowToDB(data)
  }

  const deleteRows = (indices: number[]): Promise<void> => {
    return new Promise((resolve, reject) => {
      if (!window.NL_PORT) return resolve()
      const pkCol = columns.value.find(c => c.isPrimaryKey)
      if (!pkCol) return reject(new Error('No primary key column found'))

      const pkValues = indices.map(i => rows.value[i]?.[pkCol.name]).filter(v => v !== undefined)
      if (pkValues.length === 0) return resolve()

      const reqId = Date.now().toString()

      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId !== reqId) return
        Neutralino.events.off('dbBridge.deleteRowsResult', onResult)
        if (payload.success) {
          // Remove deleted rows locally
          const idxSet = new Set(indices)
          rows.value = rows.value.filter((_, i) => !idxSet.has(i))
          totalRows.value -= indices.length
          clearSelection()
          resolve()
        } else {
          reject(new Error(payload.error))
        }
      }

      Neutralino.events.on('dbBridge.deleteRowsResult', onResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.deleteRows', {
        reqId,
        connectionId: connectionsStore.activeConnectionId,
        tableName: activeTableName.value,
        pkColumn: pkCol.name,
        pkValues,
      })
    })
  }

  const getTableColumns = (tableName: string): Promise<any[]> => {
    return new Promise((resolve, reject) => {
      if (!window.NL_PORT) return resolve([])
      const reqId = Date.now().toString()
      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId !== reqId) return
        Neutralino.events.off('dbBridge.getTableColumnsResult', onResult)
        if (payload.success) resolve(payload.columns)
        else reject(new Error(payload.error))
      }
      Neutralino.events.on('dbBridge.getTableColumnsResult', onResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.getTableColumns', { reqId, connectionId: connectionsStore.activeConnectionId, tableName })
    })
  }

  const alterTable = (tableName: string, operations: any[]): Promise<void> => {
    return new Promise((resolve, reject) => {
      if (!window.NL_PORT) return resolve()
      const reqId = Date.now().toString()
      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId !== reqId) return
        Neutralino.events.off('dbBridge.alterTableResult', onResult)
        if (payload.success) resolve()
        else reject(new Error(payload.error))
      }
      Neutralino.events.on('dbBridge.alterTableResult', onResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.alterTable', { reqId, connectionId: connectionsStore.activeConnectionId, tableName, operations })
    })
  }

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
    filterText,
    isLoading,
    totalPages,
    selectedRowIndices,
    columnWidths,
    sqlColumns,
    sqlRows,
    sqlRowCount,
    sqlExecutionTime,
    sqlLimit,
    sqlMessages,
    setPage,
    setRowsPerPage,
    loadTable,
    updateCell,
    runQuery,
    // Inline row addition
    newRowIdx,
    newRowData,
    createNewRow,
    cancelNewRow,
    saveNewRow,
    toggleSort,
    toggleRowSelection,
    toggleSelectAllRows,
    clearSelection,
    selectAllRows,
    setColumnWidth,
    insertRow,
    deleteRows,
    getTableColumns,
    alterTable,
    showAlterTableDialog,
    columnVisibility,
    toggleColumnVisibility,
  }
})
