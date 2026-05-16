// src/components/grid/GridCell.vue
<script setup lang="ts">
import { formatGridCellValue } from '@/stores/grid/valueConversion';
import { Check, X } from 'lucide-vue-next';
import { nextTick, ref, watch } from 'vue';

import type { GridColumn } from '@/types';

const props = defineProps<{
  rowIndex: number;
  column: GridColumn;
  value: any;
  isSelected?: boolean;
  isEditing?: boolean;
  isNewRow?: boolean;
  columnWidth?: number;
  minWidth: number;
  maxWidth: number;
  defaultWidth: number;
  validationError?: string;
}>();

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void;
  (e: 'dblclick'): void;
  (e: 'update:value', newValue: any): void;
  (e: 'save'): void;
  (e: 'cancel'): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const localValue = ref(props.value);

watch(
  () => props.value,
  (newVal) => {
    localValue.value = newVal;
  }
);

watch(
  () => props.isEditing,
  (isEditing) => {
    if (isEditing) {
      nextTick(() => {
        inputRef.value?.focus();
        inputRef.value?.select();
      });
    }
  }
);

const onEnter = () => {
  emit('update:value', localValue.value);
  emit('save');
};

const onEsc = () => {
  emit('cancel');
};

const getCellClass = (colName: string, value: unknown): string => {
  if (colName === 'status') {
    if (value === 'active') return 'status-active';
    if (value === 'inactive') return 'status-inactive';
  }
  return '';
};
</script>

<template>
  <td
    class="text-text-primary border-grid-border relative overflow-hidden border-r px-3 py-1.5 transition-all duration-75"
    :class="{
      'ring-primary/50 bg-primary/5 z-10 ring-2 ring-inset': isSelected
    }"
    :style="{
      width: `${Math.min(maxWidth, Math.max(minWidth, columnWidth ?? defaultWidth))}px`,
      minWidth: `${minWidth}px`,
      maxWidth: `${maxWidth}px`
    }"
    @click="emit('click', $event)"
    @dblclick="!isNewRow && emit('dblclick')"
  >
    <!-- Editing State -->
    <template v-if="isEditing">
      <div
        class="bg-surface border-primary absolute inset-0 z-10 flex items-center border-2 shadow-2xl"
      >
        <input
          ref="inputRef"
          v-model="localValue"
          class="h-full flex-1 bg-transparent px-2 text-[12px] font-(--font-mono) outline-none"
          @keydown.enter="onEnter"
          @keydown.esc="onEsc"
          @blur="onEnter"
        />
        <div class="bg-surface border-border flex h-full items-center gap-0.5 border-l px-1">
          <button
            @click.stop="onEnter"
            class="text-success hover:bg-success/10 rounded p-1"
            title="Save"
          >
            <Check :size="12" />
          </button>
          <button
            @click.stop="onEsc"
            class="text-danger hover:bg-danger/10 rounded p-1"
            title="Discard"
          >
            <X :size="12" />
          </button>
        </div>
      </div>
    </template>

    <!-- New Row State -->
    <template v-else-if="isNewRow">
      <div class="relative w-full">
        <input
          v-model="localValue"
          :placeholder="column.isPrimaryKey ? '*Req' : 'NULL'"
          class="w-full rounded border px-1.5 py-0.5 text-[12px] font-(--font-mono) transition-all outline-none focus:ring-1"
          :class="
            props.validationError
              ? 'border-danger bg-danger/5 focus:border-danger focus:ring-danger/30 text-danger'
              : 'border-border bg-surface focus:border-primary focus:ring-primary'
          "
          @input="emit('update:value', localValue)"
          @keydown.enter="emit('save')"
          @keydown.esc="emit('cancel')"
        />
        <!-- Validation error tooltip -->
        <div
          v-if="props.validationError"
          class="bg-danger text-white pointer-events-none absolute top-full left-0 z-50 mt-1 max-w-[200px] truncate rounded px-2 py-0.5 text-[10px] font-medium shadow-lg"
        >
          {{ props.validationError }}
        </div>
      </div>
    </template>

    <!-- Normal State -->
    <template v-else>
      <span
        v-if="getCellClass(column.name, value)"
        class="inline-flex cursor-default items-center rounded-full px-2 py-0.5 text-[11px] font-medium"
        :class="{
          'bg-success-light text-success': value === 'active',
          'bg-danger-light text-danger': value === 'inactive'
        }"
      >
        {{ value }}
      </span>

      <span
        v-else
        class="block cursor-text truncate tabular-nums select-none"
        :class="{ 'text-text-tertiary italic': value === null }"
      >
        {{ value === null ? 'NULL' : formatGridCellValue(value) }}
      </span>
    </template>
  </td>
</template>
