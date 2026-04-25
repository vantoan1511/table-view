<script setup lang="ts">
import DataGrid from '@/components/grid/DataGrid.vue'
import SqlEditor from '@/components/sql/SqlEditor.vue'
import type { Tab } from '@/types'

defineProps<{
  tab: Tab | null
}>()
</script>

<template>
  <div class="flex flex-col flex-1 min-w-0 min-h-0">
    <!-- Table -->
    <template v-if="tab?.type === 'table'">
      <DataGrid />
    </template>

    <!-- SQL Editor -->
    <template v-else-if="tab?.type === 'sql'">
      <SqlEditor />
    </template>

    <!-- No tab: empty state -->
    <template v-else>
      <div class="flex flex-col items-center justify-center flex-1 text-text-tertiary">
        <svg class="w-16 h-16 mb-4 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="1">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
          <line x1="3" y1="9" x2="21" y2="9" />
          <line x1="9" y1="21" x2="9" y2="9" />
        </svg>
        <p class="text-[14px] font-medium">No tab open</p>
        <p class="text-[12px] mt-1">Select a table from the sidebar or open a SQL editor</p>
      </div>
    </template>
  </div>
</template>
