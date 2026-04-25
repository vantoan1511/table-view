<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import * as Neutralino from '@neutralinojs/lib'
import { MessageBoxChoice, Icon } from '@neutralinojs/lib'
import {
  Filter,
  Columns3,
  Download,
  RefreshCw,
  ChevronDown,
  LayoutGrid,
} from 'lucide-vue-next'

const gridStore = useGridStore()

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
    <!-- Left: Table name -->
    <div class="flex items-center gap-2">
      <LayoutGrid :size="16" class="text-text-secondary" />
      <h2 class="text-[15px] font-semibold text-text-primary">
        {{ gridStore.activeTableName }}
      </h2>
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

      <!-- Refresh -->
      <button class="flex items-center justify-center w-8 h-8 border border-border rounded-lg text-text-secondary hover:bg-hover hover:border-border-strong transition-colors cursor-pointer">
        <RefreshCw :size="14" />
      </button>
    </div>
  </div>
</template>
