<script setup lang="ts">
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';
import Button from 'primevue/button';
import Select from 'primevue/select';

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
      <Button
        v-tooltip.top="'Add row'"
        id="btn-add-row"
        variant="outlined"
        severity="success"
        size="small"
        @click="handleInsert"
      >
        <template #icon>
          <Plus class="h-4 w-4" />
        </template>
      </Button>

      <!-- Delete Selected -->
      <Button
        v-if="selectedCount > 0"
        id="btn-delete-rows"
        variant="text"
        severity="danger"
        size="small"
        @click="promptDelete"
      >
        <Trash2 class="h-4 w-4" />
        <span>Delete ({{ selectedCount }})</span>
      </Button>

      <span>
        Showing {{ startRow.toLocaleString() }} to {{ endRow.toLocaleString() }} of
        {{ gridStore.totalRows.toLocaleString() }} rows
      </span>
    </div>

    <!-- Center: Page navigation -->
    <div class="flex items-center gap-0.5">
      <!-- First -->
      <Button
        variant="text"
        severity="secondary"
        size="small"
        class="h-7! w-7! p-0!"
        :disabled="gridStore.currentPage === 1"
        @click="gridStore.setPage(1)"
      >
        <template #icon>
          <ChevronsLeft :size="14" />
        </template>
      </Button>
      <!-- Prev -->
      <Button
        variant="text"
        severity="secondary"
        size="small"
        class="h-7! w-7! p-0!"
        :disabled="gridStore.currentPage === 1"
        @click="gridStore.setPage(gridStore.currentPage - 1)"
      >
        <template #icon>
          <ChevronLeft :size="14" />
        </template>
      </Button>

      <!-- Pages -->
      <template v-for="page in visiblePages" :key="page">
        <span
          v-if="page === '...'"
          class="text-text-tertiary flex h-7 w-7 items-center justify-center text-[12px]"
          >…</span
        >
        <Button
          v-else
          size="small"
          class="h-7! w-7! p-0! text-[12px]!"
          :variant="page === gridStore.currentPage ? 'outlined' : 'text'"
          :severity="page === gridStore.currentPage ? 'primary' : 'secondary'"
          @click="gridStore.setPage(page as number)"
        >
          {{ page }}
        </Button>
      </template>

      <!-- Next -->
      <Button
        variant="text"
        severity="secondary"
        size="small"
        class="h-7! w-7! p-0!"
        :disabled="gridStore.currentPage === gridStore.totalPages"
        @click="gridStore.setPage(gridStore.currentPage + 1)"
      >
        <template #icon>
          <ChevronRight :size="14" />
        </template>
      </Button>
      <!-- Last -->
      <Button
        variant="text"
        severity="secondary"
        size="small"
        class="h-7! w-7! p-0!"
        :disabled="gridStore.currentPage === gridStore.totalPages"
        @click="gridStore.setPage(gridStore.totalPages)"
      >
        <template #icon>
          <ChevronsRight :size="14" />
        </template>
      </Button>
    </div>

    <!-- Right: Rows per page -->
    <div class="flex items-center gap-1.5">
      <span>Rows per page</span>
      <Select
        :model-value="gridStore.rowsPerPage"
        :options="rowsOptions"
        optionLabel="label"
        optionValue="value"
        size="small"
        class="w-22 text-xs"
        @update:model-value="(val) => gridStore.setRowsPerPage(Number(val))"
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
