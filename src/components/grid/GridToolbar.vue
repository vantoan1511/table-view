<script setup lang="ts">
import AlterTableDialog from '@/components/ui/AlterTableDialog.vue';
import Button from 'primevue/button';
import Checkbox from 'primevue/checkbox';
import IconField from 'primevue/iconfield';
import InputIcon from 'primevue/inputicon';
import InputText from 'primevue/inputtext';
import Popover from 'primevue/popover';
import Select from 'primevue/select';

import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useToastStore } from '@/stores/toast';

import { BridgeService } from '@/services/bridge';
import { os } from '@/services/nativeService';
import {
  Columns3,
  Download,
  Filter,
  LayoutGrid,
  MoreHorizontal,
  RefreshCw,
  Wrench,
  X
} from 'lucide-vue-next';
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

const columnsPopover = ref();
const morePopover = ref();

const setRowsPerPage = (count: string | number) => {
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
    const path = await os.showSaveDialog('Export as CSV', {
      defaultPath: defaultName,
      filters: [{ name: 'CSV files', extensions: ['csv'] }]
    });

    if (path) {
      const res: any = await BridgeService.request(
        'dbBridge.exportCSV',
        'dbBridge.exportCSVResult',
        {
          connectionId: gridStore.activeConnectionId || connectionsStore.activeConnectionId,
          tableName: gridStore.activeTableName,
          exportPath: path,
          targetDatabase: gridStore.activeDbName
        }
      );

      if (res?.success) {
        toastStore.addToast({
          title: 'Export Success',
          message: 'Data exported successfully to ' + path,
          severity: 'success'
        });
      } else {
        toastStore.addToast({
          title: 'Export Error',
          message: res?.error,
          severity: 'error'
        });
      }
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
      <div class="relative flex items-center">
        <IconField class="w-full">
          <InputIcon>
            <Filter :size="12" class="text-text-tertiary" />
          </InputIcon>
          <InputText
            ref="filterInputRef"
            type="text"
            placeholder="Filter condition (e.g. id > 10)..."
            v-model="gridStore.filterText"
            size="small"
            class="w-full pr-7 text-xs"
            @input="handleInput"
            @keydown="handleKeyDown"
            @click="handleInput"
            @blur="handleBlur"
          />
        </IconField>
        <Button
          v-if="gridStore.filterText"
          variant="text"
          severity="secondary"
          size="small"
          class="absolute right-1 h-6! w-6! p-0!"
          @click="clearFilter"
        >
          <template #icon>
            <X :size="12" />
          </template>
        </Button>
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
        <!-- Columns Button & Popover -->
        <Button
          v-tooltip.bottom="'Configure Columns'"
          variant="outlined"
          severity="secondary"
          size="small"
          @click="(event) => columnsPopover.toggle(event)"
        >
          <Columns3 class="h-3.5 w-3.5" />
          <span class="hidden text-xs @[850px]:inline">Columns</span>
        </Button>
        <Popover ref="columnsPopover">
          <div class="min-w-52 p-1">
            <div
              class="text-text-tertiary px-2 py-1 text-[11px] font-semibold tracking-wider uppercase"
            >
              Columns
            </div>
            <div class="max-h-64 space-y-1 overflow-y-auto">
              <div
                v-for="col in gridStore.columns"
                :key="col.name"
                class="hover:bg-hover flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-xs"
                @click="gridStore.toggleColumnVisibility(col.name)"
              >
                <Checkbox
                  :model-value="gridStore.columnVisibility[col.name] !== false"
                  binary
                  @click.stop="gridStore.toggleColumnVisibility(col.name)"
                />
                <span class="flex-1 truncate">{{ col.name }}</span>
                <span class="text-text-tertiary text-[10px]">{{ col.dataType }}</span>
              </div>
            </div>
          </div>
        </Popover>

        <!-- Export -->
        <Button
          v-tooltip.bottom="'Export Data'"
          variant="outlined"
          severity="secondary"
          size="small"
          @click="handleExport"
        >
          <Download class="h-3.5 w-3.5" />
          <span class="hidden text-xs @[850px]:inline">Export</span>
        </Button>

        <!-- Row Count -->
        <Select
          :model-value="gridStore.rowsPerPage"
          :options="rowsOptions"
          optionLabel="label"
          optionValue="value"
          size="small"
          class="w-24 text-xs"
          @update:model-value="setRowsPerPage"
        />

        <!-- Alter Table -->
        <Button
          v-tooltip.bottom="'Alter Table Structure'"
          variant="outlined"
          severity="secondary"
          size="small"
          class="h-8! w-8! p-0!"
          @click="gridStore.showAlterTableDialog = true"
        >
          <template #icon>
            <Wrench :size="14" />
          </template>
        </Button>

        <!-- Refresh -->
        <Button
          v-tooltip.bottom="'Refresh Table'"
          variant="outlined"
          severity="secondary"
          size="small"
          class="h-8! w-8! p-0!"
          @click="handleRefresh"
        >
          <template #icon>
            <RefreshCw :size="14" />
          </template>
        </Button>
      </div>

      <!-- Level 3: Speed Dial / More Button -->
      <div class="flex items-center @[500px]:hidden">
        <Button
          v-tooltip.bottom="'More Actions'"
          variant="outlined"
          severity="secondary"
          size="small"
          class="h-8! w-8! p-0!"
          @click="(event) => morePopover.toggle(event)"
        >
          <template #icon>
            <MoreHorizontal :size="16" />
          </template>
        </Button>
        <Popover ref="morePopover">
          <div class="flex min-w-60 flex-col gap-1 p-1">
            <Button
              variant="text"
              severity="secondary"
              size="small"
              class="w-full justify-start!"
              @click="
                handleRefresh();
                morePopover.hide();
              "
            >
              <RefreshCw class="h-3.5 w-3.5" />
              <span>Refresh</span>
            </Button>
            <Button
              variant="text"
              severity="secondary"
              size="small"
              class="w-full justify-start!"
              @click="
                gridStore.showAlterTableDialog = true;
                morePopover.hide();
              "
            >
              <Wrench class="h-3.5 w-3.5" />
              <span>Alter Table</span>
            </Button>
            <Button
              variant="text"
              severity="secondary"
              size="small"
              class="w-full justify-start!"
              @click="
                handleExport();
                morePopover.hide();
              "
            >
              <Download class="h-3.5 w-3.5" />
              <span>Export CSV</span>
            </Button>

            <div
              class="text-text-tertiary flex items-center gap-2 px-2 pt-2 text-[11px] font-semibold tracking-wider uppercase"
            >
              <LayoutGrid :size="13" /> Rows per page
            </div>
            <div class="flex items-center px-2 py-1">
              <Select
                :model-value="gridStore.rowsPerPage"
                :options="rowsOptions"
                optionLabel="label"
                optionValue="value"
                size="small"
                class="w-full text-xs"
                @update:model-value="
                  (val) => {
                    setRowsPerPage(val);
                    morePopover.hide();
                  }
                "
              />
            </div>

            <div
              class="text-text-tertiary flex items-center gap-2 px-2 pt-2 text-[11px] font-semibold tracking-wider uppercase"
            >
              <Columns3 :size="13" /> Columns
            </div>
            <div class="max-h-52 space-y-1 overflow-y-auto">
              <div
                v-for="col in gridStore.columns"
                :key="col.name"
                class="hover:bg-hover flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-xs"
                @click="gridStore.toggleColumnVisibility(col.name)"
              >
                <Checkbox
                  :model-value="gridStore.columnVisibility[col.name] !== false"
                  binary
                  @click.stop="gridStore.toggleColumnVisibility(col.name)"
                />
                <span class="flex-1 truncate">{{ col.name }}</span>
                <span class="text-text-tertiary text-[10px]">{{ col.dataType }}</span>
              </div>
            </div>
          </div>
        </Popover>
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
