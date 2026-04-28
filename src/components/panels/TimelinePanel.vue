<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import { computed } from 'vue'

const gridStore = useGridStore()
// Since we don't have a dedicated timeline store yet, we'll mock it based on active table
const activeTable = computed(() => gridStore.activeTableName)
</script>

<template>
  <div class="flex-1 overflow-auto p-4 bg-surface text-[12px]">
    <div v-if="!activeTable" class="text-text-tertiary italic">
      No active table selected.
    </div>
    <div v-else class="flex flex-col gap-4">
      <div class="flex flex-col gap-1 border-l-2 border-primary/30 pl-3">
        <span class="text-text-tertiary text-[10px]">Just now</span>
        <span class="text-text-primary">Viewed table <span class="font-bold">{{ activeTable }}</span></span>
      </div>
      <div class="flex flex-col gap-1 border-l-2 border-border pl-3">
        <span class="text-text-tertiary text-[10px]">Earlier</span>
        <span class="text-text-secondary">Opened database connection</span>
      </div>
    </div>
  </div>
</template>
