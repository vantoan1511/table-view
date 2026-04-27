<script setup lang="ts">
import AlterTableDialog from '@/components/ui/AlterTableDialog.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { useGridStore } from '@/stores/grid'
import { useToastStore } from '@/stores/toast'
import * as Neutralino from '@neutralinojs/lib'

import {
  ChevronDown,
  Columns3,
  Download,
  Filter,
  LayoutGrid,
  MoreHorizontal,
  Plus,
  RefreshCw,
  Trash2,
  Wrench
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
const showMoreMenu = ref(false)
const moreMenuPos = ref({ x: 0, y: 0 })
const showSearchPopup = ref(false)
const searchPopupPos = ref({ x: 0, y: 0 })

const columnVisibility = ref<Record<string, boolean>>({})

// Initialize column visibility
watch(() => gridStore.columns, (cols) => {
  cols.forEach(c => {
    if (columnVisibility.value[c.name] === undefined) {
      columnVisibility.value[c.name] = true
    }
  })
}, { immediate: true })

const toggleColumnsMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  columnsMenuPos.value = { x: rect.left, y: rect.bottom + 5 }
  showColumnsMenu.value = !showColumnsMenu.value
  showMoreMenu.value = false
}

const toggleRowsMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  rowsMenuPos.value = { x: rect.left, y: rect.bottom + 5 }
  showRowsMenu.value = !showRowsMenu.value
  showMoreMenu.value = false
}

const toggleMoreMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  // Align to the right of the button
  moreMenuPos.value = { x: rect.right - 180, y: rect.bottom + 5 }
  showMoreMenu.value = !showMoreMenu.value
  showSearchPopup.value = false
}

const toggleSearchPopup = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  // Try to center it or align it reasonably
  searchPopupPos.value = { x: Math.max(10, rect.left - 150), y: rect.bottom + 5 }
  showSearchPopup.value = !showSearchPopup.value
  showMoreMenu.value = false
}

const setRowsPerPage = (count: number) => {
  gridStore.setRowsPerPage(count)
  showRowsMenu.value = false
}

// ─── Add Row Inline ────────────────────────────────────────────────────────
const handleInsert = async () => {
  gridStore.createNewRow()
}

// ─── Delete Confirmation ──────────────────────────────────────────────────────
const promptDelete = () => {
  if (selectedCount.value === 0) return
  showDeleteConfirm.value = true
}

const confirmDelete = async () => {
  showDeleteConfirm.value = false
  try {
    await gridStore.deleteRows([...gridStore.selectedRowIndices])
  } catch (err: any) {
    console.error('Delete failed:', err)
  }
}

const confirmAlterTable = async (operations: any[]) => {
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

const handleExport = async () => {
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

// Custom directive for auto-focusing
const vFocus = {
  mounted: (el: HTMLElement) => el.focus()
}
</script>

<template>
  <div
    class="@container flex items-center justify-between gap-2 px-4 py-2 border-b border-border bg-surface min-h-[48px]">
    <!-- Left: Table name + row actions -->
    <div class="flex items-center gap-2 min-w-0">
      <div class="flex items-center gap-2 mr-1 shrink-0">
        <h2
          class="text-[14px] font-semibold text-text-primary truncate max-w-[80px] @[400px]:max-w-[150px] @[850px]:max-w-none"
          :title="gridStore.activeTableName">
          {{ gridStore.activeTableName }}
        </h2>
      </div>

      <!-- Add Row (Hidden at Level 3, text only at Level 1) -->
      <button id="btn-add-row"
        class="hidden @[500px]:flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer shrink-0 group relative"
        @click="handleInsert">
        <Plus :size="13" class="text-success" />
        <span class="hidden @[850px]:inline">Add Row</span>

        <!-- Tooltip -->
        <div
          class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
          Insert a new row
          <div
            class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
          </div>
        </div>
      </button>

      <!-- Selection Controls (Only visible at Level 1 & 2) -->
      <div v-if="selectedCount > 0" class="hidden @[850px]:flex items-center gap-px bg-border p-px rounded-lg shrink-0">
        <div
          class="flex items-center px-2 py-[5px] bg-primary/10 text-primary text-[11px] font-medium rounded-l-[7px] whitespace-nowrap">
          {{ selectedCount }} <span class="hidden @[900px]:inline ml-1">selected</span>
        </div>
        <button
          class="flex items-center px-2 py-[5px] bg-surface text-[11px] text-text-secondary hover:text-text-primary hover:bg-hover transition-colors group relative"
          @click="gridStore.selectAllRows">
          <span>All</span>
          <!-- Tooltip -->
          <div
            class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
            Select All
            <div
              class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
            </div>
          </div>
        </button>
        <button
          class="flex items-center px-2 py-[5px] bg-surface text-[11px] text-text-secondary hover:text-text-primary hover:bg-hover transition-colors rounded-r-[7px] group relative"
          @click="gridStore.clearSelection">
          <span>None</span>
          <!-- Tooltip -->
          <div
            class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
            Deselect All
            <div
              class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
            </div>
          </div>
        </button>
      </div>

      <!-- Delete Selected (Icon only at Level 2, Hidden in Level 3) -->
      <button v-if="selectedCount > 0" id="btn-delete-rows"
        class="hidden @[500px]:flex items-center gap-1.5 px-2.5 py-1.5 border border-danger/40 rounded-lg text-[12px] text-danger hover:bg-danger-light hover:border-danger/60 transition-colors cursor-pointer shrink-0 group relative"
        @click="promptDelete">
        <Trash2 :size="13" />
        <span class="hidden @[850px]:inline">Delete</span>
        <!-- Tooltip -->
        <div
          class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
          Delete {{ selectedCount }} selected row(s)
          <div
            class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
          </div>
        </div>
      </button>
    </div>

    <!-- Right: Actions -->
    <div class="flex items-center gap-2 min-w-0">
      <!-- Search: Level 1 & 2 (Inline) -->
      <div class="hidden @[500px]:block relative group flex-1 min-w-[120px] max-w-[200px]">
        <Filter :size="13"
          class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-tertiary group-focus-within:text-primary transition-colors" />
        <input v-model="gridStore.filterText" type="text" placeholder="Search..."
          class="w-full pl-8 pr-3 py-1.5 border border-border rounded-lg text-[12px] bg-muted focus:bg-surface focus:border-primary focus:ring-1 focus:ring-primary outline-none transition-all"
          @keydown.enter="gridStore.loadTable(gridStore.activeTableName)" />
      </div>

      <!-- Search: Level 3 (Popup Icon) -->
      <div class="flex @[500px]:hidden items-center">
        <button
          class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover transition-colors cursor-pointer group relative"
          :class="{ 'bg-primary/10 border-primary text-primary': showSearchPopup }" @click.stop="toggleSearchPopup">
          <Filter :size="14" />
          <!-- Tooltip -->
          <div v-if="!showSearchPopup"
            class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
            Search
            <div
              class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
            </div>
          </div>
        </button>

        <ContextMenu :show="showSearchPopup" :x="searchPopupPos.x" :y="searchPopupPos.y" width-class="w-64"
          @close="showSearchPopup = false">
          <div class="px-3 py-2">
            <div class="relative">
              <Filter :size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-tertiary" />
              <input v-model="gridStore.filterText" v-focus type="text" placeholder="Search rows..."
                class="w-full pl-8 pr-3 py-1.5 border border-border rounded-lg text-[12px] outline-none focus:border-primary focus:ring-1 focus:ring-primary"
                @keydown.enter="gridStore.loadTable(gridStore.activeTableName); showSearchPopup = false" />
            </div>
          </div>
        </ContextMenu>
      </div>

      <!-- Level 1 & 2 Actions Group -->
      <div class="hidden @[500px]:flex items-center gap-1.5">
        <!-- Columns -->
        <button
          class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover transition-colors cursor-pointer group relative"
          @click.stop="toggleColumnsMenu">
          <Columns3 :size="13" />
          <span class="hidden @[850px]:inline">Columns</span>
          <!-- Tooltip -->
          <div
            class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
            Configure Columns
            <div
              class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
            </div>
          </div>
        </button>

        <!-- Export -->
        <button
          class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover transition-colors cursor-pointer group relative"
          @click="handleExport">
          <Download :size="13" />
          <span class="hidden @[850px]:inline">Export</span>
          <!-- Tooltip -->
          <div
            class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
            Export Data
            <div
              class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
            </div>
          </div>
        </button>

        <!-- Row Count -->
        <button
          class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover transition-colors cursor-pointer"
          @click.stop="toggleRowsMenu">
          <span class="whitespace-nowrap">{{ gridStore.rowsPerPage }} <span
              class="hidden @[850px]:inline">rows</span></span>
          <ChevronDown :size="12" />
        </button>

        <!-- Divider -->
        <div class="w-px h-5 bg-border mx-0.5 hidden @[600px]:block" />

        <!-- Alter Table -->
        <button
          class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover transition-colors cursor-pointer group relative"
          @click="gridStore.showAlterTableDialog = true">
          <Wrench :size="14" />
          <!-- Tooltip -->
          <div
            class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
            Alter Table Structure
            <div
              class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
            </div>
          </div>
        </button>

        <!-- Refresh -->
        <button
          class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover transition-colors cursor-pointer group relative"
          @click="gridStore.loadTable(gridStore.activeTableName)">
          <RefreshCw :size="14" />
          <!-- Tooltip -->
          <div
            class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
            Refresh Table
            <div
              class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
            </div>
          </div>
        </button>
      </div>

      <!-- Level 3: Speed Dial / More Button -->
      <div class="flex @[500px]:hidden items-center">
        <button
          class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover transition-colors cursor-pointer group relative"
          @click.stop="toggleMoreMenu">
          <MoreHorizontal :size="16" />
          <!-- Tooltip -->
          <div v-if="!showMoreMenu"
            class="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all -translate-y-1 group-hover:translate-y-0 z-50">
            More Actions
            <div
              class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-t border-l border-border rotate-45">
            </div>
          </div>
        </button>

        <ContextMenu :show="showMoreMenu" :x="moreMenuPos.x" :y="moreMenuPos.y" @close="showMoreMenu = false">
          <button class="w-full flex items-center gap-2 px-3 py-2 hover:bg-hover text-[12px]"
            @click="handleInsert(); showMoreMenu = false">
            <Plus :size="14" class="text-success" /> <span>Add Row</span>
          </button>
          <button class="w-full flex items-center gap-2 px-3 py-2 hover:bg-hover text-[12px]"
            @click="gridStore.loadTable(gridStore.activeTableName); showMoreMenu = false">
            <RefreshCw :size="14" /> <span>Refresh</span>
          </button>
          <button class="w-full flex items-center gap-2 px-3 py-2 hover:bg-hover text-[12px]"
            @click="gridStore.showAlterTableDialog = true; showMoreMenu = false">
            <Wrench :size="14" /> <span>Alter Table</span>
          </button>
          <button class="w-full flex items-center gap-2 px-3 py-2 hover:bg-hover text-[12px]"
            @click="toggleColumnsMenu">
            <Columns3 :size="14" /> <span>Columns...</span>
          </button>
          <button class="w-full flex items-center gap-2 px-3 py-2 hover:bg-hover text-[12px]" @click="toggleRowsMenu">
            <LayoutGrid :size="14" /> <span>Rows Per Page...</span>
          </button>
          <button class="w-full flex items-center gap-2 px-3 py-2 hover:bg-hover text-[12px]"
            @click="handleExport; showMoreMenu = false">
            <Download :size="14" /> <span>Export CSV</span>
          </button>
          <div v-if="selectedCount > 0" class="h-px bg-border my-1"></div>
          <button v-if="selectedCount > 0"
            class="w-full flex items-center gap-2 px-3 py-2 hover:bg-hover text-[12px] text-danger"
            @click="promptDelete; showMoreMenu = false">
            <Trash2 :size="14" /> <span>Delete Selected ({{ selectedCount }})</span>
          </button>
        </ContextMenu>
      </div>
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
