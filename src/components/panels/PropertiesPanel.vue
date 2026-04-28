<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import { computed } from 'vue'

const gridStore = useGridStore()
const columns = computed(() => gridStore.columns)
const activeTable = computed(() => gridStore.activeTableName)
</script>

<template>
  <div class="flex-1 overflow-auto p-4 bg-surface text-[12px]">
    <div v-if="!activeTable" class="text-text-tertiary italic">
      Select a table to view its properties.
    </div>
    <div v-else class="flex flex-col gap-4">
      <div class="font-semibold text-text-primary mb-2">Columns for {{ activeTable }}</div>
      <div v-if="columns.length === 0" class="text-text-tertiary">No columns found.</div>
      <div v-else class="flex flex-col gap-2">
        <div v-for="col in columns" :key="col.name" class="p-2 border border-border rounded bg-surface-alt flex flex-col gap-1">
          <div class="flex items-center justify-between">
            <span class="font-bold text-text-primary">{{ col.name }}</span>
            <span v-if="col.isPrimaryKey" class="text-[10px] bg-amber-500/20 text-amber-500 px-1.5 py-0.5 rounded font-bold">PK</span>
          </div>
          <div class="text-text-secondary">Type: <span class="text-primary font-mono">{{ col.dataType }}</span></div>
        </div>
      </div>
    </div>
  </div>
</template>
