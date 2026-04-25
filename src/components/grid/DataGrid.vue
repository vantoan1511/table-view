<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import GridToolbar from './GridToolbar.vue'
import Pagination from './Pagination.vue'

const gridStore = useGridStore()

function getCellClass(colName: string, value: unknown): string {
  if (colName === 'status') {
    if (value === 'active') return 'status-active'
    if (value === 'inactive') return 'status-inactive'
  }
  return ''
}
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0 bg-surface">
    <!-- Toolbar -->
    <GridToolbar />

    <!-- Table Container -->
    <div class="flex-1 overflow-auto min-h-0">
      <table class="w-full border-collapse text-[12px] font-[var(--font-mono)]">
        <!-- Header -->
        <thead class="sticky top-0 z-10">
          <tr class="bg-grid-header border-b border-grid-border">
            <!-- Row number header -->
            <th class="w-12 px-3 py-2 text-right text-[11px] font-medium text-text-tertiary border-r border-grid-border bg-grid-header">
              #
            </th>
            <th
              v-for="col in gridStore.columns"
              :key="col.name"
              class="px-3 py-1.5 text-left font-medium text-text-primary border-r border-grid-border bg-grid-header min-w-[120px]"
            >
              <div class="text-[12px]">{{ col.name }}</div>
              <div class="text-[10px] text-text-tertiary font-normal mt-0.5">{{ col.dataType }}</div>
            </th>
          </tr>
        </thead>

        <!-- Body -->
        <tbody>
          <tr
            v-for="(row, rowIdx) in gridStore.rows"
            :key="rowIdx"
            class="border-b border-grid-border hover:bg-grid-row-hover transition-colors"
            :class="rowIdx % 2 === 1 ? 'bg-grid-row-alt' : ''"
          >
            <!-- Row number -->
            <td class="px-3 py-1.5 text-right text-[11px] text-text-tertiary border-r border-grid-border tabular-nums">
              {{ (gridStore.currentPage - 1) * gridStore.rowsPerPage + rowIdx + 1 }}
            </td>

            <!-- Data cells -->
            <td
              v-for="col in gridStore.columns"
              :key="col.name"
              class="px-3 py-1.5 text-text-primary border-r border-grid-border"
            >
              <!-- Status badge -->
              <span
                v-if="getCellClass(col.name, row[col.name])"
                class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium"
                :class="{
                  'bg-success-light text-success': row[col.name] === 'active',
                  'bg-danger-light text-danger': row[col.name] === 'inactive',
                }"
              >
                {{ row[col.name] }}
              </span>
              <!-- Normal value -->
              <span v-else class="tabular-nums">
                {{ row[col.name] }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <Pagination />
  </div>
</template>
