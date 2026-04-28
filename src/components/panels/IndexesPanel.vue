<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import { computed } from 'vue'

const gridStore = useGridStore()
const activeTable = computed(() => gridStore.activeTableName)
</script>

<template>
  <div class="flex-1 overflow-auto p-4 bg-surface text-[12px]">
    <div v-if="!activeTable" class="text-text-tertiary italic">
      Select a table to view its indexes.
    </div>
    <div v-else class="flex flex-col gap-4">
      <div class="font-semibold text-text-primary mb-2">Indexes for {{ activeTable }}</div>
      <div class="p-3 border border-border border-dashed rounded text-center text-text-tertiary">
        Index visualization coming soon.
      </div>
    </div>
  </div>
</template>
