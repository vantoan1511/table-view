<script setup lang="ts">
import { useGridStore } from '@/stores/grid';

const gridStore = useGridStore();
</script>

<template>
  <table class="w-full border-collapse text-[12px] font-(--font-mono)">
    <!-- Header -->
    <thead class="sticky top-0 z-10">
      <tr class="bg-grid-header border-grid-border border-b">
        <th
          v-for="col in gridStore.sqlColumns"
          :key="col.name"
          class="text-text-primary border-grid-border bg-grid-header min-w-25 border-r px-3 py-1.5 text-left font-medium"
        >
          <div class="text-[12px]">{{ col.name }}</div>
          <div class="text-text-tertiary mt-0.5 text-[10px] font-normal">{{ col.dataType }}</div>
        </th>
      </tr>
    </thead>

    <!-- Body -->
    <tbody>
      <tr
        v-for="(row, rowIdx) in gridStore.sqlRows"
        :key="rowIdx"
        class="border-grid-border hover:bg-grid-row-hover border-b transition-colors"
        :class="rowIdx % 2 === 1 ? 'bg-grid-row-alt' : ''"
      >
        <td
          v-for="col in gridStore.sqlColumns"
          :key="col.name"
          class="text-text-primary border-grid-border border-r px-3 py-1.5"
        >
          <span class="tabular-nums">{{ row[col.name] }}</span>
        </td>
      </tr>
    </tbody>
  </table>
</template>
