<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import {
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
} from 'lucide-vue-next'
import { computed, ref } from 'vue'

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

// ─── Rows per page dropdown ──────────────────────────────────────────────────
const showRowsMenu = ref(false)
const rowsOptions = [25, 50, 100, 250, 500, 1000]

const setRowsPerPage = (count: number) => {
  gridStore.setRowsPerPage(count)
  showRowsMenu.value = false
}
</script>

<template>
  <div
    class="flex items-center justify-between px-4 py-2 border-t border-border bg-surface text-[12px] text-text-secondary">
    <!-- Left: Row info -->
    <div>
      Showing {{ startRow.toLocaleString() }} to {{ endRow.toLocaleString() }} of {{
        gridStore.totalRows.toLocaleString() }} rows
    </div>

    <!-- Center: Page navigation -->
    <div class="flex items-center gap-0.5">
      <!-- First -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md hover:bg-hover disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-colors"
        :disabled="gridStore.currentPage === 1" @click="gridStore.setPage(1)">
        <ChevronsLeft :size="14" />
      </button>
      <!-- Prev -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md hover:bg-hover disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-colors"
        :disabled="gridStore.currentPage === 1" @click="gridStore.setPage(gridStore.currentPage - 1)">
        <ChevronLeft :size="14" />
      </button>

      <!-- Pages -->
      <template v-for="page in visiblePages" :key="page">
        <span v-if="page === '...'" class="w-7 h-7 flex items-center justify-center text-text-tertiary">…</span>
        <button v-else
          class="flex items-center justify-center w-7 h-7 rounded-md cursor-pointer transition-colors text-[12px]"
          :class="page === gridStore.currentPage
              ? 'bg-primary text-text-inverse font-medium'
              : 'hover:bg-hover text-text-secondary'
            " @click="gridStore.setPage(page as number)">
          {{ page }}
        </button>
      </template>

      <!-- Next -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md hover:bg-hover disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-colors"
        :disabled="gridStore.currentPage === gridStore.totalPages"
        @click="gridStore.setPage(gridStore.currentPage + 1)">
        <ChevronRight :size="14" />
      </button>
      <!-- Last -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md hover:bg-hover disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-colors"
        :disabled="gridStore.currentPage === gridStore.totalPages" @click="gridStore.setPage(gridStore.totalPages)">
        <ChevronsRight :size="14" />
      </button>
    </div>

    <!-- Right: Rows per page -->
    <div class="flex items-center gap-1.5 relative">
      <span>Rows per page</span>
      <button
        class="flex items-center gap-1 px-2 py-1 border border-border rounded-md hover:bg-hover cursor-pointer transition-colors"
        @click.stop="showRowsMenu = !showRowsMenu">
        {{ gridStore.rowsPerPage }}
        <ChevronDown :size="12" :class="{ 'rotate-180': showRowsMenu }" class="transition-transform" />
      </button>

      <!-- Dropdown -->
      <div v-if="showRowsMenu"
        class="absolute bottom-full right-0 mb-1 bg-surface border border-border rounded-lg shadow-lg py-1 z-50 min-w-[80px]"
        @click.stop>
        <button v-for="opt in rowsOptions" :key="opt"
          class="w-full flex items-center justify-between px-3 py-1.5 hover:bg-hover text-[12px] cursor-pointer"
          :class="gridStore.rowsPerPage === opt ? 'text-primary font-medium' : 'text-text-primary'"
          @click="setRowsPerPage(opt)">
          {{ opt }}
          <span v-if="gridStore.rowsPerPage === opt" class="text-primary text-[10px]">✓</span>
        </button>
      </div>
    </div>
  </div>
</template>
