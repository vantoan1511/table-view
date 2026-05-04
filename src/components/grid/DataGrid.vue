<script setup lang="ts">
import ContextMenu from '@/components/ui/ContextMenu.vue'
import Skeleton from '@/components/ui/Skeleton.vue'
import { useGridStore } from '@/stores/grid'
import { useLayoutStore } from '@/stores/layout'
import type { GridColumn } from '@/types'
import { ArrowDown, ArrowUp, Check, Plus, RefreshCw, Trash2, Wrench, X } from 'lucide-vue-next'
import { nextTick, ref, onMounted, onUnmounted } from 'vue'
import GridToolbar from './GridToolbar.vue'
import Pagination from './Pagination.vue'

const gridStore = useGridStore()

// ─── Cell Interaction ──────────────────────────────────────────────────────
const inputRef = ref<HTMLInputElement | null>(null)

const setInputRef = (el: any) => {
  if (el) inputRef.value = el
}

const onCellClick = (rowIdx: number, col: GridColumn) => {
  gridStore.setSelectedCell(rowIdx, col)
  
  // Auto-switch to Value tab if bottom panel is visible
  const layoutStore = useLayoutStore()
  const panel = layoutStore.bottomPanel
  if (layoutStore.isBottomVisible && panel && panel.activeTabId !== 'value') {
    panel.activeTabId = 'value'
  }
}

// ─── Global Click Handling ──────────────────────────────────────────────────
const handleClickOutside = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  
  // Check if we clicked on something that should NOT clear the selection
  const isCell = target.closest('td')
  const isPanel = target.closest('.value-viewer-panel')
  const isToolbar = target.closest('.grid-toolbar')
  const isContextMenu = target.closest('.context-menu')
  const isModal = target.closest('.modal-container') || target.closest('.modal-backdrop') || target.closest('#new-connection-modal')

  if (!isCell && !isPanel && !isToolbar && !isContextMenu && !isModal) {
    gridStore.clearSelectedCell()
  }
}

onMounted(() => {
  window.addEventListener('mousedown', handleClickOutside)
})

onUnmounted(() => {
  window.removeEventListener('mousedown', handleClickOutside)
})

const startEdit = (rowIdx: number, col: GridColumn) => {
  if (col.isPrimaryKey) return
  gridStore.startEditCell(rowIdx, col)
  nextTick(() => {
    if (inputRef.value) {
      inputRef.value.focus()
      inputRef.value.select()
    }
  })
}

// ─── Column Resizing ───────────────────────────────────────────────────────
const resizing = ref<{ colName: string; startX: number; startWidth: number } | null>(null)

const onResizeStart = (colName: string, event: MouseEvent) => {
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

const getColStyle = (colName: string) => {
  const w = gridStore.columnWidths[colName]
  if (w) return { width: `${w}px`, minWidth: `${w}px`, maxWidth: `${w}px` }
  return { minWidth: '120px' }
}

// ─── Status badge helper ───────────────────────────────────────────────────
const getCellClass = (colName: string, value: unknown): string => {
  if (colName === 'status') {
    if (value === 'active') return 'status-active'
    if (value === 'inactive') return 'status-inactive'
  }
  return ''
}

// ─── Data Type helper ──────────────────────────────────────────────────────
const formatDataType = (dt: string): string => {
  const pgOids: Record<string, string> = {
    '16': 'boolean', '17': 'bytea', '20': 'bigint', '21': 'smallint',
    '23': 'integer', '25': 'text', '114': 'json', '700': 'real',
    '701': 'double precision', '1042': 'character', '1043': 'varchar',
    '1082': 'date', '1083': 'time', '1114': 'timestamp',
    '1184': 'timestamptz', '1700': 'numeric', '2950': 'uuid', '3802': 'jsonb',
  }
  return pgOids[dt] || dt
}

// ─── Context Menu ──────────────────────────────────────────────────────────
const contextMenu = ref({ show: false, x: 0, y: 0, rowIdx: -1 })

const onContextMenu = (event: MouseEvent, rowIdx: number) => {
  event.preventDefault()
  let x = event.clientX
  let y = event.clientY
  const menuWidth = 192
  const menuHeight = 160
  if (x + menuWidth > window.innerWidth) x -= menuWidth
  if (y + menuHeight > window.innerHeight) y -= menuHeight
  contextMenu.value = { show: true, x, y, rowIdx }
  if (rowIdx >= 0 && !gridStore.selectedRowIndices.has(rowIdx)) {
    gridStore.clearSelection()
    gridStore.toggleRowSelection(rowIdx, event)
  }
}

const closeContextMenu = () => {
  if (contextMenu.value.show) contextMenu.value.show = false
}

const handleContextAction = (action: string) => {
  closeContextMenu()
  if (action === 'addRow') gridStore.createNewRow()
  else if (action === 'deleteRows') {
    if (gridStore.selectedRowIndices.size > 0) {
      gridStore.deleteRows(Array.from(gridStore.selectedRowIndices)).catch(err => {
        Neutralino.os.showMessageBox('Error', 'Failed to delete row(s): ' + err.message, 'OK', 'ERROR')
      })
    }
  } else if (action === 'alterTable') gridStore.showAlterTableDialog = true
  else if (action === 'refresh') gridStore.loadTable(gridStore.activeTableName)
}
</script>

<template>
  <div class="flex flex-col w-full h-full min-h-0 bg-surface">
    <GridToolbar />

    <div class="flex-1 overflow-auto min-h-0 relative" @contextmenu.prevent="onContextMenu($event, -1)" @click.self="gridStore.clearSelectedCell">
      <table class="w-full border-collapse text-[12px] font-(--font-mono) table-fixed">
        <thead class="sticky top-0 z-20">
          <tr class="bg-grid-header border-b border-grid-border">
            <th class="w-12 px-3 py-2 text-right text-[11px] font-medium text-text-tertiary border-r border-grid-border bg-grid-header sticky left-0 z-30" @click="gridStore.toggleSelectAllRows">
              #
            </th>
            <th v-for="col in gridStore.columns.filter(c => gridStore.columnVisibility[c.name] !== false)" :key="col.name"
              class="px-3 py-1.5 text-left font-medium text-text-primary border-r border-grid-border bg-grid-header cursor-pointer select-none relative group whitespace-nowrap overflow-hidden"
              :style="getColStyle(col.name)" @click="gridStore.toggleSort(col.name)">
              <div class="flex items-center gap-1.5 overflow-hidden">
                <span class="text-[12px] truncate">{{ col.name }}</span>
                <span v-if="col.isPrimaryKey" class="text-[10px] text-amber-500 font-bold shrink-0" title="Primary Key">PK</span>
                <ArrowUp v-if="gridStore.sortColumn === col.name && gridStore.sortDirection === 'asc'" :size="12" class="text-primary shrink-0" />
                <ArrowDown v-else-if="gridStore.sortColumn === col.name && gridStore.sortDirection === 'desc'" :size="12" class="text-primary shrink-0" />
              </div>
              <div class="text-[10px] text-text-tertiary font-normal mt-0.5 truncate">{{ formatDataType(col.dataType) }}</div>
              <div class="absolute top-0 right-0 w-[4px] h-full cursor-col-resize hover:bg-primary/30 transition-colors z-40" @mousedown="onResizeStart(col.name, $event)" @click.stop></div>
            </th>
            <th v-if="gridStore.newRowIdx !== null" class="w-32 px-3 py-1.5 text-left font-medium text-text-primary border-r border-grid-border bg-grid-header sticky right-0 z-30">
              Actions
            </th>
          </tr>
        </thead>

        <template v-if="gridStore.isLoading">
          <tr v-for="i in 10" :key="'skel-' + i" class="border-b border-grid-border">
            <td class="px-3 py-1.5 border-r border-grid-border bg-inherit sticky left-0 z-10"><Skeleton height="1.25rem" /></td>
            <template v-if="gridStore.columns.length > 0">
              <td v-for="col in gridStore.columns.filter(c => gridStore.columnVisibility[c.name] !== false)" :key="'skel-' + col.name" class="px-3 py-1.5 border-r border-grid-border"><Skeleton height="1.25rem" /></td>
            </template>
            <template v-else>
              <td v-for="j in 5" :key="'skel-col-' + j" class="px-3 py-1.5 border-r border-grid-border"><Skeleton height="1.25rem" /></td>
            </template>
          </tr>
        </template>

        <template v-else>
          <tr v-for="(row, rowIdx) in gridStore.rows" :key="rowIdx"
            class="border-b border-grid-border hover:bg-grid-row-hover transition-colors" :class="{
              'bg-grid-row-alt': rowIdx % 2 === 1 && !gridStore.selectedRowIndices.has(rowIdx),
              'bg-primary/10!': gridStore.selectedRowIndices.has(rowIdx),
            }" @contextmenu.prevent.stop="onContextMenu($event, rowIdx)">
            <td class="px-3 py-1.5 text-right text-[11px] text-text-tertiary border-r border-grid-border tabular-nums cursor-pointer select-none sticky left-0 bg-inherit z-10"
              :class="{ 'bg-primary/20! text-primary! font-semibold': gridStore.selectedRowIndices.has(rowIdx) }"
              @click="gridStore.toggleRowSelection(rowIdx, $event)">
              {{ (gridStore.currentPage - 1) * gridStore.rowsPerPage + rowIdx + 1 }}
            </td>

            <td v-for="col in gridStore.columns.filter(c => gridStore.columnVisibility[c.name] !== false)" :key="col.name" 
              class="px-3 py-1.5 text-text-primary border-r border-grid-border relative overflow-hidden transition-all duration-75"
              :class="{ 'ring-2 ring-inset ring-primary/50 bg-primary/5 z-10': gridStore.selectedCell?.rowIndex === rowIdx && gridStore.selectedCell?.column.name === col.name }"
              :style="getColStyle(col.name)"
              @click="onCellClick(rowIdx, col)"
              @dblclick="gridStore.newRowIdx !== rowIdx && startEdit(rowIdx, col)">
              
              <template v-if="gridStore.newRowIdx === rowIdx">
                <input v-model="gridStore.newRowData[col.name]" :placeholder="col.isPrimaryKey ? '(auto)' : col.isNullable ? 'NULL' : '*Req'"
                  class="w-full px-1.5 py-0.5 text-[12px] font-(--font-mono) border border-border rounded outline-none focus:border-primary focus:ring-1 focus:ring-primary bg-surface transition-all"
                  @keydown.enter="gridStore.saveNewRow" @keydown.esc="gridStore.cancelNewRow" />
              </template>

              <template v-else>
                <div v-if="gridStore.editingCell?.rowIndex === rowIdx && gridStore.editingCell?.column.name === col.name"
                  class="absolute inset-0 bg-surface z-10 border-2 border-primary flex items-center shadow-2xl">
                  <input :ref="setInputRef" v-model="gridStore.editingCell.currentValue"
                    class="flex-1 h-full px-2 text-[12px] font-(--font-mono) outline-none bg-transparent"
                    @keydown.enter="gridStore.saveEditCell" @keydown.esc="gridStore.cancelEditCell" />
                  <div class="flex items-center gap-0.5 px-1 bg-surface border-l border-border h-full">
                    <button @click.stop="gridStore.saveEditCell" class="p-1 text-success hover:bg-success/10 rounded" title="Save">
                      <Check :size="12" />
                    </button>
                    <button @click.stop="gridStore.cancelEditCell" class="p-1 text-danger hover:bg-danger/10 rounded" title="Discard">
                      <X :size="12" />
                    </button>
                  </div>
                </div>

                <span v-else-if="getCellClass(col.name, row[col.name])"
                  class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium cursor-default"
                  :class="{ 'bg-success-light text-success': row[col.name] === 'active', 'bg-danger-light text-danger': row[col.name] === 'inactive' }">
                  {{ row[col.name] }}
                </span>

                <span v-else class="tabular-nums truncate block cursor-text select-none" :class="{ 'text-text-tertiary italic': row[col.name] === null }">
                  {{ row[col.name] === null ? 'NULL' : row[col.name] }}
                </span>
              </template>
            </td>

            <td v-if="gridStore.newRowIdx !== null" class="px-2 py-1.5 text-center border-r border-grid-border whitespace-nowrap sticky right-0 bg-inherit z-10">
              <div v-if="gridStore.newRowIdx === rowIdx" class="flex items-center justify-center gap-1">
                <button @click.stop="gridStore.saveNewRow" class="p-1 text-success hover:bg-success/10 rounded transition-colors" title="Save row"><Check :size="14" /></button>
                <button @click.stop="gridStore.cancelNewRow" class="p-1 text-danger hover:bg-danger/10 rounded transition-colors" title="Cancel"><X :size="14" /></button>
              </div>
            </td>
          </tr>
        </template>
      </table>
    </div>

    <Pagination />

    <ContextMenu :show="contextMenu.show" :x="contextMenu.x" :y="contextMenu.y" @close="closeContextMenu">
      <button @click="handleContextAction('addRow')" class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <Plus :size="14" class="text-text-secondary" /> <span>Add Row</span>
      </button>
      <button @click="handleContextAction('deleteRows')" class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-danger hover:bg-danger-light hover:text-danger" :disabled="gridStore.selectedRowIndices.size === 0">
        <Trash2 :size="14" /> <span>Delete Row(s)</span>
      </button>
      <div class="h-px bg-border my-1.5 w-full"></div>
      <button @click="handleContextAction('alterTable')" class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <Wrench :size="14" class="text-text-secondary" /> <span>Alter Table...</span>
      </button>
      <button @click="handleContextAction('refresh')" class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <RefreshCw :size="14" class="text-text-secondary" /> <span>Refresh</span>
      </button>
    </ContextMenu>
  </div>
</template>
