<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import { Check, X, Edit3, Clipboard, FileText } from 'lucide-vue-next'
import { computed, watch, ref } from 'vue'

const gridStore = useGridStore()
const localValue = ref('')

// Update local value when selected cell changes
watch(() => gridStore.selectedCell, (cell) => {
  if (cell) {
    const val = gridStore.rows[cell.rowIndex]?.[cell.column.name]
    localValue.value = val === null ? '' : String(val)
  } else {
    localValue.value = ''
  }
}, { immediate: true })

// Sync with editing state (for real-time feedback if changed elsewhere)
watch(() => gridStore.editingCell?.currentValue, (val) => {
  if (gridStore.editingCell && gridStore.selectedCell && 
      gridStore.editingCell.rowIndex === gridStore.selectedCell.rowIndex &&
      gridStore.editingCell.column.name === gridStore.selectedCell.column.name) {
    localValue.value = val === null ? '' : String(val)
  }
})

const isEditing = computed(() => {
  return !!(gridStore.editingCell && 
         gridStore.selectedCell && 
         gridStore.editingCell.rowIndex === gridStore.selectedCell.rowIndex &&
         gridStore.editingCell.column.name === gridStore.selectedCell.column.name)
})

const startEdit = () => {
  if (gridStore.selectedCell) {
    gridStore.startEditCell(gridStore.selectedCell.rowIndex, gridStore.selectedCell.column)
  }
}

const handleUpdate = (e: Event) => {
  const val = (e.target as HTMLTextAreaElement).value
  localValue.value = val
  if (gridStore.editingCell) {
    gridStore.editingCell.currentValue = val
  }
}

const copyToClipboard = () => {
  navigator.clipboard.writeText(localValue.value)
}
</script>

<template>
  <div class="flex flex-col h-full bg-background overflow-hidden value-viewer-panel">
    <!-- Toolbar -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-border bg-sidebar/50">
      <div class="flex items-center gap-2 overflow-hidden">
        <FileText :size="14" class="text-text-tertiary shrink-0" />
        <span class="text-[12px] font-medium text-text-secondary truncate">
          {{ gridStore.selectedCell ? `${gridStore.selectedCell.column.name} (Row ${gridStore.selectedCell.rowIndex + 1})` : 'No cell selected' }}
        </span>
      </div>

      <div class="flex items-center gap-1.5 shrink-0">
        <template v-if="gridStore.selectedCell">
          <template v-if="isEditing">
            <button 
              class="flex items-center gap-1.5 px-2 py-1 bg-success/10 hover:bg-success/20 text-success rounded text-[11px] font-bold transition-colors cursor-pointer"
              @click="gridStore.saveEditCell">
              <Check :size="13" /> Save
            </button>
            <button 
              class="flex items-center gap-1.5 px-2 py-1 bg-danger/10 hover:bg-danger/20 text-danger rounded text-[11px] font-bold transition-colors cursor-pointer"
              @click="gridStore.cancelEditCell">
              <X :size="13" /> Discard
            </button>
          </template>
          <template v-else>
            <button 
              class="flex items-center gap-1.5 px-2 py-1 bg-primary/10 hover:bg-primary/20 text-primary rounded text-[11px] font-bold transition-colors cursor-pointer"
              @click="startEdit">
              <Edit3 :size="13" /> Edit
            </button>
            <button 
              class="p-1.5 text-text-tertiary hover:text-text-primary hover:bg-hover rounded transition-colors cursor-pointer"
              title="Copy to clipboard" @click="copyToClipboard">
              <Clipboard :size="14" />
            </button>
          </template>
        </template>
      </div>
    </div>

    <!-- Content Area -->
    <div class="flex-1 min-h-0 relative group">
      <textarea
        v-if="gridStore.selectedCell"
        class="w-full h-full p-4 bg-transparent border-none outline-none resize-none text-[13px] font-mono leading-relaxed text-text-primary placeholder-text-tertiary"
        :class="{ 'cursor-default': !isEditing, 'bg-surface/30': isEditing }"
        :readonly="!isEditing"
        :value="localValue"
        @input="handleUpdate"
        placeholder="Select a cell to view its content..."
      ></textarea>
      
      <div v-else class="flex flex-col items-center justify-center h-full gap-3 opacity-30 select-none">
        <FileText :size="48" stroke-width="1" />
        <p class="text-sm font-medium">No cell selected</p>
      </div>
    </div>
  </div>
</template>
