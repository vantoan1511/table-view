<script setup lang="ts">
import GridCell from '../grid/GridCell.vue';
import GridHeaderCell from '../grid/GridHeaderCell.vue';
import Skeleton from '@/components/ui/Skeleton.vue';

import { useClickOutside } from '@/composables/useClickOutside';
import { useGridResizing } from '@/composables/useGridResizing';

import { useGridStore } from '@/stores/grid';

import type { GridColumn } from '@/types';

import { computed, ref, onMounted, onUnmounted, watch } from 'vue';

const gridStore = useGridStore();

const selectedCell = ref<{ rowIndex: number; columnName: string } | null>(null);
const selectedRowIndices = ref<Set<number>>(new Set());

// Column Resizing
const { onResizeStart, MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH, DEFAULT_COLUMN_WIDTH } = useGridResizing(
  {
    onResize: (colName, width) => gridStore.setColumnWidth(colName, width)
  }
);

// Cell selection click handler
const onCellClick = (rowIdx: number, col: GridColumn) => {
  selectedCell.value = { rowIndex: rowIdx, columnName: col.name };
};

// Row selection with multiselect capability
const toggleRowSelection = (rowIdx: number, event: MouseEvent) => {
  const newSet = new Set(selectedRowIndices.value);

  if (event.shiftKey && selectedRowIndices.value.size > 0) {
    const lastIdx = Math.max(...selectedRowIndices.value);
    const start = Math.min(lastIdx, rowIdx);
    const end = Math.max(lastIdx, rowIdx);
    for (let i = start; i <= end; i++) {
      newSet.add(i);
    }
  } else if (event.ctrlKey || event.metaKey) {
    if (newSet.has(rowIdx)) {
      newSet.delete(rowIdx);
    } else {
      newSet.add(rowIdx);
    }
  } else {
    if (newSet.has(rowIdx)) {
      newSet.delete(rowIdx);
    } else {
      newSet.clear();
      newSet.add(rowIdx);
    }
  }

  selectedRowIndices.value = newSet;
};

// Toggle all rows selection
const toggleSelectAllRows = () => {
  if (selectedRowIndices.value.size === gridStore.sqlRows.length) {
    selectedRowIndices.value = new Set();
  } else {
    const newSet = new Set<number>();
    for (let i = 0; i < gridStore.sqlRows.length; i++) {
      newSet.add(i);
    }
    selectedRowIndices.value = newSet;
  }
};

// Handle clicking outside to clear selection
const gridRef = ref<HTMLElement | null>(null);
useClickOutside(gridRef, () => {
  selectedCell.value = null;
});

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
    if (selectedCell.value) {
      const { rowIndex, columnName } = selectedCell.value;
      const row = gridStore.sqlRows[rowIndex];
      if (row) {
        const value = row[columnName];
        if (value !== undefined) {
          event.preventDefault();
          navigator.clipboard.writeText(value === null ? 'NULL' : String(value));
        }
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
    gridStore.sqlRows.length,
    Math.ceil((scrollTop.value + viewportHeight.value) / ROW_HEIGHT) + BUFFER_ROWS
  );
});

const visibleRows = computed(() => {
  return gridStore.sqlRows.slice(startIndex.value, endIndex.value);
});

const topSpacerHeight = computed(() => {
  return startIndex.value * ROW_HEIGHT;
});

const bottomSpacerHeight = computed(() => {
  return (gridStore.sqlRows.length - endIndex.value) * ROW_HEIGHT;
});

// Reset scroll on table data changes
watch(
  () => gridStore.sqlRows,
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
    <div
      ref="scrollContainerRef"
      class="relative min-h-0 flex-1 overflow-auto"
      @scroll="handleScroll"
      @click.self="selectedCell = null"
    >
      <table class="table-fixed border-collapse text-[12px] font-(--font-mono)">
        <colgroup>
          <col class="w-12" style="width: 48px" />
          <col
            v-for="col in gridStore.sqlColumns"
            :key="'sql-col-group-' + col.name"
            :style="{
              width: `${Math.min(MAX_COLUMN_WIDTH, Math.max(MIN_COLUMN_WIDTH, gridStore.columnWidths[col.name] ?? DEFAULT_COLUMN_WIDTH))}px`
            }"
          />
        </colgroup>
        <thead class="sticky top-0 z-20">
          <tr class="bg-grid-header border-grid-border border-b">
            <th
              class="text-text-tertiary border-grid-border bg-grid-header sticky left-0 z-30 w-12 border-r px-3 py-2 text-right text-[11px] font-medium select-none"
              @click="toggleSelectAllRows"
            >
              #
            </th>
            <GridHeaderCell
              v-for="col in gridStore.sqlColumns"
              :key="col.name"
              :column="col"
              :column-width="gridStore.columnWidths[col.name]"
              :min-width="MIN_COLUMN_WIDTH"
              :max-width="MAX_COLUMN_WIDTH"
              :default-width="DEFAULT_COLUMN_WIDTH"
              @resize="onResizeStart(col.name, $event)"
            />
          </tr>
        </thead>

        <tbody>
          <!-- Loading state with skeletons matching DataGrid -->
          <template v-if="gridStore.isLoading">
            <tr v-for="i in 10" :key="'skel-' + i" class="border-grid-border border-b">
              <td class="border-grid-border sticky left-0 z-10 border-r bg-inherit px-3 py-1.5">
                <Skeleton height="1.25rem" />
              </td>
              <template v-if="gridStore.sqlColumns.length > 0">
                <td
                  v-for="col in gridStore.sqlColumns"
                  :key="'skel-' + col.name"
                  class="border-grid-border border-r px-3 py-1.5"
                  :style="{
                    width: `${Math.min(MAX_COLUMN_WIDTH, Math.max(MIN_COLUMN_WIDTH, gridStore.columnWidths[col.name] ?? DEFAULT_COLUMN_WIDTH))}px`,
                    minWidth: `${MIN_COLUMN_WIDTH}px`,
                    maxWidth: `${MAX_COLUMN_WIDTH}px`
                  }"
                >
                  <Skeleton height="1.25rem" />
                </td>
              </template>
              <template v-else>
                <td
                  v-for="j in 5"
                  :key="'skel-col-' + j"
                  class="border-grid-border border-r px-3 py-1.5"
                  style="width: 160px; min-width: 80px; max-width: 320px"
                >
                  <Skeleton height="1.25rem" />
                </td>
              </template>
            </tr>
          </template>

          <!-- Query results data rendering -->
          <template v-else>
            <!-- Virtual scroll top spacer -->
            <tr v-if="topSpacerHeight > 0" :style="{ height: `${topSpacerHeight}px` }">
              <td
                :colspan="gridStore.sqlColumns.length + 1"
                style="padding: 0; border: none; background: transparent"
              ></td>
            </tr>

            <!-- Rendered visible subset of rows -->
            <template v-for="(row, i) in visibleRows" :key="startIndex + i">
              <tr
                class="border-grid-border hover:bg-grid-row-hover border-b transition-colors"
                :class="{
                  'bg-grid-row-alt':
                    (startIndex + i) % 2 === 1 && !selectedRowIndices.has(startIndex + i),
                  'bg-primary/10!': selectedRowIndices.has(startIndex + i)
                }"
              >
                <td
                  class="text-text-tertiary border-grid-border sticky left-0 z-10 cursor-pointer border-r bg-inherit px-3 py-1.5 text-right text-[11px] tabular-nums backdrop-blur-sm select-none"
                  :class="{
                    'bg-primary/20! text-primary! font-semibold': selectedRowIndices.has(
                      startIndex + i
                    )
                  }"
                  @click="toggleRowSelection(startIndex + i, $event)"
                >
                  {{ startIndex + i + 1 }}
                </td>

                <GridCell
                  v-for="col in gridStore.sqlColumns"
                  :key="col.name"
                  :row-index="startIndex + i"
                  :column="col"
                  :value="row[col.name]"
                  :is-selected="
                    selectedCell?.rowIndex === startIndex + i &&
                    selectedCell?.columnName === col.name
                  "
                  :column-width="gridStore.columnWidths[col.name]"
                  :min-width="MIN_COLUMN_WIDTH"
                  :max-width="MAX_COLUMN_WIDTH"
                  :default-width="DEFAULT_COLUMN_WIDTH"
                  @click="onCellClick(startIndex + i, col)"
                />
              </tr>
            </template>

            <!-- Virtual scroll bottom spacer -->
            <tr v-if="bottomSpacerHeight > 0" :style="{ height: `${bottomSpacerHeight}px` }">
              <td
                :colspan="gridStore.sqlColumns.length + 1"
                style="padding: 0; border: none; background: transparent"
              ></td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>
  </div>
</template>
