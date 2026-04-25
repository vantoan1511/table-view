<script setup lang="ts">
import { ref, computed } from 'vue'
import { useGridStore } from '@/stores/grid'
import * as Neutralino from '@neutralinojs/lib'
import { MessageBoxChoice, Icon } from '@neutralinojs/lib'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import AlterTableDialog from '@/components/ui/AlterTableDialog.vue'
import {
  Filter,
  Columns3,
  Download,
  RefreshCw,
  ChevronDown,
  LayoutGrid,
  Plus,
  Trash2,
  Wrench,
} from 'lucide-vue-next'

const gridStore = useGridStore()

const selectedCount = computed(() => gridStore.selectedRowIndices.size)
const showDeleteConfirm = ref(false)
const showAlterTableDialog = ref(false)

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
    showAlterTableDialog.value = false
    return
  }
  try {
    await gridStore.alterTable(gridStore.activeTableName, operations)
    showAlterTableDialog.value = false
    gridStore.loadTable(gridStore.activeTableName)
    Neutralino.os.showMessageBox('Success', 'Table altered successfully.', MessageBoxChoice.OK, Icon.INFO)
  } catch (err: any) {
    Neutralino.os.showMessageBox('Error', 'Failed to alter table: ' + err.message, MessageBoxChoice.OK, Icon.ERROR)
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
            Neutralino.os.showMessageBox('Success', 'Export completed successfully.', MessageBoxChoice.OK, Icon.INFO)
          } else {
            Neutralino.os.showMessageBox('Error', 'Failed to export: ' + payload.error, MessageBoxChoice.OK, Icon.ERROR)
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
      <button
        id="btn-add-row"
        class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        title="Insert a new row"
        @click="handleInsert"
      >
        <Plus :size="13" class="text-success" />
        <span>Add Row</span>
      </button>

      <!-- Delete Selected -->
      <button
        id="btn-delete-rows"
        class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] transition-colors cursor-pointer"
        :class="selectedCount > 0
          ? 'text-danger border-danger/40 hover:bg-danger-light hover:border-danger/60'
          : 'text-text-tertiary border-border opacity-50 cursor-not-allowed'"
        :disabled="selectedCount === 0"
        :title="selectedCount > 0 ? `Delete ${selectedCount} selected row(s)` : 'Select rows to delete'"
        @click="promptDelete"
      >
        <Trash2 :size="13" />
        <span>Delete{{ selectedCount > 0 ? ` (${selectedCount})` : '' }}</span>
      </button>
    </div>

    <!-- Right: Actions -->
    <div class="flex items-center gap-2">
      <!-- Filter -->
      <div class="relative group">
        <Filter :size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-tertiary group-focus-within:text-primary transition-colors" />
        <input 
          v-model="gridStore.filterText"
          type="text" 
          placeholder="Quick search..." 
          class="pl-8 pr-3 py-1.5 border border-border rounded-lg text-[12px] bg-muted focus:bg-surface focus:border-primary focus:ring-1 focus:ring-primary outline-none transition-all w-[180px] group-hover:border-border-strong"
          @keydown.enter="gridStore.loadTable(gridStore.activeTableName)"
        />
      </div>

      <!-- Columns -->
      <button class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer">
        <Columns3 :size="13" />
        <span>Columns</span>
      </button>

      <!-- Export -->
      <button 
        class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        @click="handleExport"
      >
        <Download :size="13" />
        <span>Export</span>
        <ChevronDown :size="12" />
      </button>

      <!-- Row Count -->
      <div class="flex items-center gap-1.5 px-2.5 py-1.5 border border-border rounded-lg text-[12px] text-text-secondary">
        <span>{{ gridStore.rowsPerPage }} rows</span>
        <ChevronDown :size="12" />
      </div>

      <!-- Alter Table -->
      <button
        class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        title="Alter Table Structure"
        @click="showAlterTableDialog = true"
      >
        <Wrench :size="14" />
      </button>

      <!-- Refresh -->
      <button
        class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer"
        @click="gridStore.loadTable(gridStore.activeTableName)"
      >
        <RefreshCw :size="14" />
      </button>
    </div>
  </div>

  <!-- Delete Confirmation Dialog -->
  <ConfirmDialog
    v-if="showDeleteConfirm"
    title="Delete rows"
    :message="`Are you sure you want to permanently delete ${selectedCount} row(s)? This cannot be undone.`"
    variant="danger"
    confirm-label="Delete"
    @confirm="confirmDelete"
    @cancel="showDeleteConfirm = false"
  />

  <!-- Alter Table Dialog -->
  <AlterTableDialog
    v-if="showAlterTableDialog"
    :tableName="gridStore.activeTableName"
    @close="showAlterTableDialog = false"
    @apply="confirmAlterTable"
  />
</template>

