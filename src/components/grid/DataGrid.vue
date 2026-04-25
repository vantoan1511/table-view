<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import { ArrowDown, ArrowUp, Check, X } from 'lucide-vue-next'
import { nextTick, ref } from 'vue'
import GridToolbar from './GridToolbar.vue'
import Pagination from './Pagination.vue'

const gridStore = useGridStore()

// ─── Inline Editing ────────────────────────────────────────────────────────
const editingCell = ref<{ rowIdx: number; colName: string } | null>(null)
const editValue = ref<any>('')
const inputRef = ref<HTMLInputElement | null>(null)

function setInputRef(el: any) {
  if (el) inputRef.value = el
}

function startEdit(rowIdx: number, colName: string, value: any) {
  const col = gridStore.columns.find((c) => c.name === colName)
  if (col?.isPrimaryKey) return
  editingCell.value = { rowIdx, colName }
  editValue.value = value
  nextTick(() => {
    inputRef.value?.focus()
    inputRef.value?.select()
  })
}

async function saveEdit() {
  if (!editingCell.value) return
  const { rowIdx, colName } = editingCell.value
  const column = gridStore.columns.find((c) => c.name === colName)
  const row = gridStore.rows[rowIdx]
  if (column && row && editValue.value !== row[colName]) {
    await gridStore.updateCell(rowIdx, column, editValue.value)
  }
  editingCell.value = null
}

function cancelEdit() {
  editingCell.value = null
}

// ─── Column Resizing ───────────────────────────────────────────────────────
const resizing = ref<{ colName: string; startX: number; startWidth: number } | null>(null)

function onResizeStart(colName: string, event: MouseEvent) {
  event.preventDefault()
  event.stopPropagation()
  const th = (event.target as HTMLElement).closest('th')
  if (!th) return
  const startWidth = th.offsetWidth
  resizing.value = { colName, startX: event.clientX, startWidth }

  const onMouseMove = (e: MouseEvent) => {
    if (!resizing.value) return
    const delta = e.clientX - resizing.value.startX
    const newWidth = Math.max(60, resizing.value.startWidth + delta)
    gridStore.setColumnWidth(resizing.value.colName, newWidth)
  }

  const onMouseUp = () => {
    resizing.value = null
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function getColStyle(colName: string) {
  const w = gridStore.columnWidths[colName]
  if (w) return { width: `${w}px`, minWidth: `${w}px`, maxWidth: `${w}px` }
  return { minWidth: '120px' }
}

// ─── Status badge helper ───────────────────────────────────────────────────
function getCellClass(colName: string, value: unknown): string {
  if (colName === 'status') {
    if (value === 'active') return 'status-active'
    if (value === 'inactive') return 'status-inactive'
  }
  return ''
}

// ─── Data Type helper ──────────────────────────────────────────────────────
function formatDataType(dt: string): string {
  const pgOids: Record<string, string> = {
    '16': 'boolean',
    '17': 'bytea',
    '20': 'bigint',
    '21': 'smallint',
    '23': 'integer',
    '25': 'text',
    '114': 'json',
    '700': 'real',
    '701': 'double precision',
    '1042': 'character',
    '1043': 'varchar',
    '1082': 'date',
    '1083': 'time',
    '1114': 'timestamp',
    '1184': 'timestamptz',
    '1700': 'numeric',
    '2950': 'uuid',
    '3802': 'jsonb',
  }
  return pgOids[dt] || dt
}

// ─── Context Menu ──────────────────────────────────────────────────────────
const contextMenu = ref({ show: false, x: 0, y: 0, rowIdx: -1 })

function onContextMenu(event: MouseEvent, rowIdx: number) {
  event.preventDefault()

  // Keep the menu fully on-screen
  let x = event.clientX
  let y = event.clientY
  const menuWidth = 192 // w-48 is 12rem = 192px
  const menuHeight = 160

  if (x + menuWidth > window.innerWidth) x -= menuWidth
  if (y + menuHeight > window.innerHeight) y -= menuHeight

  contextMenu.value = { show: true, x, y, rowIdx }

  // Select the row if it's not selected
  if (rowIdx >= 0 && !gridStore.selectedRowIndices.has(rowIdx)) {
    gridStore.clearSelection()
    gridStore.toggleRowSelection(rowIdx, event)
  }
}

function closeContextMenu() {
  if (contextMenu.value.show) {
    contextMenu.value.show = false
  }
}

function handleContextAction(action: string) {
  closeContextMenu()
  if (action === 'addRow') {
    gridStore.createNewRow()
  } else if (action === 'deleteRows') {
    // Rely on GridToolbar's promptDelete logic or do it directly
    // Let's do it directly
    if (gridStore.selectedRowIndices.size > 0) {
      gridStore.deleteRows(Array.from(gridStore.selectedRowIndices)).catch(err => {
        Neutralino.os.showMessageBox('Error', 'Failed to delete row(s): ' + err.message, 'OK', 'ERROR')
      })
    }
  } else if (action === 'alterTable') {
    gridStore.showAlterTableDialog = true
  } else if (action === 'refresh') {
    gridStore.loadTable(gridStore.activeTableName)
  }
}

import { onMounted, onUnmounted } from 'vue'

onMounted(() => {
  window.addEventListener('click', closeContextMenu)
})
onUnmounted(() => {
  window.removeEventListener('click', closeContextMenu)
})
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0 bg-surface">
    <!-- Toolbar -->
    <GridToolbar />

    <!-- Table Container -->
    <div class="flex-1 overflow-auto min-h-0" @contextmenu.prevent="onContextMenu($event, -1)">
      <table class="w-full border-collapse text-[12px] font-(--font-mono)">
        <!-- Header -->
        <thead class="sticky top-0 z-10">
          <tr class="bg-grid-header border-b border-grid-border">
            <!-- Row number header -->
            <th
              class="w-12 px-3 py-2 text-right text-[11px] font-medium text-text-tertiary border-r border-grid-border bg-grid-header"
              @click="gridStore.selectAllRows">
              #
            </th>
            <th v-for="col in gridStore.columns" :key="col.name"
              class="px-3 py-1.5 text-left font-medium text-text-primary border-r border-grid-border bg-grid-header cursor-pointer select-none relative group"
              :style="getColStyle(col.name)" @click="gridStore.toggleSort(col.name)">
              <div class="flex items-center gap-1.5">
                <span class="text-[12px]">{{ col.name }}</span>
                <span v-if="col.isPrimaryKey" class="text-[10px] text-amber-500 font-bold" title="Primary Key">PK</span>
                <!-- Sort indicator -->
                <ArrowUp v-if="gridStore.sortColumn === col.name && gridStore.sortDirection === 'asc'" :size="12"
                  class="text-primary shrink-0" />
                <ArrowDown v-else-if="gridStore.sortColumn === col.name && gridStore.sortDirection === 'desc'"
                  :size="12" class="text-primary shrink-0" />
              </div>
              <div class="text-[10px] text-text-tertiary font-normal mt-0.5">
                {{ formatDataType(col.dataType) }}
              </div>

              <!-- Resize handle -->
              <div class="absolute top-0 right-0 w-[5px] h-full cursor-col-resize hover:bg-primary/30 transition-colors"
                @mousedown="onResizeStart(col.name, $event)" @click.stop></div>
            </th>
            <th v-if="gridStore.newRowIdx !== null"
              class="w-32 px-3 py-1.5 text-left font-medium text-text-primary border-r border-grid-border bg-grid-header">
              Actions
            </th>
          </tr>
        </thead>

        <!-- Body -->
        <tbody>
          <tr v-for="(row, rowIdx) in gridStore.rows" :key="rowIdx"
            class="border-b border-grid-border hover:bg-grid-row-hover transition-colors" :class="{
              'bg-grid-row-alt': rowIdx % 2 === 1 && !gridStore.selectedRowIndices.has(rowIdx),
              'bg-primary/10!': gridStore.selectedRowIndices.has(rowIdx),
            }" @contextmenu.prevent.stop="onContextMenu($event, rowIdx)">
            <!-- Row number (selection click target) -->
            <td
              class="px-3 py-1.5 text-right text-[11px] text-text-tertiary border-r border-grid-border tabular-nums cursor-pointer select-none"
              :class="{ 'bg-primary/20! text-primary! font-semibold': gridStore.selectedRowIndices.has(rowIdx) }"
              @click="gridStore.toggleRowSelection(rowIdx, $event)">
              {{ (gridStore.currentPage - 1) * gridStore.rowsPerPage + rowIdx + 1 }}
            </td>

            <!-- Data cells -->
            <td v-for="col in gridStore.columns" :key="col.name"
              class="px-3 py-1.5 text-text-primary border-r border-grid-border relative" :style="getColStyle(col.name)"
              @dblclick="gridStore.newRowIdx !== rowIdx && startEdit(rowIdx, col.name, row[col.name])">
              <!-- New Row Input -->
              <template v-if="gridStore.newRowIdx === rowIdx">
                <input v-model="gridStore.newRowData[col.name]"
                  :placeholder="col.isPrimaryKey ? '(auto)' : col.isNullable ? 'NULL' : '*Required'"
                  class="w-full px-1.5 py-0.5 text-[12px] font-(--font-mono) border border-border rounded outline-none focus:border-primary focus:ring-1 focus:ring-primary bg-surface transition-all"
                  @keydown.enter="gridStore.saveNewRow" @keydown.esc="gridStore.cancelNewRow" />
              </template>

              <template v-else>
                <!-- Editing Input -->
                <div v-if="editingCell?.rowIdx === rowIdx && editingCell?.colName === col.name"
                  class="absolute inset-0 bg-surface z-10 border-2 border-primary">
                  <input :ref="setInputRef" v-model="editValue"
                    class="w-full h-full px-2 text-[12px] font-(--font-mono) outline-none bg-transparent"
                    @blur="saveEdit" @keydown.enter="saveEdit" @keydown.esc="cancelEdit" />
                </div>

                <!-- Status badge -->
                <span v-else-if="getCellClass(col.name, row[col.name])"
                  class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium cursor-default"
                  :class="{
                    'bg-success-light text-success': row[col.name] === 'active',
                    'bg-danger-light text-danger': row[col.name] === 'inactive',
                  }">
                  {{ row[col.name] }}
                </span>

                <!-- Normal value -->
                <span v-else class="tabular-nums truncate block cursor-text select-none"
                  :class="{ 'text-text-tertiary italic': row[col.name] === null }">
                  {{ row[col.name] === null ? 'NULL' : row[col.name] }}
                </span>
              </template>
            </td>

            <!-- Actions Cell -->
            <td v-if="gridStore.newRowIdx !== null"
              class="px-2 py-1.5 text-center border-r border-grid-border whitespace-nowrap">
              <div v-if="gridStore.newRowIdx === rowIdx" class="flex items-center justify-center gap-1">
                <button @click.stop="gridStore.saveNewRow"
                  class="p-1 text-success hover:bg-success/10 rounded transition-colors" title="Save row">
                  <Check :size="14" />
                </button>
                <button @click.stop="gridStore.cancelNewRow"
                  class="p-1 text-danger hover:bg-danger/10 rounded transition-colors" title="Cancel">
                  <X :size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <Pagination />

    <!-- Context Menu -->
    <div v-if="contextMenu.show" class="fixed z-100 w-52 py-1.5 bg-surface border border-border rounded-lg shadow-xl"
      :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }" @click.stop>
      <button @click="handleContextAction('addRow')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <Plus :size="14" class="text-text-secondary" /> <span>Add Row</span>
      </button>
      <button @click="handleContextAction('deleteRows')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-danger hover:bg-danger-light hover:text-danger"
        :disabled="gridStore.selectedRowIndices.size === 0">
        <Trash2 :size="14" /> <span>Delete Row(s)</span>
      </button>
      <div class="h-px bg-border my-1.5 w-full"></div>
      <button @click="handleContextAction('alterTable')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <Wrench :size="14" class="text-text-secondary" /> <span>Alter Table...</span>
      </button>
      <button @click="handleContextAction('refresh')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <RefreshCw :size="14" class="text-text-secondary" /> <span>Refresh</span>
      </button>
    </div>
  </div>
</template>
