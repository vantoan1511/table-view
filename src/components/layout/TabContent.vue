<script setup lang="ts">
import DataGrid from '@/components/grid/DataGrid.vue';
import SqlEditor from '@/components/sql/SqlEditor.vue';
import type { Tab } from '@/types';

defineProps<{
  tab: Tab | null;
}>();
</script>

<template>
  <div v-if="tab" :key="tab.id" class="bg-surface relative flex min-h-0 min-w-0 flex-1 flex-col">
    <!-- Table -->
    <template v-if="tab?.type === 'table'">
      <DataGrid />
    </template>

    <!-- SQL Editor -->
    <template v-else-if="tab?.type === 'sql'">
      <SqlEditor v-if="tab" :tab="tab" />
    </template>

    <!-- No tab: empty state -->
    <template v-else>
      <div class="text-text-tertiary flex flex-1 flex-col items-center justify-center">
        <svg
          class="mb-4 h-16 w-16 opacity-30"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1"
        >
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
          <line x1="3" y1="9" x2="21" y2="9" />
          <line x1="9" y1="21" x2="9" y2="9" />
        </svg>
        <p class="text-[14px] font-medium">No tab open</p>
        <p class="mt-1 text-[12px]">Select a table from the sidebar or open a SQL editor</p>
      </div>
    </template>
  </div>
</template>
