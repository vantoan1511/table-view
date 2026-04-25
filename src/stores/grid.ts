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
          columns.value = payload.fields.map((f: any) => ({ name: f.name, dataType: String(f.dataTypeID) }))
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
      offset
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
          sqlColumns.value = (payload.fields || []).map((f: any) => ({ name: f.name, dataType: String(f.dataTypeID) }))
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
    totalPages,
    sqlColumns,
    sqlRows,
    sqlRowCount,
    sqlExecutionTime,
    sqlMessages,
    setPage,
    setRowsPerPage,
    loadTable,
    runQuery,
  }
})
