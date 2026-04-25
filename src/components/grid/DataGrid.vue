<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { useGridStore } from '@/stores/grid'
import type { GridColumn } from '@/types'
import { ArrowUp, ArrowDown } from 'lucide-vue-next'
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
  if (column && editValue.value !== gridStore.rows[rowIdx][colName]) {
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
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0 bg-surface">
    <!-- Toolbar -->
    <GridToolbar />

    <!-- Table Container -->
    <div class="flex-1 overflow-auto min-h-0">
      <table class="w-full border-collapse text-[12px] font-[var(--font-mono)]">
        <!-- Header -->
        <thead class="sticky top-0 z-10">
          <tr class="bg-grid-header border-b border-grid-border">
            <!-- Row number header -->
            <th
              class="w-12 px-3 py-2 text-right text-[11px] font-medium text-text-tertiary border-r border-grid-border bg-grid-header"
            >
              #
            </th>
            <th
              v-for="col in gridStore.columns"
              :key="col.name"
              class="px-3 py-1.5 text-left font-medium text-text-primary border-r border-grid-border bg-grid-header cursor-pointer select-none relative group"
              :style="getColStyle(col.name)"
              @click="gridStore.toggleSort(col.name)"
            >
              <div class="flex items-center gap-1.5">
                <span class="text-[12px]">{{ col.name }}</span>
                <span
                  v-if="col.isPrimaryKey"
                  class="text-[10px] text-amber-500 font-bold"
                  title="Primary Key"
                  >PK</span
                >
                <!-- Sort indicator -->
                <ArrowUp
                  v-if="gridStore.sortColumn === col.name && gridStore.sortDirection === 'asc'"
                  :size="12"
                  class="text-primary shrink-0"
                />
                <ArrowDown
                  v-else-if="gridStore.sortColumn === col.name && gridStore.sortDirection === 'desc'"
                  :size="12"
                  class="text-primary shrink-0"
                />
              </div>
              <div class="text-[10px] text-text-tertiary font-normal mt-0.5">
                {{ col.dataType }}
              </div>

              <!-- Resize handle -->
              <div
                class="absolute top-0 right-0 w-[5px] h-full cursor-col-resize hover:bg-primary/30 transition-colors"
                @mousedown="onResizeStart(col.name, $event)"
                @click.stop
              ></div>
            </th>
          </tr>
        </thead>

        <!-- Body -->
        <tbody>
          <tr
            v-for="(row, rowIdx) in gridStore.rows"
            :key="rowIdx"
            class="border-b border-grid-border hover:bg-grid-row-hover transition-colors"
            :class="{
              'bg-grid-row-alt': rowIdx % 2 === 1 && !gridStore.selectedRowIndices.has(rowIdx),
              '!bg-primary/10': gridStore.selectedRowIndices.has(rowIdx),
            }"
          >
            <!-- Row number (selection click target) -->
            <td
              class="px-3 py-1.5 text-right text-[11px] text-text-tertiary border-r border-grid-border tabular-nums cursor-pointer select-none"
              :class="{ '!bg-primary/20 !text-primary font-semibold': gridStore.selectedRowIndices.has(rowIdx) }"
              @click="gridStore.toggleRowSelection(rowIdx, $event)"
            >
              {{ (gridStore.currentPage - 1) * gridStore.rowsPerPage + rowIdx + 1 }}
            </td>

            <!-- Data cells -->
            <td
              v-for="col in gridStore.columns"
              :key="col.name"
              class="px-3 py-1.5 text-text-primary border-r border-grid-border relative"
              :style="getColStyle(col.name)"
              @dblclick="startEdit(rowIdx, col.name, row[col.name])"
            >
              <!-- Editing Input -->
              <div
                v-if="editingCell?.rowIdx === rowIdx && editingCell?.colName === col.name"
                class="absolute inset-0 bg-surface z-10 border-2 border-primary"
              >
                <input
                  :ref="setInputRef"
                  v-model="editValue"
                  class="w-full h-full px-2 text-[12px] font-[var(--font-mono)] outline-none bg-transparent"
                  @blur="saveEdit"
                  @keydown.enter="saveEdit"
                  @keydown.esc="cancelEdit"
                />
              </div>

              <!-- Status badge -->
              <span
                v-else-if="getCellClass(col.name, row[col.name])"
                class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium cursor-default"
                :class="{
                  'bg-success-light text-success': row[col.name] === 'active',
                  'bg-danger-light text-danger': row[col.name] === 'inactive',
                }"
              >
                {{ row[col.name] }}
              </span>

              <!-- Normal value -->
              <span
                v-else
                class="tabular-nums truncate block cursor-text select-none"
                :class="{ 'text-text-tertiary italic': row[col.name] === null }"
              >
                {{ row[col.name] === null ? 'NULL' : row[col.name] }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <Pagination />
  </div>
</template>
