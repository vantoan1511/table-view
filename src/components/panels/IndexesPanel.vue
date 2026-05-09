<script setup lang="ts">
import { useGridStore } from '@/stores/grid';
import { computed } from 'vue';

const gridStore = useGridStore();
const activeTable = computed(() => gridStore.activeTableName);
</script>

<template>
  <div class="bg-surface flex-1 overflow-auto p-4 text-[12px]">
    <div v-if="!activeTable" class="text-text-tertiary italic">
      Select a table to view its indexes.
    </div>
    <div v-else class="flex flex-col gap-4">
      <div class="text-text-primary mb-2 font-semibold">Indexes for {{ activeTable }}</div>
      <div class="border-border text-text-tertiary rounded border border-dashed p-3 text-center">
        Index visualization coming soon.
      </div>
    </div>
  </div>
</template>
