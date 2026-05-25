<script setup lang="ts">
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';
import DropdownMenu, { type DropdownValue } from '@/components/ui/DropdownMenu.vue';

import { useGridStore } from '@/stores/grid';
import { useToastStore } from '@/stores/toast';

import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Plus,
  Trash2
} from 'lucide-vue-next';
import { computed, ref } from 'vue';

const gridStore = useGridStore();
const toastStore = useToastStore();

const showDeleteConfirm = ref(false);
const selectedCount = computed(() => gridStore.selectedRowIndices.size);

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

const handleInsert = async () => {
  gridStore.createNewRow();
};

const promptDelete = () => {
  if (selectedCount.value === 0) return;
  showDeleteConfirm.value = true;
};

const confirmDelete = async () => {
  showDeleteConfirm.value = false;
  try {
    await gridStore.deleteRows([...gridStore.selectedRowIndices]);
  } catch (err: any) {
    console.error('Delete failed:', err);
    toastStore.addToast({
      severity: 'error',
      title: 'Delete Failed',
      message: err.message || 'Failed to delete the selected row(s).'
    });
  }
};
</script>

<template>
  <div
    class="border-border bg-surface text-text-secondary flex items-center justify-between border-t px-4 py-2 text-[12px]"
  >
    <!-- Left: Row actions & Row info -->
    <div class="flex items-center gap-2.5">
      <!-- Add Row -->
      <button
        id="btn-add-row"
        class="border-border text-text-secondary hover:bg-hover hover:border-border-strong group relative flex shrink-0 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1 text-[11px] transition-colors"
        @click="handleInsert"
      >
        <Plus :size="12" class="text-success" />
        <span>Add Row</span>
      </button>

      <!-- Delete Selected -->
      <button
        v-if="selectedCount > 0"
        id="btn-delete-rows"
        class="border-danger/40 text-danger hover:bg-danger-light hover:border-danger/60 group relative flex shrink-0 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1 text-[11px] transition-colors"
        @click="promptDelete"
      >
        <Trash2 :size="12" />
        <span>Delete ({{ selectedCount }})</span>
      </button>

      <span>
        Showing {{ startRow.toLocaleString() }} to {{ endRow.toLocaleString() }} of
        {{ gridStore.totalRows.toLocaleString() }} rows
      </span>
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

  <!-- Delete Confirmation Dialog -->
  <ConfirmDialog
    v-if="showDeleteConfirm"
    title="Delete rows"
    :message="`Are you sure you want to permanently delete ${selectedCount} row(s)? This cannot be undone.`"
    variant="danger"
    confirm-label="Delete"
    @confirm="confirmDelete"
    @cancel="showDeleteConfirm = false"
  />
</template>
