<script setup lang="ts">
import AlterTableDialog from '@/components/ui/AlterTableDialog.vue';
import DropdownMenu, { type DropdownValue } from '@/components/ui/DropdownMenu.vue';

import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useToastStore } from '@/stores/toast';

import * as Neutralino from '@neutralinojs/lib';
import {
  Columns3,
  Download,
  Filter,
  LayoutGrid,
  MoreHorizontal,
  RefreshCw,
  Wrench,
  X
} from '@lucide/vue';
import { computed, nextTick, ref, watch } from 'vue';

const gridStore = useGridStore();
const toastStore = useToastStore();
const connectionsStore = useConnectionsStore();

const rowsOptions = [25, 50, 100, 250, 500, 1000].map((count) => ({
  label: `${count}`,
  value: count
}));

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

const setRowsPerPage = (count: DropdownValue) => {
  gridStore.setRowsPerPage(Number(count));
};

// ─── Refresh ──────────────────────────────────────────────────────────────────────
const handleRefresh = () => {
  gridStore.loadTable(gridStore.activeTableName, undefined, undefined, undefined, true);
};

// ─── Filter ──────────────────────────────────────────────────────────────────────
const handleFilter = () => {
  gridStore.currentPage = 1;
  gridStore.loadTable(gridStore.activeTableName);
};

const clearFilter = () => {
  gridStore.filterText = '';
  handleFilter();
};

// ─── Filter Autocomplete ────────────────────────────────────────────────────────
const filterInputRef = ref<HTMLInputElement | null>(null);
const showSuggestions = ref(false);
const activeSuggestionIndex = ref(0);
const activeToken = ref('');

const getActiveToken = (text: string, cursorOffset: number) => {
  const textBeforeCursor = text.slice(0, cursorOffset);
  const words = textBeforeCursor.split(/[\s(),;=!><]+/);
  return words[words.length - 1] || '';
};

const updateCursorOffset = () => {
  const inputEl = filterInputRef.value;
  if (!inputEl) return;
  const cursorOffset = inputEl.selectionStart || 0;
  activeToken.value = getActiveToken(gridStore.filterText || '', cursorOffset);
};

const handleInput = () => {
  showSuggestions.value = true;
  activeSuggestionIndex.value = 0;
  updateCursorOffset();
};

const handleBlur = () => {
  setTimeout(() => {
    showSuggestions.value = false;
  }, 200);
};

const filteredSuggestions = computed(() => {
  const token = activeToken.value.toLowerCase();

  const cols = gridStore.columns.map((c) => ({
    label: c.name,
    value: c.name,
    type: 'column' as const,
    dataType: c.dataType,
    description: 'Column'
  }));

  const ops = [
    {
      label: 'AND',
      value: 'AND ',
      type: 'operator' as const,
      description: 'Logical AND',
      dataType: undefined
    },
    {
      label: 'OR',
      value: 'OR ',
      type: 'operator' as const,
      description: 'Logical OR',
      dataType: undefined
    },
    {
      label: 'LIKE',
      value: 'LIKE ',
      type: 'operator' as const,
      description: 'Pattern matching',
      dataType: undefined
    },
    {
      label: 'ILIKE',
      value: 'ILIKE ',
      type: 'operator' as const,
      description: 'Case-insensitive LIKE',
      dataType: undefined
    },
    {
      label: 'IN',
      value: 'IN ',
      type: 'operator' as const,
      description: 'In list',
      dataType: undefined
    },
    {
      label: 'IS NULL',
      value: 'IS NULL ',
      type: 'operator' as const,
      description: 'Null check',
      dataType: undefined
    },
    {
      label: 'IS NOT NULL',
      value: 'IS NOT NULL ',
      type: 'operator' as const,
      description: 'Not null check',
      dataType: undefined
    }
  ];

  const all = [...cols, ...ops];

  if (!token) return cols;

  return all.filter(
    (item) => item.label.toLowerCase().includes(token) && item.label.toLowerCase() !== token
  );
});

const selectSuggestion = (item: { label: string; value: string }) => {
  const inputEl = filterInputRef.value;
  if (!inputEl) return;

  const text = gridStore.filterText || '';
  const cursorOffset = inputEl.selectionStart || 0;

  const textBeforeCursor = text.slice(0, cursorOffset);
  const lastTokenStart = textBeforeCursor.search(/[a-zA-Z0-9_]*$/);

  if (lastTokenStart !== -1) {
    const before = text.slice(0, lastTokenStart);
    const after = text.slice(cursorOffset);
    gridStore.filterText = before + item.value + after;

    const newCursorPos = lastTokenStart + item.value.length;
    nextTick(() => {
      inputEl.focus();
      inputEl.setSelectionRange(newCursorPos, newCursorPos);
      updateCursorOffset();
    });
  }

  showSuggestions.value = false;
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (!showSuggestions.value || filteredSuggestions.value.length === 0) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      showSuggestions.value = true;
      activeSuggestionIndex.value = 0;
      e.preventDefault();
    } else if (e.key === 'Enter') {
      handleFilter();
    }
    return;
  }

  if (e.key === 'ArrowDown') {
    activeSuggestionIndex.value =
      (activeSuggestionIndex.value + 1) % filteredSuggestions.value.length;
    e.preventDefault();
  } else if (e.key === 'ArrowUp') {
    activeSuggestionIndex.value =
      (activeSuggestionIndex.value - 1 + filteredSuggestions.value.length) %
      filteredSuggestions.value.length;
    e.preventDefault();
  } else if (e.key === 'Enter' || e.key === 'Tab') {
    const index = Math.min(activeSuggestionIndex.value, filteredSuggestions.value.length - 1);
    const item = filteredSuggestions.value[index];
    if (item) {
      selectSuggestion(item);
      e.preventDefault();
      e.stopPropagation();
    }
  } else if (e.key === 'Escape') {
    showSuggestions.value = false;
    e.preventDefault();
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
        'com.github.vantoan1511.tableview.db-bridge',
        'dbBridge.exportCSV',
        {
          reqId,
          connectionId: gridStore.activeConnectionId || connectionsStore.activeConnectionId,
          tableName: gridStore.activeTableName,
          exportPath: path,
          targetDatabase: gridStore.activeDbName
        }
      );
    }
  } catch (err: any) {
    // Neutralino throws { code: 'NE_OS_DLGCDL' } when user cancels the dialog — not an error
    if (err && err.code !== 'NE_OS_DLGCDL') {
      console.error('Export failed', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Export Failed',
        message: err.message || 'An error occurred while exporting data.'
      });
    }
  }
};
</script>

<template>
  <div
    class="border-border bg-surface @container flex min-h-12 items-center justify-between gap-2 border-b px-4 py-2"
  >
    <!-- Left: Table name + row actions -->
    <div class="flex min-w-0 items-center gap-2">
      <div class="mr-1 flex shrink-0 items-center gap-2">
        <h2
          v-tooltip.bottom="gridStore.activeTableName"
          class="text-text-primary max-w-20 cursor-help truncate text-[14px] font-semibold @[400px]:max-w-37.5 @[850px]:max-w-none"
        >
          {{ gridStore.activeTableName }}
        </h2>
      </div>
    </div>

    <!-- Filter Query Input -->
    <div v-if="gridStore.activeTableName" class="relative min-w-0 flex-1">
      <div
        class="bg-surface border-border focus-within:border-primary/50 flex items-center gap-2 rounded-lg border px-2.5 py-1.5 transition-colors"
      >
        <Filter :size="12" class="text-text-tertiary shrink-0" />
        <input
          ref="filterInputRef"
          type="text"
          placeholder="Filter condition (e.g. id > 10)..."
          class="text-text-primary placeholder-text-tertiary flex-1 border-none bg-transparent text-[12px] outline-none"
          v-model="gridStore.filterText"
          @input="handleInput"
          @keydown="handleKeyDown"
          @click="handleInput"
          @blur="handleBlur"
        />
        <button
          v-if="gridStore.filterText"
          class="text-text-tertiary hover:text-text-primary shrink-0 cursor-pointer"
          @click="clearFilter"
        >
          <X :size="12" />
        </button>
      </div>

      <!-- Autocomplete Dropdown -->
      <div
        v-if="showSuggestions && filteredSuggestions.length > 0"
        class="border-border bg-surface absolute top-full left-0 z-50 mt-1 max-h-60 w-full overflow-y-auto rounded-lg border shadow-xl"
      >
        <div class="p-1">
          <button
            v-for="(item, index) in filteredSuggestions"
            :key="item.value"
            class="hover:bg-hover flex w-full cursor-pointer items-center justify-between rounded-md px-3 py-1.5 text-left text-[12px]"
            :class="{ 'bg-hover': index === activeSuggestionIndex }"
            @click="selectSuggestion(item)"
          >
            <div class="flex items-center gap-2">
              <span class="text-text-tertiary text-[10px]">
                <span
                  v-if="item.type === 'column'"
                  class="bg-primary/10 text-primary rounded px-1.5 py-0.5 text-[9px] font-semibold uppercase"
                >
                  Col
                </span>
                <span
                  v-else
                  class="bg-accent/10 text-accent rounded px-1.5 py-0.5 text-[9px] font-semibold uppercase"
                >
                  Op
                </span>
              </span>
              <span class="text-text-primary font-medium">{{ item.label }}</span>
              <span v-if="item.dataType" class="text-text-tertiary text-[10px]">
                ({{ item.dataType }})
              </span>
            </div>
            <span class="text-text-tertiary text-[10px]">{{ item.description }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Right: Actions -->
    <div class="flex min-w-0 items-center gap-2">
      <!-- Level 1 & 2 Actions Group -->
      <div class="hidden items-center gap-1.5 @[500px]:flex">
        <!-- Columns -->
        <DropdownMenu
          v-tooltip.bottom="'Configure Columns'"
          :model-value="''"
          :options="[]"
          aria-label="Columns configuration"
          align="right"
          :show-chevron="true"
          button-class="border-border text-text-secondary hover:bg-hover group rounded-lg px-2.5 py-1.5"
          menu-class="min-w-52"
        >
          <template #trigger>
            <Columns3 :size="13" />
            <span class="hidden @[850px]:inline">Columns</span>
          </template>
          <div
            class="text-text-tertiary px-3 py-1.5 text-[11px] font-semibold tracking-wider uppercase"
          >
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
        </DropdownMenu>

        <!-- Export -->
        <button
          v-tooltip.bottom="'Export Data'"
          class="border-border text-text-secondary hover:bg-hover group relative flex cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] transition-colors"
          @click="handleExport"
        >
          <Download :size="13" />
          <span class="hidden @[850px]:inline">Export</span>
        </button>

        <!-- Row Count -->
        <DropdownMenu
          :model-value="gridStore.rowsPerPage"
          :options="rowsOptions"
          aria-label="Rows per page"
          button-class="border-border text-text-secondary hover:bg-hover rounded-lg px-2.5 py-1.5"
          @update:model-value="setRowsPerPage"
        >
          <template #trigger>
            <span class="whitespace-nowrap">
              {{ gridStore.rowsPerPage }} <span class="hidden @[850px]:inline">rows</span>
            </span>
          </template>
        </DropdownMenu>

        <!-- Divider -->
        <div class="bg-border mx-0.5 hidden h-5 w-px @[600px]:block" />

        <!-- Alter Table -->
        <button
          v-tooltip.bottom="'Alter Table Structure'"
          class="border-border text-text-secondary hover:bg-hover group relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border transition-colors"
          @click="gridStore.showAlterTableDialog = true"
        >
          <Wrench :size="14" />
        </button>

        <!-- Refresh -->
        <button
          v-tooltip.bottom="'Refresh Table'"
          class="border-border text-text-secondary hover:bg-hover group relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border transition-colors"
          @click="handleRefresh"
        >
          <RefreshCw :size="14" />
        </button>
      </div>

      <!-- Level 3: Speed Dial / More Button -->
      <div class="flex items-center @[500px]:hidden">
        <DropdownMenu
          v-tooltip.bottom="'More Actions'"
          :model-value="''"
          :options="[]"
          aria-label="More actions"
          align="right"
          :show-chevron="false"
          button-class="border-border text-text-secondary hover:bg-hover group h-8 w-8 justify-center rounded-lg p-0"
          menu-class="min-w-64"
        >
          <template #trigger>
            <MoreHorizontal :size="16" />
          </template>
          <template #default="{ close }">
            <button
              class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
              @click="
                handleRefresh();
                close();
              "
            >
              <RefreshCw :size="14" /> <span>Refresh</span>
            </button>
            <button
              class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
              @click="
                gridStore.showAlterTableDialog = true;
                close();
              "
            >
              <Wrench :size="14" /> <span>Alter Table</span>
            </button>
            <button
              class="hover:bg-hover flex w-full items-center gap-2 px-3 py-2 text-[12px]"
              @click="
                handleExport();
                close();
              "
            >
              <Download :size="14" /> <span>Export CSV</span>
            </button>

            <div class="bg-border my-1 h-px"></div>
            <div
              class="text-text-tertiary flex items-center gap-2 px-3 py-1.5 text-[11px] font-semibold tracking-wider uppercase"
            >
              <LayoutGrid :size="13" /> Rows per page
            </div>
            <button
              v-for="option in rowsOptions"
              :key="option.value"
              class="hover:bg-hover flex w-full cursor-pointer items-center justify-between px-3 py-1.5 text-[12px]"
              @click="setRowsPerPage(option.value)"
            >
              <span>{{ option.label }}</span>
              <span v-if="gridStore.rowsPerPage === option.value" class="text-primary">✓</span>
            </button>

            <div class="bg-border my-1 h-px"></div>
            <div
              class="text-text-tertiary flex items-center gap-2 px-3 py-1.5 text-[11px] font-semibold tracking-wider uppercase"
            >
              <Columns3 :size="13" /> Columns
            </div>
            <div class="max-h-52 overflow-y-auto">
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
          </template>
        </DropdownMenu>
      </div>
    </div>
  </div>

  <!-- Alter Table Dialog -->
  <AlterTableDialog
    v-if="gridStore.showAlterTableDialog"
    :tableName="gridStore.activeTableName"
    @close="gridStore.showAlterTableDialog = false"
    @apply="confirmAlterTable"
  />
</template>
