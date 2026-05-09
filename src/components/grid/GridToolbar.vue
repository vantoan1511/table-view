<script setup lang="ts">
import AlterTableDialog from '@/components/ui/AlterTableDialog.vue';
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';
import ContextMenu from '@/components/ui/ContextMenu.vue';
import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useToastStore } from '@/stores/toast';
import * as Neutralino from '@neutralinojs/lib';
import { computed, ref, watch } from 'vue';

import {
  ChevronDown,
  Columns3,
  Download,
  Filter,
  LayoutGrid,
  MoreHorizontal,
  Plus,
  RefreshCw,
  Trash2,
  Wrench
} from 'lucide-vue-next';

const gridStore = useGridStore();
const toastStore = useToastStore();
const connectionsStore = useConnectionsStore();

const selectedCount = computed(() => gridStore.selectedRowIndices.size);
const showDeleteConfirm = ref(false);

// ─── Dropdowns state ────────────────────────────────────────────────────────
const showColumnsMenu = ref(false);
const columnsMenuPos = ref({ x: 0, y: 0 });
const showRowsMenu = ref(false);
const rowsMenuPos = ref({ x: 0, y: 0 });
const showMoreMenu = ref(false);
const moreMenuPos = ref({ x: 0, y: 0 });
const showSearchPopup = ref(false);
const searchPopupPos = ref({ x: 0, y: 0 });

// Initialize column visibility tracking in the store
watch(
  () => gridStore.columns,
  (cols) => {
    cols.forEach((c) => {
      if (gridStore.columnVisibility[c.name] === undefined) {
        gridStore.columnVisibility[c.name] = true;
      }
    });
  },
  { immediate: true }
);

const toggleColumnsMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  columnsMenuPos.value = { x: rect.left, y: rect.bottom + 5 };
  showColumnsMenu.value = !showColumnsMenu.value;
  showMoreMenu.value = false;
};

const toggleRowsMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  rowsMenuPos.value = { x: rect.left, y: rect.bottom + 5 };
  showRowsMenu.value = !showRowsMenu.value;
  showMoreMenu.value = false;
};

const toggleMoreMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  moreMenuPos.value = { x: rect.right - 180, y: rect.bottom + 5 };
  showMoreMenu.value = !showMoreMenu.value;
  showSearchPopup.value = false;
};

const toggleSearchPopup = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  searchPopupPos.value = { x: Math.max(10, rect.left - 150), y: rect.bottom + 5 };
  showSearchPopup.value = !showSearchPopup.value;
  showMoreMenu.value = false;
};

const setRowsPerPage = (count: number) => {
  gridStore.setRowsPerPage(count);
  showRowsMenu.value = false;
};

// ─── Add Row Inline ──────────────────────────────────────────────────────
const handleInsert = async () => {
  gridStore.createNewRow();
};

// ─── Refresh ──────────────────────────────────────────────────────────────────────
const handleRefresh = () => {
  gridStore.loadTable(gridStore.activeTableName);
};

// ─── Delete Confirmation ──────────────────────────────────────────────────────
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
  }
};

const confirmAlterTable = async (operations: any[]) => {
  if (operations.length === 0) {
    gridStore.showAlterTableDialog = false;
    return;
  }
  try {
    await gridStore.alterTable(gridStore.activeTableName, operations);
    gridStore.showAlterTableDialog = false;
    gridStore.loadTable(gridStore.activeTableName);
    toastStore.addToast({
      title: 'Table Altered',
      message: 'Table altered successfully.',
      severity: 'success',
      variation: 'filled',
      position: 'bottom-center'
    });
  } catch (err: any) {
    toastStore.addToast({
      title: 'Table Alteration Failed',
      message: err.message,
      severity: 'error',
      variation: 'filled',
      position: 'bottom-center'
    });
  }
};

const handleExport = async () => {
  if (!gridStore.activeTableName) return;

  try {
    const defaultName = `${gridStore.activeTableName}_export.csv`;
    const path = await Neutralino.os.showSaveDialog('Export as CSV', {
      defaultPath: defaultName,
      filters: [{ name: 'CSV files', extensions: ['csv'] }]
    });

    if (path) {
      const reqId = Date.now().toString();

      const onResult = (evt: any) => {
        const payload = evt.detail;
        if (payload.reqId === reqId) {
          if (payload.success) {
            toastStore.addToast({
              title: 'Export Success',
              message: 'Data exported successfully to ' + path,
              severity: 'success'
            });
          } else {
            toastStore.addToast({
              title: 'Export Error',
              message: payload.error,
              severity: 'error'
            });
          }
          Neutralino.events.off('dbBridge.exportCSVResult', onResult);
        }
      };

      Neutralino.events.on('dbBridge.exportCSVResult', onResult);
      Neutralino.extensions.dispatch(
        'com.github.vantoan1511.table-view.db-bridge',
        'dbBridge.exportCSV',
        {
          reqId,
          connectionId: connectionsStore.activeConnectionId,
          tableName: gridStore.activeTableName,
          exportPath: path
        }
      );
    }
  } catch (err) {
    console.error('Export cancelled or failed', err);
  }
};

// Custom directive for auto-focusing
const vFocus = {
  mounted: (el: HTMLElement) => el.focus()
};
</script>

<template>
  <div
    class="border-border bg-surface @container flex min-h-[48px] items-center justify-between gap-2 border-b px-4 py-2"
  >
    <!-- Left: Table name + row actions -->
    <div class="flex min-w-0 items-center gap-2">
      <div class="mr-1 flex shrink-0 items-center gap-2">
        <h2
          class="text-text-primary max-w-[80px] truncate text-[14px] font-semibold @[400px]:max-w-[150px] @[850px]:max-w-none"
          :title="gridStore.activeTableName"
        >
          {{ gridStore.activeTableName }}
        </h2>
      </div>

      <!-- Add Row (Hidden at Level 3, text only at Level 1) -->
      <button
        id="btn-add-row"
        class="border-border text-text-secondary hover:bg-hover hover:border-border-strong group relative hidden shrink-0 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] transition-colors @[500px]:flex"
        @click="handleInsert"
      >
        <Plus :size="13" class="text-success" />
        <span class="hidden @[850px]:inline">Add Row</span>

        <!-- Tooltip -->
        <div
          class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
        >
          Insert a new row
          <div
            class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
          ></div>
        </div>
      </button>

      <!-- Selection Controls (Only visible at Level 1 & 2) -->
      <div
        v-if="selectedCount > 0"
        class="bg-border hidden shrink-0 items-center gap-px rounded-lg p-px @[850px]:flex"
      >
        <div
          class="bg-primary/10 text-primary flex items-center rounded-l-[7px] px-2 py-[5px] text-[11px] font-medium whitespace-nowrap"
        >
          {{ selectedCount }} <span class="ml-1 hidden @[900px]:inline">selected</span>
        </div>
        <button
          class="bg-surface text-text-secondary hover:text-text-primary hover:bg-hover group relative flex items-center px-2 py-[5px] text-[11px] transition-colors"
          @click="gridStore.selectAllRows"
        >
          <span>All</span>
          <!-- Tooltip -->
          <div
            class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
          >
            Select All
            <div
              class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
            ></div>
          </div>
        </button>
        <button
          class="bg-surface text-text-secondary hover:text-text-primary hover:bg-hover group relative flex items-center rounded-r-[7px] px-2 py-[5px] text-[11px] transition-colors"
          @click="gridStore.clearSelection"
        >
          <span>None</span>
          <!-- Tooltip -->
          <div
            class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
          >
            Deselect All
            <div
              class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
            ></div>
          </div>
        </button>
      </div>

      <!-- Delete Selected (Icon only at Level 2, Hidden in Level 3) -->
      <button
        v-if="selectedCount > 0"
        id="btn-delete-rows"
        class="border-danger/40 text-danger hover:bg-danger-light hover:border-danger/60 group relative hidden shrink-0 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] transition-colors @[500px]:flex"
        @click="promptDelete"
      >
        <Trash2 :size="13" />
        <span class="hidden @[850px]:inline">Delete</span>
        <!-- Tooltip -->
        <div
          class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
        >
          Delete {{ selectedCount }} selected row(s)
          <div
            class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
          ></div>
        </div>
      </button>
    </div>

    <!-- Right: Actions -->
    <div class="flex min-w-0 items-center gap-2">
      <!-- Search: Level 1 & 2 (Inline) -->
      <div class="group relative hidden max-w-[200px] min-w-[120px] flex-1 @[500px]:block">
        <Filter
          :size="13"
          class="text-text-tertiary group-focus-within:text-primary absolute top-1/2 left-2.5 -translate-y-1/2 transition-colors"
        />
        <input
          v-model="gridStore.filterText"
          type="text"
          placeholder="Search..."
          class="border-border bg-muted focus:bg-surface focus:border-primary focus:ring-primary w-full rounded-lg border py-1.5 pr-3 pl-8 text-[12px] transition-all outline-none focus:ring-1"
          @keydown.enter="handleRefresh"
        />
      </div>

      <!-- Search: Level 3 (Popup Icon) -->
      <div class="flex items-center @[500px]:hidden">
        <button
          class="border-border text-text-secondary hover:bg-hover group relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border transition-colors"
          :class="{ 'bg-primary/10 border-primary text-primary': showSearchPopup }"
          @click.stop="toggleSearchPopup"
        >
          <Filter :size="14" />
          <!-- Tooltip -->
          <div
            v-if="!showSearchPopup"
            class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
          >
            Search
            <div
              class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
            ></div>
          </div>
        </button>

        <ContextMenu
          :show="showSearchPopup"
          :x="searchPopupPos.x"
          :y="searchPopupPos.y"
          width-class="w-64"
          @close="showSearchPopup = false"
        >
          <div class="px-3 py-2">
            <div class="relative">
              <Filter
                :size="13"
                class="text-text-tertiary absolute top-1/2 left-2.5 -translate-y-1/2"
              />
              <input
                v-model="gridStore.filterText"
                v-focus
                type="text"
                placeholder="Search rows..."
                class="border-border focus:border-primary focus:ring-primary w-full rounded-lg border py-1.5 pr-3 pl-8 text-[12px] outline-none focus:ring-1"
                @keydown.enter="
                  handleRefresh();
                  showSearchPopup = false;
                "
              />
            </div>
          </div>
        </ContextMenu>
      </div>

      <!-- Level 1 & 2 Actions Group -->
      <div class="hidden items-center gap-1.5 @[500px]:flex">
        <!-- Columns -->
        <button
          class="border-border text-text-secondary hover:bg-hover group relative flex cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] transition-colors"
          @click.stop="toggleColumnsMenu"
        >
          <Columns3 :size="13" />
          <span class="hidden @[850px]:inline">Columns</span>
          <!-- Tooltip -->
          <div
            class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
          >
            Configure Columns
            <div
              class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
            ></div>
          </div>
        </button>

        <!-- Export -->
        <button
          class="border-border text-text-secondary hover:bg-hover group relative flex cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] transition-colors"
          @click="handleExport"
        >
          <Download :size="13" />
          <span class="hidden @[850px]:inline">Export</span>
          <!-- Tooltip -->
          <div
            class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
          >
            Export Data
            <div
              class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
            ></div>
          </div>
        </button>

        <!-- Row Count -->
        <button
          class="border-border text-text-secondary hover:bg-hover flex cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] transition-colors"
          @click.stop="toggleRowsMenu"
        >
          <span class="whitespace-nowrap"
            >{{ gridStore.rowsPerPage }} <span class="hidden @[850px]:inline">rows</span></span
          >
          <ChevronDown :size="12" />
        </button>

        <!-- Divider -->
        <div class="bg-border mx-0.5 hidden h-5 w-px @[600px]:block" />

        <!-- Alter Table -->
        <button
          class="border-border text-text-secondary hover:bg-hover group relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border transition-colors"
          @click="gridStore.showAlterTableDialog = true"
        >
          <Wrench :size="14" />
          <!-- Tooltip -->
          <div
            class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
          >
            Alter Table Structure
            <div
              class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
            ></div>
          </div>
        </button>

        <!-- Refresh -->
        <button
          class="border-border text-text-secondary hover:bg-hover group relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border transition-colors"
          @click="handleRefresh"
        >
          <RefreshCw :size="14" />
          <!-- Tooltip -->
          <div
            class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
          >
            Refresh Table
            <div
              class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
            ></div>
          </div>
        </button>
      </div>

      <!-- Level 3: Speed Dial / More Button -->
      <div class="flex items-center @[500px]:hidden">
        <button
          class="border-border text-text-secondary hover:bg-hover group relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border transition-colors"
          @click.stop="toggleMoreMenu"
        >
          <MoreHorizontal :size="16" />
          <!-- Tooltip -->
          <div
            v-if="!showMoreMenu"
            class="bg-surface border-border text-text-primary pointer-events-none absolute top-full left-1/2 z-50 mt-2 -translate-x-1/2 -translate-y-1 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover:translate-y-0 group-hover:opacity-100"
          >
            More Actions
            <div
              class="bg-surface border-border absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-t border-l"
            ></div>
          </div>
        </button>

        <ContextMenu
          :show="showMoreMenu"
          :x="moreMenuPos.x"
          :y="moreMenuPos.y"
          @close="showMoreMenu = false"
        >
          <button
            class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
            @click="
              handleInsert();
              showMoreMenu = false;
            "
          >
            <Plus :size="14" class="text-success" /> <span>Add Row</span>
          </button>
          <button
            class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
            @click="
              handleRefresh();
              showMoreMenu = false;
            "
          >
            <RefreshCw :size="14" /> <span>Refresh</span>
          </button>
          <button
            class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
            @click="
              gridStore.showAlterTableDialog = true;
              showMoreMenu = false;
            "
          >
            <Wrench :size="14" /> <span>Alter Table</span>
          </button>
          <button
            class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
            @click="toggleColumnsMenu"
          >
            <Columns3 :size="14" /> <span>Columns...</span>
          </button>
          <button
            class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
            @click="toggleRowsMenu"
          >
            <LayoutGrid :size="14" /> <span>Rows Per Page...</span>
          </button>
          <button
            class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
            @click="
              handleExport;
              showMoreMenu = false;
            "
          >
            <Download :size="14" /> <span>Export CSV</span>
          </button>
          <div v-if="selectedCount > 0" class="bg-border my-1 h-px"></div>
          <button
            v-if="selectedCount > 0"
            class="hover:bg-hover text-danger flex w-full items-center gap-2 px-3 py-2 text-[12px]"
            @click="
              promptDelete;
              showMoreMenu = false;
            "
          >
            <Trash2 :size="14" /> <span>Delete Selected ({{ selectedCount }})</span>
          </button>
        </ContextMenu>
      </div>
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

  <!-- Alter Table Dialog -->
  <AlterTableDialog
    v-if="gridStore.showAlterTableDialog"
    :tableName="gridStore.activeTableName"
    @close="gridStore.showAlterTableDialog = false"
    @apply="confirmAlterTable"
  />

  <!-- Columns Visibility ContextMenu -->
  <ContextMenu
    :show="showColumnsMenu"
    :x="columnsMenuPos.x"
    :y="columnsMenuPos.y"
    @close="showColumnsMenu = false"
  >
    <div class="text-text-tertiary px-3 py-1.5 text-[11px] font-semibold tracking-wider uppercase">
      Columns
    </div>
    <div class="max-h-64 overflow-y-auto">
      <button
        v-for="col in gridStore.columns"
        :key="col.name"
        class="hover:bg-hover flex w-full cursor-pointer items-center gap-2 px-3 py-1.5 text-left text-[12px]"
        @click="gridStore.toggleColumnVisibility(col.name)"
      >
        <span
          class="border-border flex h-3 w-3 shrink-0 items-center justify-center rounded border"
          :class="
            gridStore.columnVisibility[col.name] !== false
              ? 'bg-primary border-primary'
              : 'bg-surface'
          "
        >
          <svg
            v-if="gridStore.columnVisibility[col.name] !== false"
            class="h-2 w-2 text-white"
            viewBox="0 0 12 12"
            fill="none"
          >
            <polyline
              points="1,6 4,9 11,2"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            />
          </svg>
        </span>
        <span class="truncate">{{ col.name }}</span>
        <span class="text-text-tertiary ml-auto text-[10px]">{{ col.dataType }}</span>
      </button>
    </div>
  </ContextMenu>

  <!-- Rows Per Page ContextMenu -->
  <ContextMenu
    :show="showRowsMenu"
    :x="rowsMenuPos.x"
    :y="rowsMenuPos.y"
    @close="showRowsMenu = false"
  >
    <div class="text-text-tertiary px-3 py-1.5 text-[11px] font-semibold tracking-wider uppercase">
      Rows per page
    </div>
    <button
      v-for="count in [25, 50, 100, 250, 500, 1000]"
      :key="count"
      class="hover:bg-hover flex w-full cursor-pointer items-center justify-between px-3 py-1.5 text-[12px]"
      @click="setRowsPerPage(count)"
    >
      <span>{{ count }}</span>
      <span v-if="gridStore.rowsPerPage === count" class="text-primary">✓</span>
    </button>
  </ContextMenu>
</template>
