// src/components/grid/GridHeaderCell.vue
<script setup lang="ts">
import { ArrowDown, ArrowUp } from 'lucide-vue-next';

import type { GridColumn } from '@/types';

defineProps<{
  column: GridColumn;
  isSorted?: boolean;
  sortDirection?: 'asc' | 'desc';
}>();

defineEmits<{
  (e: 'sort'): void;
  (e: 'resize', event: MouseEvent): void;
}>();

const formatDataType = (dt: string): string => {
  const pgOids: Record<string, string> = {
    '16': 'boolean',
    '17': 'bytea',
    '20': 'bigint',
    '21': 'smallint',
    '23': 'integer',
    '25': 'text',
    '114': 'json',
    '700': 'real',
    '701': 'double precision',
    '1042': 'character',
    '1043': 'varchar',
    '1082': 'date',
    '1083': 'time',
    '1114': 'timestamp',
    '1184': 'timestamptz',
    '1700': 'numeric',
    '2950': 'uuid',
    '3802': 'jsonb'
  };
  return pgOids[dt] || dt;
};
</script>

<template>
  <th
    class="text-text-primary border-grid-border bg-grid-header group relative cursor-pointer overflow-hidden border-r px-3 py-1.5 text-left font-medium whitespace-nowrap select-none"
    @click="$emit('sort')"
  >
    <div class="flex items-center gap-1.5 overflow-hidden">
      <span class="truncate text-[12px]">{{ column.displayName || column.name }}</span>
      <span
        v-if="column.isPrimaryKey"
        v-tooltip.top="'Primary Key'"
        class="relative shrink-0 text-[10px] font-bold text-amber-500"
      >
        PK
      </span>
      <ArrowUp
        v-if="isSorted && sortDirection === 'asc'"
        :size="12"
        class="text-primary shrink-0"
      />
      <ArrowDown
        v-else-if="isSorted && sortDirection === 'desc'"
        :size="12"
        class="text-primary shrink-0"
      />
    </div>
    <div class="text-text-tertiary mt-0.5 truncate text-[10px] font-normal">
      {{ formatDataType(column.dataType) }}
    </div>
    <div
      class="hover:bg-primary/30 absolute top-0 right-0 z-40 h-full w-1 cursor-col-resize transition-colors"
      @mousedown.stop="$emit('resize', $event)"
      @click.stop
    ></div>
  </th>
</template>
