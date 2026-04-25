<script setup lang="ts">
import { computed } from 'vue'
import { useGridStore } from '@/stores/grid'
import {
  ChevronsLeft,
  ChevronLeft,
  ChevronRight,
  ChevronsRight,
  ChevronDown,
} from 'lucide-vue-next'

const gridStore = useGridStore()

const startRow = computed(() => (gridStore.currentPage - 1) * gridStore.rowsPerPage + 1)
const endRow = computed(() => Math.min(gridStore.currentPage * gridStore.rowsPerPage, gridStore.totalRows))

const visiblePages = computed(() => {
  const total = gridStore.totalPages
  const current = gridStore.currentPage
  const pages: (number | string)[] = []

  if (total <= 7) {
    for (let i = 1; i <= total; i++) pages.push(i)
    return pages
  }

  pages.push(1)
  if (current > 3) pages.push('...')

  const start = Math.max(2, current - 1)
  const end = Math.min(total - 1, current + 1)
  for (let i = start; i <= end; i++) pages.push(i)

  if (current < total - 2) pages.push('...')
  pages.push(total)

  return pages
})
</script>

<template>
  <div class="flex items-center justify-between px-4 py-2 border-t border-border bg-surface text-[12px] text-text-secondary">
    <!-- Left: Row info -->
    <div>
      Showing {{ startRow.toLocaleString() }} to {{ endRow.toLocaleString() }} of {{ gridStore.totalRows.toLocaleString() }} rows
    </div>

    <!-- Center: Page navigation -->
    <div class="flex items-center gap-0.5">
      <!-- First -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md hover:bg-hover disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-colors"
        :disabled="gridStore.currentPage === 1"
        @click="gridStore.setPage(1)"
      >
        <ChevronsLeft :size="14" />
      </button>
      <!-- Prev -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md hover:bg-hover disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-colors"
        :disabled="gridStore.currentPage === 1"
        @click="gridStore.setPage(gridStore.currentPage - 1)"
      >
        <ChevronLeft :size="14" />
      </button>

      <!-- Pages -->
      <template v-for="page in visiblePages" :key="page">
        <span v-if="page === '...'" class="w-7 h-7 flex items-center justify-center text-text-tertiary">…</span>
        <button
          v-else
          class="flex items-center justify-center w-7 h-7 rounded-md cursor-pointer transition-colors text-[12px]"
          :class="
            page === gridStore.currentPage
              ? 'bg-primary text-text-inverse font-medium'
              : 'hover:bg-hover text-text-secondary'
          "
          @click="gridStore.setPage(page as number)"
        >
          {{ page }}
        </button>
      </template>

      <!-- Next -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md hover:bg-hover disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-colors"
        :disabled="gridStore.currentPage === gridStore.totalPages"
        @click="gridStore.setPage(gridStore.currentPage + 1)"
      >
        <ChevronRight :size="14" />
      </button>
      <!-- Last -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md hover:bg-hover disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-colors"
        :disabled="gridStore.currentPage === gridStore.totalPages"
        @click="gridStore.setPage(gridStore.totalPages)"
      >
        <ChevronsRight :size="14" />
      </button>
    </div>

    <!-- Right: Rows per page -->
    <div class="flex items-center gap-1.5">
      <span>Rows per page</span>
      <button class="flex items-center gap-1 px-2 py-1 border border-border rounded-md hover:bg-hover cursor-pointer transition-colors">
        {{ gridStore.rowsPerPage }}
        <ChevronDown :size="12" />
      </button>
    </div>
  </div>
</template>
