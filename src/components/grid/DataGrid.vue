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
import { NativeService } from '@/services/native';
import { useGridStore } from '@/stores/grid';
import { useLayoutStore } from '@/stores/layout';
import { Check, Plus, RefreshCw, Trash2, Wrench, X } from 'lucide-vue-next';
import { computed, ref } from 'vue';

import type { GridColumn } from '@/types';

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
};
</script>

<template>
  <div class="bg-surface flex h-full min-h-0 w-full flex-col" ref="gridRef">
    <GridToolbar />

    <div
      class="relative min-h-0 flex-1 overflow-auto"
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
          <template v-for="(row, rowIdx) in gridStore.rows" :key="rowIdx">
            <tr
              class="border-grid-border hover:bg-grid-row-hover border-b transition-colors"
              :class="{
                'bg-grid-row-alt': rowIdx % 2 === 1 && !gridStore.selectedRowIndices.has(rowIdx),
                'bg-primary/10!': gridStore.selectedRowIndices.has(rowIdx)
              }"
              @contextmenu.prevent.stop="onGridContextMenu($event, rowIdx)"
            >
              <td
                class="text-text-tertiary border-grid-border sticky left-0 z-10 cursor-pointer border-r bg-inherit px-3 py-1.5 text-right text-[11px] tabular-nums backdrop-blur-sm select-none"
                :class="{
                  'bg-primary/20! text-primary! font-semibold':
                    gridStore.selectedRowIndices.has(rowIdx)
                }"
                @click="gridStore.toggleRowSelection(rowIdx, $event)"
              >
                {{ (gridStore.currentPage - 1) * gridStore.rowsPerPage + rowIdx + 1 }}
              </td>

              <GridCell
                v-for="col in visibleColumns"
                :key="col.name"
                :row-index="rowIdx"
                :column="col"
                :value="
                  gridStore.newRowIdx === rowIdx ? gridStore.newRowData[col.name] : row[col.name]
                "
                :is-selected="
                  gridStore.selectedCell?.rowIndex === rowIdx &&
                  gridStore.selectedCell?.column.name === col.name
                "
                :is-editing="
                  gridStore.editingCell?.rowIndex === rowIdx &&
                  gridStore.editingCell?.column.name === col.name
                "
                :is-new-row="gridStore.newRowIdx === rowIdx"
                :column-width="gridStore.columnWidths[col.name]"
                :min-width="MIN_COLUMN_WIDTH"
                :max-width="MAX_COLUMN_WIDTH"
                :default-width="DEFAULT_COLUMN_WIDTH"
                @click="onCellClick(rowIdx, col)"
                @dblclick="startEdit(rowIdx, col)"
                @update:value="
                  gridStore.newRowIdx === rowIdx
                    ? (gridStore.newRowData[col.name] = $event)
                    : gridStore.editingCell
                      ? (gridStore.editingCell.currentValue = $event)
                      : null
                "
                @save="
                  gridStore.newRowIdx === rowIdx ? gridStore.saveNewRow() : gridStore.saveEditCell()
                "
                @cancel="
                  gridStore.newRowIdx === rowIdx
                    ? gridStore.cancelNewRow()
                    : gridStore.cancelEditCell()
                "
              />

              <!-- Save/Cancel Floating Buttons for New Row -->
              <div
                v-if="gridStore.newRowIdx === rowIdx"
                class="absolute right-2 z-40 mt-1 flex justify-end"
                :style="{ top: '100%' }"
              >
                <div
                  class="border-border bg-surface flex items-center gap-1 rounded-md border px-1.5 py-1 shadow-lg"
                >
                  <button
                    @click.stop="gridStore.saveNewRow"
                    class="text-success hover:bg-success/10 rounded p-1"
                    title="Save row"
                  >
                    <Check :size="14" />
                  </button>
                  <button
                    @click.stop="gridStore.cancelNewRow"
                    class="text-danger hover:bg-danger/10 rounded p-1"
                    title="Cancel"
                  >
                    <X :size="14" />
                  </button>
                </div>
              </div>
            </tr>
          </template>
        </template>
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
