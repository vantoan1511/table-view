<script setup lang="ts">
import ContextMenu from '@/components/ui/ContextMenu.vue';
import Skeleton from '@/components/ui/Skeleton.vue';
import GridCell from './GridCell.vue';
import GridHeaderCell from './GridHeaderCell.vue';
import GridToolbar from './GridToolbar.vue';
import Pagination from './Pagination.vue';

import { useClickOutside } from '@/composables/useClickOutside';
import { useContextMenu } from '@/composables/useContextMenu';
import { useGridResizing } from '@/composables/useGridResizing';

import { useGridStore } from '@/stores/grid';
import { useLayoutStore } from '@/stores/layout';

import type { GridColumn } from '@/types';

import { Check, Copy, Plus, RefreshCw, Trash2, Wrench, X } from 'lucide-vue-next';
import { computed, ref, onMounted, onUnmounted, watch } from 'vue';

import { NativeService } from '@/services/native';

const gridStore = useGridStore();
const layoutStore = useLayoutStore();

const visibleColumns = computed(() =>
  gridStore.columns.filter((column) => gridStore.columnVisibility[column.name] !== false)
);

// ─── Column Resizing ───────────────────────────────────────────────────────
const { onResizeStart, MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH, DEFAULT_COLUMN_WIDTH } = useGridResizing(
  {
    onResize: (colName, width) => gridStore.setColumnWidth(colName, width)
  }
);

// ─── Cell Interaction ──────────────────────────────────────────────────────
const onCellClick = (rowIdx: number, col: GridColumn) => {
  gridStore.setSelectedCell(rowIdx, col);

  // Auto-switch to Value tab if bottom panel is visible
  const panel = layoutStore.bottomPanel;
  if (layoutStore.isBottomVisible && panel && panel.activeTabId !== 'value') {
    panel.activeTabId = 'value';
  }
};

const startEdit = (rowIdx: number, col: GridColumn) => {
  if (col.isPrimaryKey) return;
  gridStore.startEditCell(rowIdx, col);
};

// ─── Global Click Handling ──────────────────────────────────────────────────
const gridRef = ref<HTMLElement | null>(null);
useClickOutside(gridRef, () => {
  gridStore.clearSelectedCell();
  gridStore.cancelEditCell();
}, [
  '.value-viewer-panel',
  '.grid-toolbar',
  '.context-menu',
  '.modal-container',
  '.modal-backdrop',
  '#new-connection-modal'
]);

// ─── Context Menu ──────────────────────────────────────────────────────────
const { contextMenu, openContextMenu, closeContextMenu } = useContextMenu<number>();

const onGridContextMenu = (event: MouseEvent, rowIdx: number) => {
  openContextMenu(event, rowIdx);
  if (rowIdx >= 0 && !gridStore.selectedRowIndices.has(rowIdx)) {
    gridStore.clearSelection();
    gridStore.toggleRowSelection(rowIdx, event);
  }
};

const handleContextAction = (action: string) => {
  closeContextMenu();
  if (action === 'addRow') gridStore.createNewRow();
  else if (action === 'deleteRows') {
    if (gridStore.selectedRowIndices.size > 0) {
      gridStore.deleteRows(Array.from(gridStore.selectedRowIndices)).catch((err) => {
        NativeService.os.showMessageBox(
          'Error',
          'Failed to delete row(s): ' + err.message,
          'ERROR'
        );
      });
    }
  } else if (action === 'alterTable') gridStore.showAlterTableDialog = true;
  else if (action === 'refresh') gridStore.loadTable(gridStore.activeTableName);
  else if (action === 'copyCell') {
    if (gridStore.selectedCell) {
      const { rowIndex, column } = gridStore.selectedCell;
      const value = gridStore.rows[rowIndex]?.[column.name];
      if (value !== undefined) {
        navigator.clipboard.writeText(value === null ? 'NULL' : String(value));
      }
    }
  }
};

// ─── Copy Keyboard Shortcut ─────────────────────────────────────────────────
const handleKeyDown = (event: KeyboardEvent) => {
  const activeEl = document.activeElement;
  if (
    activeEl &&
    (activeEl.tagName === 'INPUT' ||
      activeEl.tagName === 'TEXTAREA' ||
      activeEl.tagName === 'SELECT' ||
      activeEl.getAttribute('contenteditable') === 'true')
  ) {
    return;
  }

  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'c') {
    if (gridStore.selectedCell) {
      const { rowIndex, column } = gridStore.selectedCell;
      const value = gridStore.rows[rowIndex]?.[column.name];
      if (value !== undefined) {
        event.preventDefault();
        navigator.clipboard.writeText(value === null ? 'NULL' : String(value));
      }
    }
  }
};

// ─── Virtual Scrolling ──────────────────────────────────────────────────────
const ROW_HEIGHT = 33;
const BUFFER_ROWS = 15;

const scrollContainerRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const viewportHeight = ref(400);

let resizeObserver: ResizeObserver | null = null;

const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement;
  scrollTop.value = target.scrollTop;
};

const startIndex = computed(() => {
  return Math.max(0, Math.floor(scrollTop.value / ROW_HEIGHT) - BUFFER_ROWS);
});

const endIndex = computed(() => {
  return Math.min(
    gridStore.rows.length,
    Math.ceil((scrollTop.value + viewportHeight.value) / ROW_HEIGHT) + BUFFER_ROWS
  );
});

const visibleRows = computed(() => {
  return gridStore.rows.slice(startIndex.value, endIndex.value);
});

const topSpacerHeight = computed(() => {
  return startIndex.value * ROW_HEIGHT;
});

const bottomSpacerHeight = computed(() => {
  return (gridStore.rows.length - endIndex.value) * ROW_HEIGHT;
});

// Reset scroll on table data changes
watch(
  () => gridStore.rows,
  () => {
    scrollTop.value = 0;
    if (scrollContainerRef.value) {
      scrollContainerRef.value.scrollTop = 0;
    }
  }
);

onMounted(() => {
  if (scrollContainerRef.value) {
    scrollTop.value = scrollContainerRef.value.scrollTop;
    viewportHeight.value = scrollContainerRef.value.clientHeight;

    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        viewportHeight.value = entry.contentRect.height || entry.target.clientHeight;
      }
    });
    resizeObserver.observe(scrollContainerRef.value);
  }
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <div class="bg-surface flex h-full min-h-0 w-full flex-col" ref="gridRef">
    <GridToolbar />

    <div
      ref="scrollContainerRef"
      class="relative min-h-0 flex-1 overflow-auto"
      @scroll="handleScroll"
      @contextmenu.prevent="onGridContextMenu($event, -1)"
      @click.self="gridStore.clearSelectedCell"
    >
      <table class="table-fixed border-collapse text-[12px] font-(--font-mono)">
        <thead class="sticky top-0 z-20">
          <tr class="bg-grid-header border-grid-border border-b">
            <th
              class="text-text-tertiary border-grid-border bg-grid-header sticky left-0 z-30 w-12 border-r px-3 py-2 text-right text-[11px] font-medium"
              @click="gridStore.toggleSelectAllRows"
            >
              #
            </th>
            <GridHeaderCell
              v-for="col in visibleColumns"
              :key="col.name"
              :column="col"
              :is-sorted="gridStore.sortColumn === col.name"
              :sort-direction="gridStore.sortDirection"
              :column-width="gridStore.columnWidths[col.name]"
              :min-width="MIN_COLUMN_WIDTH"
              :max-width="MAX_COLUMN_WIDTH"
              :default-width="DEFAULT_COLUMN_WIDTH"
              @sort="gridStore.toggleSort(col.name)"
              @resize="onResizeStart(col.name, $event)"
            />
          </tr>
        </thead>

        <tbody>
          <!-- Loading skeletons -->
          <template v-if="gridStore.isLoading">
            <tr v-for="i in 10" :key="'skel-' + i" class="border-grid-border border-b">
              <td class="border-grid-border sticky left-0 z-10 border-r bg-inherit px-3 py-1.5">
                <Skeleton height="1.25rem" />
              </td>
              <template v-if="gridStore.columns.length > 0">
                <td
                  v-for="col in visibleColumns"
                  :key="'skel-' + col.name"
                  class="border-grid-border border-r px-3 py-1.5"
                >
                  <Skeleton height="1.25rem" />
                </td>
              </template>
              <template v-else>
                <td
                  v-for="j in 5"
                  :key="'skel-col-' + j"
                  class="border-grid-border border-r px-3 py-1.5"
                >
                  <Skeleton height="1.25rem" />
                </td>
              </template>
            </tr>
          </template>

          <template v-else>
            <!-- Virtual scroll top spacer -->
            <tr v-if="topSpacerHeight > 0" :style="{ height: `${topSpacerHeight}px` }">
              <td
                :colspan="visibleColumns.length + 1"
                style="padding: 0; border: none; background: transparent"
              ></td>
            </tr>

            <!-- Rendered visible subset of rows -->
            <template v-for="(row, i) in visibleRows" :key="startIndex + i">
              <tr
                class="border-grid-border hover:bg-grid-row-hover border-b transition-colors"
                :class="{
                  'bg-grid-row-alt':
                    (startIndex + i) % 2 === 1 && !gridStore.selectedRowIndices.has(startIndex + i),
                  'bg-primary/10!': gridStore.selectedRowIndices.has(startIndex + i)
                }"
                @contextmenu.prevent.stop="onGridContextMenu($event, startIndex + i)"
              >
                <td
                  class="text-text-tertiary border-grid-border sticky left-0 z-10 cursor-pointer border-r bg-inherit px-3 py-1.5 text-right text-[11px] tabular-nums backdrop-blur-sm select-none"
                  :class="{
                    'bg-primary/20! text-primary! font-semibold': gridStore.selectedRowIndices.has(
                      startIndex + i
                    )
                  }"
                  @click="gridStore.toggleRowSelection(startIndex + i, $event)"
                >
                  {{ (gridStore.currentPage - 1) * gridStore.rowsPerPage + (startIndex + i) + 1 }}
                </td>

                <GridCell
                  v-for="col in visibleColumns"
                  :key="col.name"
                  :row-index="startIndex + i"
                  :column="col"
                  :value="
                    gridStore.newRowIdx === startIndex + i
                      ? gridStore.newRowData[col.name]
                      : row[col.name]
                  "
                  :is-selected="
                    gridStore.selectedCell?.rowIndex === startIndex + i &&
                    gridStore.selectedCell?.column.name === col.name
                  "
                  :is-editing="
                    gridStore.editingCell?.rowIndex === startIndex + i &&
                    gridStore.editingCell?.column.name === col.name
                  "
                  :is-new-row="gridStore.newRowIdx === startIndex + i"
                  :validation-error="
                    gridStore.newRowIdx === startIndex + i
                      ? gridStore.newRowErrors[col.name]
                      : undefined
                  "
                  :column-width="gridStore.columnWidths[col.name]"
                  :min-width="MIN_COLUMN_WIDTH"
                  :max-width="MAX_COLUMN_WIDTH"
                  :default-width="DEFAULT_COLUMN_WIDTH"
                  @click="onCellClick(startIndex + i, col)"
                  @dblclick="startEdit(startIndex + i, col)"
                  @update:value="
                    gridStore.newRowIdx === startIndex + i
                      ? ((gridStore.newRowData[col.name] = $event),
                        gridStore.validateNewRowCell(col.name, $event))
                      : gridStore.editingCell
                        ? (gridStore.editingCell.currentValue = $event)
                        : null
                  "
                  @save="
                    gridStore.newRowIdx === startIndex + i
                      ? gridStore.saveNewRow()
                      : gridStore.saveEditCell()
                  "
                  @cancel="
                    gridStore.newRowIdx === startIndex + i
                      ? gridStore.cancelNewRow()
                      : gridStore.cancelEditCell()
                  "
                />
              </tr>

              <!-- Save/Cancel action row for new row -->
              <tr v-if="gridStore.newRowIdx === startIndex + i" class="border-grid-border border-b">
                <td
                  :colspan="visibleColumns.length + 1"
                  class="border-grid-border border-r px-2 py-1"
                >
                  <div class="flex items-center justify-end gap-1.5">
                    <button
                      @click.stop="gridStore.saveNewRow"
                      class="text-success hover:bg-success/10 flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-medium"
                      title="Save row"
                    >
                      <Check :size="12" />
                      <span>Save</span>
                    </button>
                    <button
                      @click.stop="gridStore.cancelNewRow"
                      class="text-danger hover:bg-danger/10 flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-medium"
                      title="Discard"
                    >
                      <X :size="12" />
                      <span>Discard</span>
                    </button>
                  </div>
                </td>
              </tr>
            </template>

            <!-- Virtual scroll bottom spacer -->
            <tr v-if="bottomSpacerHeight > 0" :style="{ height: `${bottomSpacerHeight}px` }">
              <td
                :colspan="visibleColumns.length + 1"
                style="padding: 0; border: none; background: transparent"
              ></td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <Pagination />

    <ContextMenu
      :show="contextMenu.show"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
    >
      <button
        @click="handleContextAction('copyCell')"
        class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px] disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="!gridStore.selectedCell"
      >
        <Copy :size="14" class="text-text-secondary" /> <span>Copy Cell Value</span>
      </button>
      <div class="bg-border my-1.5 h-px w-full"></div>
      <button
        @click="handleContextAction('addRow')"
        class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      >
        <Plus :size="14" class="text-text-secondary" /> <span>Add Row</span>
      </button>
      <button
        @click="handleContextAction('deleteRows')"
        class="text-danger hover:bg-danger-light hover:text-danger flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
        :disabled="gridStore.selectedRowIndices.size === 0"
      >
        <Trash2 :size="14" /> <span>Delete Row(s)</span>
      </button>
      <div class="bg-border my-1.5 h-px w-full"></div>
      <button
        @click="handleContextAction('alterTable')"
        class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      >
        <Wrench :size="14" class="text-text-secondary" /> <span>Alter Table...</span>
      </button>
      <button
        @click="handleContextAction('refresh')"
        class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      >
        <RefreshCw :size="14" class="text-text-secondary" /> <span>Refresh</span>
      </button>
    </ContextMenu>
  </div>
</template>
