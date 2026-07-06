<script setup lang="ts">
import type { GridColumn } from '@/types';

import { Check, X } from 'lucide-vue-next';
import { computed, nextTick, ref, watch } from 'vue';

import { formatGridCellValue } from '@/stores/grid/valueConversion';

const props = defineProps<{
  rowIndex: number;
  column: GridColumn;
  value: any;
  isSelected?: boolean;
  isEditing?: boolean;
  isNewRow?: boolean;
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

const booleanTypes = new Set(['bool', 'boolean', '16']);
const isBooleanType = computed(() => {
  if (!props.column?.dataType) return false;
  return booleanTypes.has(props.column.dataType.trim().toLowerCase());
});

const toggleBooleanValue = () => {
  if (props.column.isPrimaryKey) return;
  const nextVal = props.value === true ? false : true;
  emit('update:value', nextVal);
};

const toggleNewRowBooleanValue = () => {
  const nextVal = localValue.value === true ? false : true;
  localValue.value = nextVal;
  emit('update:value', nextVal);
};
</script>

<template>
  <td
    class="text-text-primary border-grid-border relative border-r px-3 py-1.5 transition-all duration-75"
    :class="{
      'ring-primary/50 bg-primary/5 z-10 ring-2 ring-inset': isSelected,
      'overflow-hidden': !isEditing
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
        />
        <div
          class="edit-actions-floating bg-surface border-primary absolute -top-0.5 left-full z-20 flex h-[calc(100%+4px)] items-center gap-1 rounded-r border-2 border-l-0 px-1.5 shadow-2xl"
        >
          <button
            @mousedown.prevent="onEnter"
            v-tooltip.top="'Save'"
            class="text-success hover:bg-success/10 rounded p-1"
          >
            <Check :size="12" />
          </button>
          <button
            @mousedown.prevent="onEsc"
            v-tooltip.top="'Discard'"
            class="text-danger hover:bg-danger/10 rounded p-1"
          >
            <X :size="12" />
          </button>
        </div>
      </div>
    </template>

    <!-- New Row State -->
    <template v-else-if="isNewRow">
      <div v-if="isBooleanType" class="flex w-full items-center justify-center py-0.5">
        <button
          type="button"
          class="flex h-4 w-4 items-center justify-center rounded border transition-all duration-200"
          :class="[
            localValue === true
              ? 'bg-primary border-primary hover:bg-primary/90 text-white'
              : localValue === false
                ? 'bg-surface border-border hover:border-primary'
                : 'bg-surface/30 border-border hover:border-primary text-text-tertiary border-dashed'
          ]"
          @click.stop="toggleNewRowBooleanValue"
        >
          <Check v-if="localValue === true" :size="10" stroke-width="3" />
          <div
            v-else-if="localValue === null || localValue === undefined"
            class="bg-text-tertiary h-0.5 w-1.5 rounded-full"
          ></div>
        </button>
      </div>
      <div v-else class="relative w-full">
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
          class="bg-danger pointer-events-none absolute top-full left-0 z-50 mt-1 max-w-[200px] truncate rounded px-2 py-0.5 text-[10px] font-medium text-white shadow-lg"
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

      <!-- Boolean Tri-state Checkbox -->
      <div v-else-if="isBooleanType" class="flex w-full items-center justify-center py-0.5">
        <button
          type="button"
          :disabled="column.isPrimaryKey"
          class="flex h-4 w-4 items-center justify-center rounded border transition-all duration-200"
          :class="[
            value === true
              ? 'bg-primary border-primary hover:bg-primary/90 text-white'
              : value === false
                ? 'bg-surface border-border hover:border-primary'
                : 'bg-surface/30 border-border hover:border-primary text-text-tertiary border-dashed',
            column.isPrimaryKey ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'
          ]"
          @click.stop="toggleBooleanValue"
        >
          <Check v-if="value === true" :size="10" stroke-width="3" />
          <div v-else-if="value === null" class="bg-text-tertiary h-0.5 w-1.5 rounded-full"></div>
        </button>
      </div>

      <span
        v-else
        class="block cursor-text truncate tabular-nums select-text"
        :class="{ 'text-text-tertiary italic': value === null }"
      >
        {{ value === null ? 'NULL' : formatGridCellValue(value) }}
      </span>
    </template>
  </td>
</template>

<style scoped>
td:last-child .edit-actions-floating {
  left: auto;
  right: 100%;
  border-left-width: 2px;
  border-right-width: 0px;
  border-radius: 0.375rem 0 0 0.375rem;
}
</style>
