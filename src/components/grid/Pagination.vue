<script setup lang="ts">
import DropdownMenu, { type DropdownValue } from '@/components/ui/DropdownMenu.vue';
import { useGridStore } from '@/stores/grid';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-vue-next';
import { computed } from 'vue';

const gridStore = useGridStore();

const startRow = computed(() => (gridStore.currentPage - 1) * gridStore.rowsPerPage + 1);
const endRow = computed(() =>
  Math.min(gridStore.currentPage * gridStore.rowsPerPage, gridStore.totalRows)
);

const visiblePages = computed(() => {
  const total = gridStore.totalPages;
  const current = gridStore.currentPage;
  const pages: (number | string)[] = [];

  if (total <= 7) {
    for (let i = 1; i <= total; i++) pages.push(i);
    return pages;
  }

  pages.push(1);
  if (current > 3) pages.push('...');

  const start = Math.max(2, current - 1);
  const end = Math.min(total - 1, current + 1);
  for (let i = start; i <= end; i++) pages.push(i);

  if (current < total - 2) pages.push('...');
  pages.push(total);

  return pages;
});

const rowsOptions = [25, 50, 100, 250, 500, 1000].map((value) => ({
  label: String(value),
  value
}));

const setRowsPerPage = (count: DropdownValue) => {
  gridStore.setRowsPerPage(Number(count));
};
</script>

<template>
  <div
    class="border-border bg-surface text-text-secondary flex items-center justify-between border-t px-4 py-2 text-[12px]"
  >
    <!-- Left: Row info -->
    <div>
      Showing {{ startRow.toLocaleString() }} to {{ endRow.toLocaleString() }} of
      {{ gridStore.totalRows.toLocaleString() }} rows
    </div>

    <!-- Center: Page navigation -->
    <div class="flex items-center gap-0.5">
      <!-- First -->
      <button
        class="hover:bg-hover flex h-7 w-7 cursor-pointer items-center justify-center rounded-md transition-colors disabled:cursor-not-allowed disabled:opacity-30"
        :disabled="gridStore.currentPage === 1"
        @click="gridStore.setPage(1)"
      >
        <ChevronsLeft :size="14" />
      </button>
      <!-- Prev -->
      <button
        class="hover:bg-hover flex h-7 w-7 cursor-pointer items-center justify-center rounded-md transition-colors disabled:cursor-not-allowed disabled:opacity-30"
        :disabled="gridStore.currentPage === 1"
        @click="gridStore.setPage(gridStore.currentPage - 1)"
      >
        <ChevronLeft :size="14" />
      </button>

      <!-- Pages -->
      <template v-for="page in visiblePages" :key="page">
        <span
          v-if="page === '...'"
          class="text-text-tertiary flex h-7 w-7 items-center justify-center"
          >…</span
        >
        <button
          v-else
          class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-[12px] transition-colors"
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
        class="hover:bg-hover flex h-7 w-7 cursor-pointer items-center justify-center rounded-md transition-colors disabled:cursor-not-allowed disabled:opacity-30"
        :disabled="gridStore.currentPage === gridStore.totalPages"
        @click="gridStore.setPage(gridStore.currentPage + 1)"
      >
        <ChevronRight :size="14" />
      </button>
      <!-- Last -->
      <button
        class="hover:bg-hover flex h-7 w-7 cursor-pointer items-center justify-center rounded-md transition-colors disabled:cursor-not-allowed disabled:opacity-30"
        :disabled="gridStore.currentPage === gridStore.totalPages"
        @click="gridStore.setPage(gridStore.totalPages)"
      >
        <ChevronsRight :size="14" />
      </button>
    </div>

    <!-- Right: Rows per page -->
    <div class="flex items-center gap-1.5">
      <span>Rows per page</span>
      <DropdownMenu
        :model-value="gridStore.rowsPerPage"
        :options="rowsOptions"
        placement="top"
        align="right"
        aria-label="Rows per page"
        @update:model-value="setRowsPerPage"
      />
    </div>
  </div>
</template>
