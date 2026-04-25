<script setup lang="ts">
import AlterTableDialog from '@/components/ui/AlterTableDialog.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { useGridStore } from '@/stores/grid'
import { useToastStore } from '@/stores/toast'
import * as Neutralino from '@neutralinojs/lib'
import { Icon, MessageBoxChoice } from '@neutralinojs/lib'

import {
  Check,
  ChevronDown,
  Columns3,
  Download,
  Filter,
  LayoutGrid,
  Plus,
  RefreshCw,
  Trash2,
  Wrench,
} from 'lucide-vue-next'
import { computed, ref, watch } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'

const gridStore = useGridStore()
const toastStore = useToastStore()

const selectedCount = computed(() => gridStore.selectedRowIndices.size)
const showDeleteConfirm = ref(false)

// ─── Dropdowns state ────────────────────────────────────────────────────────
const showColumnsMenu = ref(false)
const columnsMenuPos = ref({ x: 0, y: 0 })
const showRowsMenu = ref(false)
const rowsMenuPos = ref({ x: 0, y: 0 })

const columnVisibility = ref<Record<string, boolean>>({})

// Initialize column visibility
watch(() => gridStore.columns, (cols) => {
  cols.forEach(c => {
    if (columnVisibility.value[c.name] === undefined) {
      columnVisibility.value[c.name] = true
    }
  })
}, { immediate: true })

function toggleColumnsMenu(e: MouseEvent) {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  columnsMenuPos.value = { x: rect.left, y: rect.bottom + 5 }
  showColumnsMenu.value = !showColumnsMenu.value
}

function toggleRowsMenu(e: MouseEvent) {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  rowsMenuPos.value = { x: rect.left, y: rect.bottom + 5 }
  showRowsMenu.value = !showRowsMenu.value
}

function setRowsPerPage(count: number) {
  gridStore.setRowsPerPage(count)
  showRowsMenu.value = false
}

// ─── Add Row Inline ────────────────────────────────────────────────────────
async function handleInsert() {
  gridStore.createNewRow()
}

// ─── Delete Confirmation ──────────────────────────────────────────────────────
function promptDelete() {
  if (selectedCount.value === 0) return
  showDeleteConfirm.value = true
}

async function confirmDelete() {
  showDeleteConfirm.value = false
  try {
    await gridStore.deleteRows([...gridStore.selectedRowIndices])
  } catch (err: any) {
    console.error('Delete failed:', err)
  }
}

async function confirmAlterTable(operations: any[]) {
  if (operations.length === 0) {
    gridStore.showAlterTableDialog = false
    return
  }
  try {
    await gridStore.alterTable(gridStore.activeTableName, operations)
    gridStore.showAlterTableDialog = false
    gridStore.loadTable(gridStore.activeTableName)
    toastStore.addToast({
      title: 'Table Altered',
      message: 'Table altered successfully.',
      severity: 'success',
      variation: 'filled',
      position: 'bottom-center',
    })
  } catch (err: any) {
    toastStore.addToast({
      title: 'Table Alteration Failed',
      message: err.message,
      severity: 'error',
      variation: 'filled',
      position: 'bottom-center'
    })
  }
}

async function handleExport() {
  if (!gridStore.activeTableName) return

  try {
    const defaultName = `${gridStore.activeTableName}_export.csv`
    const path = await Neutralino.os.showSaveDialog('Export as CSV', {
      defaultPath: defaultName,
      filters: [{ name: 'CSV files', extensions: ['csv'] }]
    })

    if (path) {
      const reqId = Date.now().toString()

      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId === reqId) {
          if (payload.success) {
            toastStore.addToast({
              title: 'Export Success',
              message: 'Data exported successfully to ' + path,
              severity: 'success'
            })
          } else {
            toastStore.addToast({
              title: 'Export Error',
              message: payload.error,
              severity: 'error'
            })
          }
          Neutralino.events.off('dbBridge.exportCSVResult', onResult)
        }
      }

      Neutralino.events.on('dbBridge.exportCSVResult', onResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.exportCSV', {
        reqId,
        tableName: gridStore.activeTableName,
        exportPath: path
      })
    }
  } catch (err) {
    console.error("Export cancelled or failed", err)
  }
}
</script>

<template>
  <div class="flex items-center justify-between px-4 py-2 border-b border-border bg-surface">
    <!-- Left: Table name + row actions -->
    <div class="flex items-center gap-2">
      <LayoutGrid :size="16" class="text-text-secondary" />
      <h2 class="text-[15px] font-semibold text-text-primary">
        {{ gridStore.activeTableName }}
      </h2>

      <!-- Divider -->
      <div class="w-px h-5 bg-border mx-1" />

      <!-- Add Row -->
      <button id="btn-add-row"
        class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        title="Insert a new row" @click="handleInsert">
        <Plus :size="13" class="text-success" />
        <span>Add Row</span>
      </button>

      <!-- Selection Controls -->
      <div v-if="selectedCount > 0" class="flex items-center gap-px bg-border p-px rounded-lg">
        <div
          class="flex items-center px-2.5 py-[5px] bg-primary/10 text-primary text-[12px] font-medium rounded-l-[7px]">
          {{ selectedCount }} selected
        </div>
        <button
          class="flex items-center gap-1.5 px-2.5 py-[5px] bg-surface text-[12px] text-text-secondary hover:text-text-primary hover:bg-hover transition-colors"
          :class="{
            'rounded-l-[7px]': selectedCount === 0,
            'text-primary': selectedCount === gridStore.rows.length && gridStore.rows.length > 0
          }" title="Select All" @click="gridStore.selectAllRows">
          <span>All</span>
        </button>
        <button
          class="flex items-center gap-1.5 px-2.5 py-[5px] bg-surface text-[12px] text-text-secondary hover:text-text-primary hover:bg-hover transition-colors rounded-r-[7px]"
          title="Deselect All" @click="gridStore.clearSelection" :disabled="selectedCount === 0"
          :class="selectedCount === 0 ? 'opacity-50 cursor-not-allowed' : ''">
          <span>None</span>
        </button>
      </div>

      <!-- Delete Selected -->
      <button id="btn-delete-rows"
        class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] transition-colors cursor-pointer"
        :class="selectedCount > 0
          ? 'text-danger border-danger/40 hover:bg-danger-light hover:border-danger/60'
          : 'text-text-tertiary border-border opacity-50 cursor-not-allowed'" :disabled="selectedCount === 0"
        :title="selectedCount > 0 ? `Delete ${selectedCount} selected row(s)` : 'Select rows to delete'"
        @click="promptDelete">
        <Trash2 :size="13" />
        <span>Delete</span>
      </button>
    </div>

    <!-- Right: Actions -->
    <div class="flex items-center gap-2">
      <!-- Filter -->
      <div class="relative group">
        <Filter :size="13"
          class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-tertiary group-focus-within:text-primary transition-colors" />
        <input v-model="gridStore.filterText" type="text" placeholder="Quick search..."
          class="pl-8 pr-3 py-1.5 border border-border rounded-lg text-[12px] bg-muted focus:bg-surface focus:border-primary focus:ring-1 focus:ring-primary outline-none transition-all w-[180px] group-hover:border-border-strong"
          @keydown.enter="gridStore.loadTable(gridStore.activeTableName)" />
      </div>

      <!-- Columns Dropdown -->
      <button
        class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        @click="toggleColumnsMenu">
        <Columns3 :size="13" />
        <span>Columns</span>
      </button>
      <ContextMenu :show="showColumnsMenu" :x="columnsMenuPos.x" :y="columnsMenuPos.y" @close="showColumnsMenu = false">
        <div class="px-3 py-2 text-[11px] font-semibold text-text-tertiary uppercase tracking-wider border-b border-border mb-1">
          Toggle Visibility
        </div>
        <div class="max-h-[300px] overflow-y-auto">
          <label v-for="col in gridStore.columns" :key="col.name" 
            class="flex items-center gap-2 px-3 py-1.5 hover:bg-hover cursor-pointer text-[12px]">
            <input type="checkbox" :checked="gridStore.columnVisibility[col.name] !== false"
              @change="gridStore.toggleColumnVisibility(col.name)"
              class="rounded border-border text-primary focus:ring-primary" />
            <span class="truncate">{{ col.name }}</span>
          </label>
        </div>
      </ContextMenu>

      <!-- Export -->
      <button
        class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        @click="handleExport">
        <Download :size="13" />
        <span>Export</span>
        <ChevronDown :size="12" />
      </button>

      <!-- Row Count Dropdown -->
      <button
        class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover transition-colors cursor-pointer"
        @click="toggleRowsMenu">
        <span>{{ gridStore.rowsPerPage }} rows</span>
        <ChevronDown :size="12" />
      </button>
      <ContextMenu :show="showRowsMenu" :x="rowsMenuPos.x" :y="rowsMenuPos.y" @close="showRowsMenu = false">
        <div class="px-3 py-2 text-[11px] font-semibold text-text-tertiary uppercase tracking-wider border-b border-border mb-1">
          Rows Per Page
        </div>
        <button v-for="count in [25, 50, 100, 200, 500]" :key="count"
          class="w-full flex items-center justify-between px-3 py-2 hover:bg-hover text-[12px]"
          @click="setRowsPerPage(count)">
          <span>{{ count }} rows</span>
          <Check v-if="gridStore.rowsPerPage === count" :size="14" class="text-primary" />
        </button>
      </ContextMenu>

      <!-- Alter Table -->
      <button
        class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        title="Alter Table Structure" @click="gridStore.showAlterTableDialog = true">
        <Wrench :size="14" />
      </button>

      <!-- Refresh -->
      <button
        class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        @click="gridStore.loadTable(gridStore.activeTableName)">
        <RefreshCw :size="14" />
      </button>
    </div>
  </div>

  <!-- Delete Confirmation Dialog -->
  <ConfirmDialog v-if="showDeleteConfirm" title="Delete rows"
    :message="`Are you sure you want to permanently delete ${selectedCount} row(s)? This cannot be undone.`"
    variant="danger" confirm-label="Delete" @confirm="confirmDelete" @cancel="showDeleteConfirm = false" />

  <!-- Alter Table Dialog -->
  <AlterTableDialog v-if="gridStore.showAlterTableDialog" :tableName="gridStore.activeTableName"
    @close="gridStore.showAlterTableDialog = false" @apply="confirmAlterTable" />
</template>
