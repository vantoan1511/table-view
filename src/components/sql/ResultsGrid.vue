<script setup lang="ts">
import { useGridStore } from '@/stores/grid'

const gridStore = useGridStore()

const getCellClass = (colName: string, value: unknown) : string => {
  if (colName === 'status') {
    if (value === 'active') return 'status-active'
    if (value === 'inactive') return 'status-inactive'
  }
  return ''
}
</script>

<template>
  <table class="w-full border-collapse text-[12px] font-[var(--font-mono)]">
    <!-- Header -->
    <thead class="sticky top-0 z-10">
      <tr class="bg-grid-header border-b border-grid-border">
        <th
          v-for="col in gridStore.sqlColumns"
          :key="col.name"
          class="px-3 py-1.5 text-left font-medium text-text-primary border-r border-grid-border bg-grid-header min-w-[100px]"
        >
          <div class="text-[12px]">{{ col.name }}</div>
          <div class="text-[10px] text-text-tertiary font-normal mt-0.5">{{ col.dataType }}</div>
        </th>
      </tr>
    </thead>

    <!-- Body -->
    <tbody>
      <tr
        v-for="(row, rowIdx) in gridStore.sqlRows"
        :key="rowIdx"
        class="border-b border-grid-border hover:bg-grid-row-hover transition-colors"
        :class="rowIdx % 2 === 1 ? 'bg-grid-row-alt' : ''"
      >
        <td
          v-for="col in gridStore.sqlColumns"
          :key="col.name"
          class="px-3 py-1.5 text-text-primary border-r border-grid-border"
        >
          <span class="tabular-nums">{{ row[col.name] }}</span>
        </td>
      </tr>
    </tbody>
  </table>
</template>
