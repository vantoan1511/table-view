<script setup lang="ts">
import {
  Database,
  Download,
  Info,
  Key,
  Link,
  Maximize,
  RefreshCw,
  Search,
  Table,
  ZoomIn,
  ZoomOut
} from 'lucide-vue-next';
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';

import { useDiagramStore } from '@/stores/diagram';
import { useTabsStore } from '@/stores/tabs';
import type { Tab, TableDetails } from '@/types';

const props = defineProps<{
  tab: Tab;
}>();

const diagramStore = useDiagramStore();
const tabsStore = useTabsStore();

// Viewport layout states
const zoom = ref(1.0);
const panX = ref(50);
const panY = ref(50);
const searchQuery = ref('');

// Table coordinate positions
const tablePositions = ref<Record<string, { x: number; y: number }>>({});
const hoveredTable = ref<string | null>(null);
const hoveredColumn = ref<{ tableName: string; columnName: string } | null>(null);

// Interactive dragging states
const isDraggingCanvas = ref(false);
const isDraggingCard = ref(false);
const draggedTable = ref<string | null>(null);
const dragStartMouseX = ref(0);
const dragStartMouseY = ref(0);
const dragStartCardX = ref(0);
const dragStartCardY = ref(0);
const canvasStartPanX = ref(0);
const canvasStartPanY = ref(0);

// Viewport dimensions for culling
const viewportWidth = ref(1200);
const viewportHeight = ref(800);
let resizeObserver: ResizeObserver | null = null;

// rAF batching state
let rafId: number | null = null;
let pendingMouseEvent: MouseEvent | null = null;

// References
const viewportRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLDivElement | null>(null);

// Constants for layout
const CARD_WIDTH = 220;
const HEADER_HEIGHT = 40;
const ROW_HEIGHT = 28;
const MAX_COLUMNS_DISPLAY_HEIGHT = 220;

// Get current details from store
const cacheKey = computed(() =>
  diagramStore.getCacheKey(props.tab.connectionId || '', props.tab.schema || '', props.tab.dbName)
);

const schemaDetails = computed(() => diagramStore.diagrams[cacheKey.value] || null);
const isLoading = computed(() => diagramStore.loading[cacheKey.value] || false);
const errorMsg = computed(() => diagramStore.errors[cacheKey.value] || null);

// Filtered tables based on search query
const filteredTables = computed(() => {
  if (!schemaDetails.value) return [];
  const query = searchQuery.value.toLowerCase().trim();
  if (!query) return schemaDetails.value.tables;
  return schemaDetails.value.tables.filter((t) => t.name.toLowerCase().includes(query));
});

// Pre-indexed metadata for instant O(1) relational lookups
interface IndexedRelation {
  id: string;
  sourceTable: string;
  sourceColumn: string;
  targetTable: string;
  targetColumn: string;
  sourceColIdx: number;
  targetColIdx: number;
}

const schemaIndex = computed(() => {
  if (!schemaDetails.value) {
    return {
      tableByName: new Map<string, TableDetails>(),
      colIndexByTable: new Map<string, Map<string, number>>(),
      indexedRelations: [] as IndexedRelation[],
      relationsByTable: new Map<string, number[]>()
    };
  }

  const tableByName = new Map<string, TableDetails>();
  const colIndexByTable = new Map<string, Map<string, number>>();
  const relationsByTable = new Map<string, number[]>();

  schemaDetails.value.tables.forEach((t) => {
    tableByName.set(t.name, t);
    const colMap = new Map<string, number>();
    t.columns.forEach((col, idx) => {
      colMap.set(col.name, idx);
    });
    colIndexByTable.set(t.name, colMap);
  });

  const indexedRelations: IndexedRelation[] = [];

  schemaDetails.value.relations.forEach((rel, index) => {
    const sourceCols = colIndexByTable.get(rel.sourceTable);
    const targetCols = colIndexByTable.get(rel.targetTable);

    const sourceColIdx = sourceCols?.get(rel.sourceColumn) ?? -1;
    const targetColIdx = targetCols?.get(rel.targetColumn) ?? -1;

    if (tableByName.has(rel.sourceTable) && tableByName.has(rel.targetTable)) {
      const relIdx = indexedRelations.length;
      indexedRelations.push({
        id: `${rel.constraintName || 'rel'}-${index}`,
        sourceTable: rel.sourceTable,
        sourceColumn: rel.sourceColumn,
        targetTable: rel.targetTable,
        targetColumn: rel.targetColumn,
        sourceColIdx,
        targetColIdx
      });

      const srcList = relationsByTable.get(rel.sourceTable) || [];
      srcList.push(relIdx);
      relationsByTable.set(rel.sourceTable, srcList);

      const tgtList = relationsByTable.get(rel.targetTable) || [];
      tgtList.push(relIdx);
      relationsByTable.set(rel.targetTable, tgtList);
    }
  });

  return {
    tableByName,
    colIndexByTable,
    indexedRelations,
    relationsByTable
  };
});

// Card height for auto-layout
const getTableHeight = (table: TableDetails): number => {
  const colHeight = Math.min(table.columns.length * ROW_HEIGHT, MAX_COLUMNS_DISPLAY_HEIGHT);
  return HEADER_HEIGHT + colHeight + 8;
};

// Load coordinate positions from LocalStorage
const getStorageKey = () =>
  `table-positions-${props.tab.connectionId}-${props.tab.dbName || 'default'}-${props.tab.schema}`;

const loadPositions = () => {
  try {
    const saved = localStorage.getItem(getStorageKey());
    if (saved) {
      tablePositions.value = JSON.parse(saved);
      return true;
    }
  } catch (e) {
    console.error('Failed to load table positions from local storage:', e);
  }
  return false;
};

// Save coordinate positions to LocalStorage
const savePositions = () => {
  try {
    localStorage.setItem(getStorageKey(), JSON.stringify(tablePositions.value));
  } catch (e) {
    console.error('Failed to save table positions:', e);
  }
};

// Height-aware column packing auto-layout to prevent overlapping cards
const performAutoLayout = (force = false) => {
  if (!schemaDetails.value) return;

  const tables = schemaDetails.value.tables;
  if (tables.length === 0) return;

  const cols = Math.max(3, Math.ceil(Math.sqrt(tables.length * 1.5)));
  const spacingX = 290;
  const gapY = 32;

  const colY = new Array(cols).fill(80);
  const newPositions: Record<string, { x: number; y: number }> = {};

  tables.forEach((table) => {
    if (!force && tablePositions.value[table.name]) {
      newPositions[table.name] = { ...tablePositions.value[table.name]! };
      return;
    }

    // Place table in the column with minimum cumulative height to balance distribution
    let minCol = 0;
    for (let c = 1; c < cols; c++) {
      if (colY[c] < colY[minCol]) {
        minCol = c;
      }
    }

    const cardHeight = getTableHeight(table);

    newPositions[table.name] = {
      x: 80 + minCol * spacingX,
      y: colY[minCol]
    };

    colY[minCol] += cardHeight + gapY;
  });

  tablePositions.value = newPositions;
  savePositions();
};

const loadData = async (force = false) => {
  if (!props.tab.connectionId || !props.tab.schema) return;
  await diagramStore.fetchSchemaDetails(
    props.tab.connectionId,
    props.tab.schema,
    props.tab.dbName,
    force
  );

  const hasSavedPositions = loadPositions();
  performAutoLayout(!hasSavedPositions);
};

const handleTableDoubleClick = (tableName: string) => {
  tabsStore.openTableTab(tableName, props.tab.schema, props.tab.connectionId, props.tab.dbName);
};

// ResizeObserver for viewport dimensions
const setupResizeObserver = () => {
  watch(
    viewportRef,
    (el) => {
      if (el) {
        viewportWidth.value = el.clientWidth || window.innerWidth;
        viewportHeight.value = el.clientHeight || window.innerHeight;
        resizeObserver?.disconnect();
        resizeObserver = new ResizeObserver((entries) => {
          for (const entry of entries) {
            if (entry.contentRect) {
              viewportWidth.value = entry.contentRect.width;
              viewportHeight.value = entry.contentRect.height;
            }
          }
        });
        resizeObserver.observe(el);
      }
    },
    { immediate: true }
  );
};

// Fetch on mount
onMounted(() => {
  loadData();
  setupResizeObserver();
  window.addEventListener('mouseup', handleGlobalMouseUp);
  window.addEventListener('mousemove', handleGlobalMouseMove);
});

onUnmounted(() => {
  if (rafId !== null) {
    cancelAnimationFrame(rafId);
  }
  resizeObserver?.disconnect();
  window.removeEventListener('mouseup', handleGlobalMouseUp);
  window.removeEventListener('mousemove', handleGlobalMouseMove);
});

// Viewport Zoom Actions
const zoomIn = () => {
  zoom.value = Math.min(3.0, zoom.value + 0.1);
};

const zoomOut = () => {
  zoom.value = Math.max(0.1, zoom.value - 0.1);
};

const zoomToFit = () => {
  if (!schemaDetails.value || !viewportRef.value) return;

  const names = Object.keys(tablePositions.value);
  if (names.length === 0) return;

  let minX = Infinity,
    maxX = -Infinity;
  let minY = Infinity,
    maxY = -Infinity;

  names.forEach((name) => {
    const pos = tablePositions.value[name];
    if (pos) {
      minX = Math.min(minX, pos.x);
      maxX = Math.max(maxX, pos.x + CARD_WIDTH);
      minY = Math.min(minY, pos.y);
      const table = schemaIndex.value.tableByName.get(name);
      const height = table ? getTableHeight(table) : HEADER_HEIGHT;
      maxY = Math.max(maxY, pos.y + height);
    }
  });

  const viewportW = viewportRef.value.clientWidth;
  const viewportH = viewportRef.value.clientHeight;

  const contentW = maxX - minX + 100;
  const contentH = maxY - minY + 100;

  const scaleX = viewportW / contentW;
  const scaleY = viewportH / contentH;

  zoom.value = Math.min(Math.max(Math.min(scaleX, scaleY), 0.15), 1.5);
  panX.value = Math.round((viewportW - (maxX + minX) * zoom.value) / 2);
  panY.value = Math.round((viewportH - (maxY + minY) * zoom.value) / 2);
};

const resetZoom = () => {
  zoom.value = 1.0;
  panX.value = 50;
  panY.value = 50;
};

// Canvas Mouse Events (Panning)
const handleCanvasMouseDown = (e: MouseEvent) => {
  if ((e.target as HTMLElement).closest('.table-card')) return;

  isDraggingCanvas.value = true;
  canvasStartPanX.value = panX.value;
  canvasStartPanY.value = panY.value;
  dragStartMouseX.value = e.clientX;
  dragStartMouseY.value = e.clientY;

  e.preventDefault();
};

// Canvas Mouse Wheel (Zooming)
const handleCanvasWheel = (e: WheelEvent) => {
  e.preventDefault();
  const zoomFactor = 0.05;
  const direction = e.deltaY > 0 ? -1 : 1;
  const oldZoom = zoom.value;
  const newZoom = Math.min(3.0, Math.max(0.1, zoom.value + direction * zoomFactor));

  if (!viewportRef.value) return;

  const rect = viewportRef.value.getBoundingClientRect();
  const mouseX = e.clientX - rect.left;
  const mouseY = e.clientY - rect.top;

  const contentX = (mouseX - panX.value) / oldZoom;
  const contentY = (mouseY - panY.value) / oldZoom;

  zoom.value = newZoom;
  panX.value = Math.round(mouseX - contentX * newZoom);
  panY.value = Math.round(mouseY - contentY * newZoom);
};

// Table Card Mouse Events (Dragging Table Cards)
const handleCardMouseDown = (e: MouseEvent, tableName: string) => {
  e.stopPropagation();
  const pos = tablePositions.value[tableName];
  if (!pos) return;

  isDraggingCard.value = true;
  draggedTable.value = tableName;
  dragStartMouseX.value = e.clientX;
  dragStartMouseY.value = e.clientY;
  dragStartCardX.value = pos.x;
  dragStartCardY.value = pos.y;

  e.preventDefault();
};

// Global Mouse Movement handling throttled with requestAnimationFrame
const handleGlobalMouseMove = (e: MouseEvent) => {
  if (!isDraggingCanvas.value && !isDraggingCard.value) return;
  pendingMouseEvent = e;
  if (rafId === null) {
    rafId = requestAnimationFrame(processMouseMove);
  }
};

const processMouseMove = () => {
  rafId = null;
  const e = pendingMouseEvent;
  if (!e) return;

  if (isDraggingCanvas.value) {
    const deltaX = e.clientX - dragStartMouseX.value;
    const deltaY = e.clientY - dragStartMouseY.value;
    panX.value = canvasStartPanX.value + deltaX;
    panY.value = canvasStartPanY.value + deltaY;
  } else if (isDraggingCard.value && draggedTable.value) {
    const deltaX = e.clientX - dragStartMouseX.value;
    const deltaY = e.clientY - dragStartMouseY.value;

    const scaledDeltaX = deltaX / zoom.value;
    const scaledDeltaY = deltaY / zoom.value;

    if (tablePositions.value[draggedTable.value]) {
      tablePositions.value[draggedTable.value] = {
        x: Math.round(dragStartCardX.value + scaledDeltaX),
        y: Math.round(dragStartCardY.value + scaledDeltaY)
      };
    }
  }
};

const handleGlobalMouseUp = () => {
  if (rafId !== null) {
    cancelAnimationFrame(rafId);
    rafId = null;
  }
  if (isDraggingCard.value) {
    savePositions();
  }
  isDraggingCanvas.value = false;
  isDraggingCard.value = false;
  draggedTable.value = null;
};

// Compute Dynamic S-curves Bezier paths with pre-indexed lookups and viewport culling
const connectionPaths = computed(() => {
  if (!schemaDetails.value) return [];

  const { indexedRelations } = schemaIndex.value;
  const hTable = hoveredTable.value;
  const hCol = hoveredColumn.value;

  const paths: Array<{
    id: string;
    path: string;
    isActive: boolean;
  }> = [];

  for (const rel of indexedRelations) {
    const sourcePos = tablePositions.value[rel.sourceTable];
    const targetPos = tablePositions.value[rel.targetTable];

    if (!sourcePos || !targetPos) continue;

    const sIdx = rel.sourceColIdx !== -1 ? rel.sourceColIdx : 0;
    const tIdx = rel.targetColIdx !== -1 ? rel.targetColIdx : 0;

    const sColY = Math.min(sIdx * ROW_HEIGHT, MAX_COLUMNS_DISPLAY_HEIGHT - ROW_HEIGHT / 2);
    const tColY = Math.min(tIdx * ROW_HEIGHT, MAX_COLUMNS_DISPLAY_HEIGHT - ROW_HEIGHT / 2);

    const sourceY = sourcePos.y + HEADER_HEIGHT + sColY + ROW_HEIGHT / 2;
    const targetY = targetPos.y + HEADER_HEIGHT + tColY + ROW_HEIGHT / 2;

    let startX = 0;
    let endX = 0;

    const sourceRight = sourcePos.x + CARD_WIDTH;
    const targetRight = targetPos.x + CARD_WIDTH;

    if (sourceRight < targetPos.x) {
      startX = sourceRight;
      endX = targetPos.x;
    } else if (targetRight < sourcePos.x) {
      startX = sourcePos.x;
      endX = targetRight;
    } else {
      if (Math.abs(sourcePos.x - targetPos.x) < Math.abs(sourceRight - targetRight)) {
        startX = sourcePos.x;
        endX = targetPos.x;
      } else {
        startX = sourceRight;
        endX = targetRight;
      }
    }

    const dx = Math.abs(endX - startX);
    const cp1x = startX + (endX > startX ? dx * 0.45 : -dx * 0.45);
    const cp2x = endX + (endX > startX ? -dx * 0.45 : dx * 0.45);

    const path = `M ${startX} ${sourceY} C ${cp1x} ${sourceY}, ${cp2x} ${targetY}, ${endX} ${targetY}`;

    const isRelationHovered =
      (hCol?.tableName === rel.sourceTable && hCol?.columnName === rel.sourceColumn) ||
      (hCol?.tableName === rel.targetTable && hCol?.columnName === rel.targetColumn) ||
      hTable === rel.sourceTable ||
      hTable === rel.targetTable;

    paths.push({
      id: rel.id,
      path,
      isActive: !!isRelationHovered
    });
  }

  return paths;
});

// Export Workspace as Standalone SVG file
const exportToSvg = () => {
  if (!schemaDetails.value) return;

  let minX = Infinity,
    maxX = -Infinity;
  let minY = Infinity,
    maxY = -Infinity;

  const names = Object.keys(tablePositions.value);
  if (names.length === 0) return;

  names.forEach((name) => {
    const pos = tablePositions.value[name];
    if (pos) {
      minX = Math.min(minX, pos.x);
      maxX = Math.max(maxX, pos.x + CARD_WIDTH);
      minY = Math.min(minY, pos.y);
      const table = schemaIndex.value.tableByName.get(name);
      const colCount = table?.columns.length || 5;
      const height = HEADER_HEIGHT + colCount * ROW_HEIGHT;
      maxY = Math.max(maxY, pos.y + height);
    }
  });

  const padding = 50;
  const width = maxX - minX + padding * 2;
  const height = maxY - minY + padding * 2;

  const shiftX = padding - minX;
  const shiftY = padding - minY;

  let svgContent = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" style="background-color: #0b0f19; font-family: sans-serif;">`;

  svgContent += `
    <style>
      .table-card { fill: #131b2e; stroke: #2a364f; stroke-width: 1.5px; rx: 8px; }
      .header-bg { fill: #1e2942; rx: 8px; }
      .text-title { fill: #ffffff; font-size: 13px; font-weight: bold; }
      .text-col { fill: #a0aec0; font-size: 11px; }
      .text-type { fill: #718096; font-size: 10px; font-style: italic; }
      .pk-icon { fill: #d69e2e; }
      .edge { fill: none; stroke: #4a5568; stroke-width: 1.5px; opacity: 0.6; }
    </style>
  `;

  // Draw connection edges shifted
  schemaDetails.value.relations.forEach((rel) => {
    const sourcePos = tablePositions.value[rel.sourceTable];
    const targetPos = tablePositions.value[rel.targetTable];
    if (!sourcePos || !targetPos) return;

    const sourceTableData = schemaIndex.value.tableByName.get(rel.sourceTable);
    const targetTableData = schemaIndex.value.tableByName.get(rel.targetTable);
    if (!sourceTableData || !targetTableData) return;

    const sourceColIdx =
      schemaIndex.value.colIndexByTable.get(rel.sourceTable)?.get(rel.sourceColumn) ?? -1;
    const targetColIdx =
      schemaIndex.value.colIndexByTable.get(rel.targetTable)?.get(rel.targetColumn) ?? -1;
    if (sourceColIdx === -1 || targetColIdx === -1) return;

    const sourceY =
      sourcePos.y + HEADER_HEIGHT + sourceColIdx * ROW_HEIGHT + ROW_HEIGHT / 2 + shiftY;
    const targetY =
      targetPos.y + HEADER_HEIGHT + targetColIdx * ROW_HEIGHT + ROW_HEIGHT / 2 + shiftY;

    let startX = 0;
    let endX = 0;
    const sourceRight = sourcePos.x + CARD_WIDTH;
    const targetRight = targetPos.x + CARD_WIDTH;

    if (sourceRight < targetPos.x) {
      startX = sourceRight + shiftX;
      endX = targetPos.x + shiftX;
    } else if (targetRight < sourcePos.x) {
      startX = sourcePos.x + shiftX;
      endX = targetRight + shiftX;
    } else {
      if (Math.abs(sourcePos.x - targetPos.x) < Math.abs(sourceRight - targetRight)) {
        startX = sourcePos.x + shiftX;
        endX = targetPos.x + shiftX;
      } else {
        startX = sourceRight + shiftX;
        endX = targetRight + shiftX;
      }
    }

    const dx = Math.abs(endX - startX);
    const cp1x = startX + (endX > startX ? dx * 0.45 : -dx * 0.45);
    const cp2x = endX + (endX > startX ? -dx * 0.45 : dx * 0.45);

    svgContent += `<path d="M ${startX} ${sourceY} C ${cp1x} ${sourceY}, ${cp2x} ${targetY}, ${endX} ${targetY}" class="edge" />`;
  });

  // Draw cards shifted
  schemaDetails.value.tables.forEach((table) => {
    const pos = tablePositions.value[table.name];
    if (!pos) return;

    const cardX = pos.x + shiftX;
    const cardY = pos.y + shiftY;
    const cardHeight = HEADER_HEIGHT + table.columns.length * ROW_HEIGHT;

    svgContent += `<g>`;
    svgContent += `<rect x="${cardX}" y="${cardY}" width="${CARD_WIDTH}" height="${cardHeight}" class="table-card" />`;
    svgContent += `<rect x="${cardX}" y="${cardY}" width="${CARD_WIDTH}" height="${HEADER_HEIGHT}" class="header-bg" />`;
    svgContent += `<text x="${cardX + 12}" y="${cardY + 24}" class="text-title">${table.name}</text>`;

    table.columns.forEach((col, idx) => {
      const colY = cardY + HEADER_HEIGHT + idx * ROW_HEIGHT + ROW_HEIGHT / 2 + 4;
      let textX = cardX + 12;

      if (col.isPrimaryKey) {
        svgContent += `<circle cx="${cardX + 16}" cy="${colY - 3}" r="4" class="pk-icon" />`;
        textX = cardX + 26;
      }

      svgContent += `<text x="${textX}" y="${colY}" class="text-col">${col.name}</text>`;
      svgContent += `<text x="${cardX + CARD_WIDTH - 12}" y="${colY}" text-anchor="end" class="text-type">${col.dataType}</text>`;
    });
    svgContent += `</g>`;
  });

  svgContent += `</svg>`;

  const blob = new Blob([svgContent], { type: 'image/svg+xml;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `${props.tab.schema || 'schema'}-er-diagram.svg`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
};
</script>

<template>
  <div class="relative flex h-full w-full flex-col overflow-hidden bg-[#070b13]">
    <!-- Diagram Toolbar -->
    <div
      class="border-border/30 bg-surface/85 z-10 flex items-center justify-between border-b px-4 py-2.5 backdrop-blur-md"
    >
      <div class="flex items-center gap-3">
        <div class="rounded-lg bg-indigo-500/10 p-1.5 text-indigo-400">
          <Database class="h-4.5 w-4.5" />
        </div>
        <div>
          <h2 class="text-text-primary text-sm font-semibold">
            {{ tab.schema }}
            <span class="text-text-tertiary text-xs font-normal">Relationship Diagram</span>
          </h2>
        </div>
      </div>

      <!-- Toolbar Controls -->
      <div class="flex items-center gap-2">
        <!-- Search bar -->
        <div class="relative flex items-center">
          <Search class="text-text-tertiary absolute left-2.5 h-3.5 w-3.5" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search tables..."
            class="border-border/40 bg-surface-elevated text-text-primary placeholder:text-text-tertiary h-8 w-44 rounded-md border pr-3 pl-8 text-xs transition-all outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/30"
          />
        </div>

        <div class="bg-border/40 mx-1 h-4 w-px"></div>

        <button
          @click="zoomOut"
          v-tooltip.bottom="'Zoom Out'"
          class="border-border/40 bg-surface-elevated text-text-secondary hover:bg-surface-hover hover:text-text-primary flex h-8 w-8 items-center justify-center rounded-md border transition-colors"
        >
          <ZoomOut class="h-4 w-4" />
        </button>

        <span class="text-text-secondary w-12 text-center font-mono text-xs select-none">
          {{ Math.round(zoom * 100) }}%
        </span>

        <button
          @click="zoomIn"
          v-tooltip.bottom="'Zoom In'"
          class="border-border/40 bg-surface-elevated text-text-secondary hover:bg-surface-hover hover:text-text-primary flex h-8 w-8 items-center justify-center rounded-md border transition-colors"
        >
          <ZoomIn class="h-4 w-4" />
        </button>

        <button
          @click="zoomToFit"
          v-tooltip.bottom="'Zoom to Fit'"
          class="border-border/40 bg-surface-elevated text-text-secondary hover:bg-surface-hover hover:text-text-primary flex h-8 w-8 items-center justify-center rounded-md border transition-colors"
        >
          <Maximize class="h-4 w-4" />
        </button>

        <button
          @click="resetZoom"
          v-tooltip.bottom="'Reset Viewport'"
          class="border-border/40 bg-surface-elevated text-text-secondary hover:bg-surface-hover hover:text-text-primary flex h-8 w-8 items-center justify-center rounded-md border transition-colors"
        >
          <RefreshCw class="h-4 w-4" />
        </button>

        <div class="bg-border/40 mx-1 h-4 w-px"></div>

        <button
          @click="performAutoLayout(true)"
          v-tooltip.bottom="'Auto Layout'"
          class="border-border/40 bg-surface-elevated text-text-secondary hover:bg-surface-hover hover:text-text-primary flex h-8 items-center gap-1.5 rounded-md border px-3 text-xs font-medium transition-colors"
        >
          <Maximize class="h-3.5 w-3.5 rotate-45" />
          <span>Auto Layout</span>
        </button>

        <button
          @click="exportToSvg"
          v-tooltip.bottom="'Export diagram to SVG image'"
          class="flex h-8 items-center gap-1.5 rounded-md border border-indigo-500/30 bg-indigo-500/10 px-3 text-xs font-medium text-indigo-400 transition-colors hover:bg-indigo-500/20"
        >
          <Download class="h-3.5 w-3.5" />
          <span>Export SVG</span>
        </button>

        <button
          @click="loadData(true)"
          v-tooltip.bottom="'Refresh Metadata'"
          class="border-border/40 bg-surface-elevated text-text-secondary hover:bg-surface-hover hover:text-text-primary flex h-8 w-8 items-center justify-center rounded-md border transition-colors"
        >
          <RefreshCw class="h-4 w-4" :class="{ 'animate-spin': isLoading }" />
        </button>
      </div>
    </div>

    <!-- Error Screen -->
    <div
      v-if="errorMsg"
      class="flex flex-1 flex-col items-center justify-center bg-[#070b13] p-8 text-center"
    >
      <div class="mb-4 rounded-2xl border border-red-500/20 bg-red-500/10 p-3 text-red-400">
        <Info class="h-10 w-10" />
      </div>
      <h3 class="text-text-primary mb-1 text-base font-semibold">Failed to Load Schema Details</h3>
      <p class="text-text-secondary mb-6 max-w-md text-sm">{{ errorMsg }}</p>
      <button
        @click="loadData(true)"
        class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-lg shadow-indigo-600/20 transition-colors hover:bg-indigo-500"
      >
        Retry Connection
      </button>
    </div>

    <!-- Loading Screen -->
    <div
      v-else-if="isLoading"
      class="flex flex-1 flex-col items-center justify-center bg-[#070b13]"
    >
      <div class="relative flex h-20 w-20 items-center justify-center">
        <div
          class="absolute h-14 w-14 animate-spin rounded-full border-4 border-indigo-500/10 border-t-indigo-500"
        ></div>
        <div
          class="animate-duration-1000 absolute h-8 w-8 animate-spin rounded-full border-4 border-indigo-500/20 border-b-indigo-400"
        ></div>
        <Database class="h-5 w-5 animate-pulse text-indigo-400" />
      </div>
      <p
        class="text-text-secondary mt-4 animate-pulse text-xs font-medium tracking-wider uppercase"
      >
        Retrieving Schema Metadata...
      </p>
    </div>

    <!-- Diagram Viewport Workspace -->
    <div
      v-else
      ref="viewportRef"
      class="relative flex-1 overflow-hidden select-none"
      @mousedown="handleCanvasMouseDown"
      @wheel="handleCanvasWheel"
    >
      <!-- Transforming Interactive Canvas (Hardware Accelerated) -->
      <div
        ref="canvasRef"
        class="pointer-events-none absolute inset-0 z-10 origin-top-left"
        :style="{
          transform: `translate(${panX}px, ${panY}px) scale(${zoom})`
        }"
      >
        <!-- SVG Connections & Background Dot Pattern Layer -->
        <svg class="pointer-events-none absolute inset-0 h-full w-full overflow-visible">
          <defs>
            <!-- Hardware-accelerated dot pattern inside transformed canvas -->
            <pattern id="grid-dots" width="24" height="24" patternUnits="userSpaceOnUse">
              <circle cx="2" cy="2" r="1" fill="rgba(255, 255, 255, 0.05)" />
            </pattern>

            <marker
              id="arrow"
              viewBox="0 0 10 10"
              refX="6"
              refY="5"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z" fill="#4a5568" />
            </marker>
            <marker
              id="arrow-active"
              viewBox="0 0 10 10"
              refX="6"
              refY="5"
              markerWidth="7"
              markerHeight="7"
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z" fill="#6366f1" />
            </marker>
          </defs>

          <!-- Dot grid background covering virtual plane with zero CPU repaint -->
          <rect x="-50000" y="-50000" width="100000" height="100000" fill="url(#grid-dots)" />

          <!-- Dynamic S-curves Connections -->
          <path
            v-for="edge in connectionPaths"
            :key="edge.id"
            :d="edge.path"
            :class="
              edge.isActive
                ? 'stroke-indigo-500 stroke-[2px] opacity-100'
                : 'stroke-border/50 stroke-[1.5px] opacity-60'
            "
            fill="none"
            :marker-end="edge.isActive ? 'url(#arrow-active)' : 'url(#arrow)'"
          />
        </svg>

        <!-- HTML Table Cards Layer -->
        <div class="pointer-events-auto absolute inset-0">
          <div
            v-for="table in filteredTables"
            :key="table.name"
            class="table-card group absolute flex cursor-grab flex-col rounded-xl border border-(--color-border) bg-[#0f172a] shadow-lg transition-colors active:cursor-grabbing"
            :class="{
              'border-indigo-500/70 ring-2 ring-indigo-500/30': hoveredTable === table.name,
              'shadow-2xl will-change-transform': draggedTable === table.name
            }"
            :style="{
              width: `${CARD_WIDTH}px`,
              transform: `translate(${tablePositions[table.name]?.x ?? 0}px, ${tablePositions[table.name]?.y ?? 0}px)`
            }"
            @mousedown="handleCardMouseDown($event, table.name)"
            @mouseenter="hoveredTable = table.name"
            @mouseleave="hoveredTable = null"
            @dblclick="handleTableDoubleClick(table.name)"
          >
            <!-- Card Header -->
            <div
              class="border-border/30 flex h-10 items-center justify-between rounded-t-xl border-b bg-[#1e293b]/70 px-3 select-none"
            >
              <div class="flex min-w-0 items-center gap-1.5">
                <Table
                  class="text-text-tertiary h-3.5 w-3.5 shrink-0 transition-colors group-hover:text-indigo-400"
                />
                <span :title="table.name" class="text-text-primary truncate text-xs font-semibold">
                  {{ table.name }}
                </span>
              </div>
              <span
                class="bg-border/40 text-text-tertiary rounded-full px-1.5 py-0.5 font-mono text-[10px] font-medium"
              >
                {{ table.columns.length }}
              </span>
            </div>

            <!-- Column list -->
            <div
              class="divide-border/10 custom-scrollbar flex max-h-[220px] flex-col divide-y overflow-y-auto py-1"
            >
              <div
                v-for="col in table.columns"
                :key="col.name"
                class="hover:bg-surface-hover/80 flex h-7 items-center justify-between gap-1 px-3 text-[11px] transition-colors"
                :class="{
                  'bg-indigo-500/10 font-medium text-indigo-400':
                    hoveredColumn?.tableName === table.name &&
                    hoveredColumn?.columnName === col.name
                }"
                @mouseenter="hoveredColumn = { tableName: table.name, columnName: col.name }"
                @mouseleave="hoveredColumn = null"
              >
                <!-- Name & Key Indicators -->
                <div
                  class="group/col relative flex min-w-0 items-center gap-1.5"
                  :title="
                    col.foreignKey
                      ? `References ${col.foreignKey.targetTable}.${col.foreignKey.targetColumn}`
                      : undefined
                  "
                >
                  <Key
                    v-if="col.isPrimaryKey"
                    class="h-3 w-3 shrink-0 text-amber-500 drop-shadow-[0_0_2px_rgba(245,158,11,0.3)] filter"
                  />
                  <Link
                    v-else-if="col.foreignKey"
                    class="h-2.5 w-2.5 shrink-0 cursor-pointer text-indigo-400 drop-shadow-[0_0_2px_rgba(99,102,241,0.3)] filter"
                    @click.stop="handleTableDoubleClick(col.foreignKey.targetTable)"
                  />
                  <span
                    class="truncate"
                    :class="
                      col.isPrimaryKey
                        ? 'font-semibold text-amber-500'
                        : col.foreignKey
                          ? 'font-medium text-indigo-400'
                          : 'text-text-secondary'
                    "
                  >
                    {{ col.name }}
                  </span>
                </div>

                <!-- Data type -->
                <span
                  :title="col.dataType"
                  class="text-text-tertiary max-w-[90px] truncate font-mono text-[9px]"
                >
                  {{ col.dataType.toLowerCase() }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style>
