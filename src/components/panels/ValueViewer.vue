<script setup lang="ts">
import Button from 'primevue/button';
import { useGridStore } from '@/stores/grid';
import { formatGridCellValue } from '@/stores/grid/valueConversion';
import { Check, Clipboard, Edit3, FileText, X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const gridStore = useGridStore();
const localValue = ref('');

// Update local value when selected cell changes
watch(
  () => gridStore.selectedCell,
  (cell) => {
    if (cell) {
      const val = gridStore.rows[cell.rowIndex]?.[cell.column.name];
      localValue.value = formatGridCellValue(val);
    } else {
      localValue.value = '';
    }
  },
  { immediate: true }
);

// Sync with editing state (for real-time feedback if changed elsewhere)
watch(
  () => gridStore.editingCell,
  (editCell) => {
    if (
      editCell &&
      gridStore.selectedCell &&
      editCell.rowIndex === gridStore.selectedCell.rowIndex &&
      editCell.column.name === gridStore.selectedCell.column.name
    ) {
      localValue.value = formatGridCellValue(editCell.currentValue);
    } else if (!editCell && gridStore.selectedCell) {
      // Editing was canceled/saved, reset to the current grid value
      const cell = gridStore.selectedCell;
      const currentGridVal = gridStore.rows[cell.rowIndex]?.[cell.column.name];
      localValue.value = formatGridCellValue(currentGridVal);
    }
  },
  { deep: true }
);

const isEditing = computed(() => {
  return !!(
    gridStore.editingCell &&
    gridStore.selectedCell &&
    gridStore.editingCell.rowIndex === gridStore.selectedCell.rowIndex &&
    gridStore.editingCell.column.name === gridStore.selectedCell.column.name
  );
});

const startEdit = () => {
  if (gridStore.selectedCell) {
    gridStore.startEditCell(gridStore.selectedCell.rowIndex, gridStore.selectedCell.column);
  }
};

const handleUpdate = (e: Event) => {
  const val = (e.target as HTMLTextAreaElement).value;
  localValue.value = val;
  if (gridStore.editingCell) {
    gridStore.editingCell.currentValue = val;
  }
};

const copyToClipboard = () => {
  navigator.clipboard.writeText(localValue.value);
};

const isBooleanType = computed(() => {
  const cell = gridStore.selectedCell;
  if (!cell) return false;
  const type = cell.column.dataType.trim().toLowerCase();
  return ['bool', 'boolean', '16'].includes(type);
});

const setBooleanLocal = (valStr: string) => {
  if (!isEditing.value) return;
  localValue.value = valStr;
  if (gridStore.editingCell) {
    gridStore.editingCell.currentValue = valStr;
  }
};
</script>

<template>
  <div class="bg-background value-viewer-panel flex h-full flex-col overflow-hidden">
    <!-- Toolbar -->
    <div class="border-border bg-sidebar/50 flex items-center justify-between border-b px-3 py-2">
      <div class="flex items-center gap-2 overflow-hidden">
        <FileText :size="14" class="text-text-tertiary shrink-0" />
        <span class="text-text-secondary truncate text-[12px] font-medium">
          {{
            gridStore.selectedCell
              ? `${gridStore.selectedCell.column.displayName || gridStore.selectedCell.column.name} (Row ${gridStore.selectedCell.rowIndex + 1})`
              : 'No cell selected'
          }}
        </span>
      </div>

      <div class="flex shrink-0 items-center gap-1.5">
        <template v-if="gridStore.selectedCell">
          <template v-if="isEditing">
            <Button
              size="small"
              severity="success"
              variant="text"
              class="h-7! text-[11px]!"
              @click="gridStore.saveEditCell"
            >
              <Check :size="13" /> Save
            </Button>
            <Button
              size="small"
              severity="danger"
              variant="text"
              class="h-7! text-[11px]!"
              @click="gridStore.cancelEditCell"
            >
              <X :size="13" /> Discard
            </Button>
          </template>
          <template v-else>
            <Button
              size="small"
              severity="primary"
              variant="text"
              class="h-7! text-[11px]!"
              @click="startEdit"
            >
              <Edit3 :size="13" /> Edit
            </Button>
            <Button
              v-tooltip.top="'Copy to clipboard'"
              variant="text"
              severity="secondary"
              size="small"
              class="h-7! w-7! p-0!"
              @click="copyToClipboard"
            >
              <Clipboard :size="14" />
            </Button>
          </template>
        </template>
      </div>
    </div>

    <!-- Content Area -->
    <div class="group relative min-h-0 flex-1">
      <template v-if="gridStore.selectedCell">
        <!-- Boolean visual switcher -->
        <div
          v-if="isBooleanType"
          class="flex h-full flex-col items-center justify-center gap-6 p-6 select-none"
        >
          <div class="flex flex-col items-center gap-2">
            <span class="text-text-tertiary text-[11px] font-semibold tracking-wider uppercase"
              >Boolean Controller</span
            >
            <div
              class="text-text-primary bg-surface/50 border-border rounded border px-3 py-1.5 font-mono text-[14px] font-semibold"
            >
              Current:
              <span
                :class="[
                  localValue === ''
                    ? 'text-text-tertiary italic'
                    : localValue.toLowerCase() === 'true'
                      ? 'text-success font-bold'
                      : 'text-danger font-bold'
                ]"
                >{{ localValue === '' ? 'NULL' : localValue.toUpperCase() }}</span
              >
            </div>
          </div>

          <div class="flex items-center gap-2">
            <Button
              size="small"
              :variant="localValue.toLowerCase() === 'true' ? 'outlined' : 'text'"
              severity="success"
              :disabled="!isEditing"
              @click="setBooleanLocal('true')"
            >
              True
            </Button>

            <Button
              size="small"
              :variant="localValue.toLowerCase() === 'false' ? 'outlined' : 'text'"
              severity="danger"
              :disabled="!isEditing"
              @click="setBooleanLocal('false')"
            >
              False
            </Button>

            <Button
              v-if="gridStore.selectedCell.column.isNullable !== false"
              size="small"
              :variant="
                localValue === '' || localValue.toUpperCase() === 'NULL' ? 'outlined' : 'text'
              "
              severity="secondary"
              :disabled="!isEditing"
              @click="setBooleanLocal('')"
            >
              Null
            </Button>
          </div>

          <p v-if="!isEditing" class="text-text-tertiary text-[11px]">
            Click "Edit" in the toolbar above to modify value.
          </p>
        </div>

        <textarea
          v-else
          class="text-text-primary placeholder-text-tertiary h-full w-full resize-none border-none bg-transparent p-4 font-mono text-[13px] leading-relaxed outline-none"
          :class="{ 'cursor-default': !isEditing, 'bg-surface/30': isEditing }"
          :readonly="!isEditing"
          :value="localValue"
          @input="handleUpdate"
          placeholder="Select a cell to view its content..."
        ></textarea>
      </template>

      <div
        v-else
        class="flex h-full flex-col items-center justify-center gap-3 opacity-30 select-none"
      >
        <FileText :size="48" stroke-width="1" />
        <p class="text-sm font-medium">No cell selected</p>
      </div>
    </div>
  </div>
</template>
