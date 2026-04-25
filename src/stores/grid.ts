import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GridColumn, GridRow, GridState } from '@/types'

import * as Neutralino from '@neutralinojs/lib'

export const useGridStore = defineStore('grid', () => {
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

  // Row selection state
  const selectedRowIndices = ref<Set<number>>(new Set())

  // Column widths state (column name -> width in px)
  const columnWidths = ref<Record<string, number>>({})

  // SQL result state
  const sqlColumns = ref<GridColumn[]>([])
  const sqlRows = ref<GridRow[]>([])
  const sqlRowCount = ref(0)
  const sqlExecutionTime = ref(0)
  const sqlMessages = ref<Array<{ type: string; text: string; timestamp: string }>>([])

  const totalPages = computed(() => Math.max(1, Math.ceil(totalRows.value / rowsPerPage.value)))

  function setPage(page: number) {
    currentPage.value = page
    loadTable(activeTableName.value)
  }

  function setRowsPerPage(count: number) {
    rowsPerPage.value = count
    currentPage.value = 1
    loadTable(activeTableName.value)
  }

  async function loadTable(tableName: string) {
    activeTableName.value = tableName
    if (!tableName || !window.NL_PORT) return
    
    const reqId = Date.now().toString()
    const offset = (currentPage.value - 1) * rowsPerPage.value
    const startTime = performance.now()
    
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
        Neutralino.events.off('dbBridge.fetchTableDataResult', onResult)
      }
    }
    
    Neutralino.events.on('dbBridge.fetchTableDataResult', onResult)
    Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.fetchTableData', { 
      reqId,
      tableName,
      limit: rowsPerPage.value,
      offset,
      sortColumn: sortColumn.value,
      sortDirection: sortDirection.value,
      filter: filterText.value
    })
  }

  async function updateCell(rowIndex: number, column: GridColumn, newValue: any) {
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
        tableName: activeTableName.value,
        pkColumn: pkColumn.name,
        pkValue,
        targetColumn: column.name,
        newValue
      })
    })
  }

  async function runQuery(sql: string) {
    if (!sql || !window.NL_PORT) return

    const reqId = Date.now().toString()
    const startTime = performance.now()
    sqlMessages.value = [] // clear previous messages

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
        Neutralino.events.off('dbBridge.executeQueryResult', onResult)
      }
    }

    Neutralino.events.on('dbBridge.executeQueryResult', onResult)
    Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.executeQuery', { 
      reqId,
      sql 
    })
  }

  function toggleSort(colName: string) {
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

  function toggleRowSelection(rowIdx: number, event: MouseEvent) {
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
      newSet.clear()
      newSet.add(rowIdx)
    }

    selectedRowIndices.value = newSet
  }

  function clearSelection() {
    selectedRowIndices.value = new Set()
  }

  function setColumnWidth(colName: string, width: number) {
    columnWidths.value = { ...columnWidths.value, [colName]: width }
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
    totalPages,
    selectedRowIndices,
    columnWidths,
    sqlColumns,
    sqlRows,
    sqlRowCount,
    sqlExecutionTime,
    sqlMessages,
    setPage,
    setRowsPerPage,
    loadTable,
    updateCell,
    runQuery,
    toggleSort,
    toggleRowSelection,
    clearSelection,
    setColumnWidth,
  }
})
