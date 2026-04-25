import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GridColumn, GridRow, GridState } from '@/types'

// ─── Mock Data ──────────────────────────────────────────────────────────────

const mockColumns: GridColumn[] = [
  { name: 'id', dataType: 'int4', isPrimaryKey: true },
  { name: 'name', dataType: 'varchar' },
  { name: 'email', dataType: 'varchar' },
  { name: 'created_at', dataType: 'timestamptz' },
  { name: 'status', dataType: 'varchar' },
  { name: 'role_id', dataType: 'int4' },
]

const mockNames = [
  'John Doe', 'Jane Smith', 'Michael Brown', 'Emily Johnson', 'Daniel Wilson',
  'Sarah Davis', 'David Miller', 'Olivia Garcia', 'James Martinez', 'Sophia Anderson',
]

const mockEmails = mockNames.map(
  (n) => `${n.toLowerCase().replace(' ', '.')}@example.com`,
)

function generateMockRows(count: number, offset: number): GridRow[] {
  const rows: GridRow[] = []
  for (let i = 0; i < count; i++) {
    const idx = (offset + i) % mockNames.length
    rows.push({
      id: offset + i + 1,
      name: mockNames[idx]!,
      email: mockEmails[idx]!,
      created_at: `2024-05-01 ${String(10 + (i % 14)).padStart(2, '0')}:${String((i * 7 + 15) % 60).padStart(2, '0')}:${String((i * 13 + 23) % 60).padStart(2, '0')}`,
      status: (offset + i) % 5 === 4 ? 'inactive' : 'active',
      role_id: (idx % 2) + 1,
    })
  }
  return rows
}

// ─── SQL Result Mock Data ───────────────────────────────────────────────────

const sqlResultColumns: GridColumn[] = [
  { name: 'id', dataType: 'int4', isPrimaryKey: true },
  { name: 'name', dataType: 'varchar' },
  { name: 'email', dataType: 'varchar' },
  { name: 'created_at', dataType: 'timestamptz' },
  { name: 'role_name', dataType: 'varchar' },
]

const sqlResultRows: GridRow[] = mockNames.slice(0, 8).map((name, i) => ({
  id: i + 1,
  name,
  email: mockEmails[i]!,
  created_at: `2024-05-01 ${String(10 + i).padStart(2, '0')}:${String((i * 7 + 15) % 60).padStart(2, '0')}:${String((i * 13 + 23) % 60).padStart(2, '0')}`,
  role_name: i % 2 === 0 ? 'Admin' : 'Editor',
}))

// ─── Store ──────────────────────────────────────────────────────────────────

export const useGridStore = defineStore('grid', () => {
  // Table data grid state
  const columns = ref<GridColumn[]>(mockColumns)
  const rows = ref<GridRow[]>(generateMockRows(10, 0))
  const totalRows = ref(12345)
  const currentPage = ref(1)
  const rowsPerPage = ref(100)
  const sortColumn = ref<string | undefined>()
  const sortDirection = ref<'asc' | 'desc' | undefined>()
  const executionTime = ref(120)
  const activeTableName = ref('users')

  // SQL result state
  const sqlColumns = ref<GridColumn[]>(sqlResultColumns)
  const sqlRows = ref<GridRow[]>(sqlResultRows)
  const sqlRowCount = ref(100)
  const sqlExecutionTime = ref(120)
  const sqlMessages = ref<Array<{ type: string; text: string; timestamp: string }>>([])

  const totalPages = computed(() => Math.ceil(totalRows.value / rowsPerPage.value))

  function setPage(page: number) {
    currentPage.value = page
    const offset = (page - 1) * rowsPerPage.value
    rows.value = generateMockRows(Math.min(10, rowsPerPage.value), offset)
  }

  function setRowsPerPage(count: number) {
    rowsPerPage.value = count
    currentPage.value = 1
    rows.value = generateMockRows(Math.min(10, count), 0)
  }

  function loadTable(tableName: string) {
    activeTableName.value = tableName
    currentPage.value = 1
    rows.value = generateMockRows(10, 0)
  }

  function runQuery(_sql: string) {
    sqlExecutionTime.value = Math.floor(Math.random() * 200) + 50
    sqlMessages.value = [
      {
        type: 'info',
        text: `Query executed successfully. ${sqlRowCount.value} rows returned.`,
        timestamp: new Date().toISOString(),
      },
    ]
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
