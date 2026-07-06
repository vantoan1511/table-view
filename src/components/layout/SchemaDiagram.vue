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
import { computed, onMounted, onUnmounted, ref } from 'vue';

import { useDiagramStore } from '@/stores/diagram';
import { useTabsStore } from '@/stores/tabs';
import type { Tab } from '@/types';

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

// References
const viewportRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLDivElement | null>(null);

// Constants for layout
const CARD_WIDTH = 220;
const HEADER_HEIGHT = 40;
const ROW_HEIGHT = 28;

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

// Elegant auto-grid layout function to layout tables in rows
const performAutoLayout = (force = false) => {
  if (!schemaDetails.value) return;

  const tables = schemaDetails.value.tables;
  const cols = Math.ceil(Math.sqrt(tables.length));
  const spacingX = 320;
  const spacingY = 220;

  const newPositions: Record<string, { x: number; y: number }> = {};

  tables.forEach((table, idx) => {
    // If it was already loaded and we're not forcing, keep original
    if (!force && tablePositions.value[table.name]) {
      newPositions[table.name] = { ...tablePositions.value[table.name]! };
      return;
    }
    const r = Math.floor(idx / cols);
    const c = idx % cols;
    newPositions[table.name] = {
      x: 100 + c * spacingX,
      y: 100 + r * (spacingY + Math.min(table.columns.length * 15, 100))
    };
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

const getForeignKeyReference = (tableName: string, columnName: string) => {
  if (!schemaDetails.value) return null;
  return (
    schemaDetails.value.relations.find(
      (r) => r.sourceTable === tableName && r.sourceColumn === columnName
    ) || null
  );
};

const handleTableDoubleClick = (tableName: string) => {
  tabsStore.openTableTab(tableName, props.tab.schema, props.tab.connectionId, props.tab.dbName);
};

// Fetch on mount
onMounted(() => {
  loadData();
  window.addEventListener('mouseup', handleGlobalMouseUp);
  window.addEventListener('mousemove', handleGlobalMouseMove);
});

onUnmounted(() => {
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
      // approximate card height
      const colCount =
        schemaDetails.value?.tables.find((t) => t.name === name)?.columns.length || 5;
      const height = HEADER_HEIGHT + colCount * ROW_HEIGHT;
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

  // Zoom towards mouse cursor coordinates
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

// Global Mouse Movement handling
const handleGlobalMouseMove = (e: MouseEvent) => {
  if (isDraggingCanvas.value) {
    const deltaX = e.clientX - dragStartMouseX.value;
    const deltaY = e.clientY - dragStartMouseY.value;
    panX.value = canvasStartPanX.value + deltaX;
    panY.value = canvasStartPanY.value + deltaY;
  } else if (isDraggingCard.value && draggedTable.value) {
    const deltaX = e.clientX - dragStartMouseX.value;
    const deltaY = e.clientY - dragStartMouseY.value;

    // Crucial: Adjust card delta by current zoom factor so movement matches cursor 1:1!
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
  if (isDraggingCard.value) {
    savePositions();
  }
  isDraggingCanvas.value = false;
  isDraggingCard.value = false;
  draggedTable.value = null;
};

// Compute Dynamic S-curves Bezier paths between tables
const connectionPaths = computed(() => {
  if (!schemaDetails.value) return [];

  const paths: Array<{
    id: string;
    path: string;
    isActive: boolean;
  }> = [];

  schemaDetails.value.relations.forEach((rel, index) => {
    const sourcePos = tablePositions.value[rel.sourceTable];
    const targetPos = tablePositions.value[rel.targetTable];

    if (!sourcePos || !targetPos) return;

    // Find table columns to get vertical offset index
    const sourceTableData = schemaDetails.value?.tables.find((t) => t.name === rel.sourceTable);
    const targetTableData = schemaDetails.value?.tables.find((t) => t.name === rel.targetTable);

    if (!sourceTableData || !targetTableData) return;

    const sourceColIdx = sourceTableData.columns.findIndex((c) => c.name === rel.sourceColumn);
    const targetColIdx = targetTableData.columns.findIndex((c) => c.name === rel.targetColumn);

    if (sourceColIdx === -1 || targetColIdx === -1) return;

    // Calculate Y offsets inside the card
    const sourceY = sourcePos.y + HEADER_HEIGHT + sourceColIdx * ROW_HEIGHT + ROW_HEIGHT / 2;
    const targetY = targetPos.y + HEADER_HEIGHT + targetColIdx * ROW_HEIGHT + ROW_HEIGHT / 2;

    let startX = 0;
    let endX = 0;

    // Determine dynamic ports based on horizontal relative layouts
    const sourceRight = sourcePos.x + CARD_WIDTH;
    const targetRight = targetPos.x + CARD_WIDTH;

    if (sourceRight < targetPos.x) {
      // Source is clearly to the left of Target
      startX = sourceRight;
      endX = targetPos.x;
    } else if (targetRight < sourcePos.x) {
      // Target is clearly to the left of Source
      startX = sourcePos.x;
      endX = targetRight;
    } else {
      // Overlapping horizontally: connect closest sides
      if (Math.abs(sourcePos.x - targetPos.x) < Math.abs(sourceRight - targetRight)) {
        startX = sourcePos.x;
        endX = targetPos.x;
      } else {
        startX = sourceRight;
        endX = targetRight;
      }
    }

    const dx = Math.abs(endX - startX);
    // Draw Bezier S-curve
    const cp1x = startX + (endX > startX ? dx * 0.45 : -dx * 0.45);
    const cp2x = endX + (endX > startX ? -dx * 0.45 : dx * 0.45);

    const path = `M ${startX} ${sourceY} C ${cp1x} ${sourceY}, ${cp2x} ${targetY}, ${endX} ${targetY}`;

    // Active highlights if hovered
    const isRelationHovered =
      (hoveredColumn.value?.tableName === rel.sourceTable &&
        hoveredColumn.value?.columnName === rel.sourceColumn) ||
      (hoveredColumn.value?.tableName === rel.targetTable &&
        hoveredColumn.value?.columnName === rel.targetColumn) ||
      hoveredTable.value === rel.sourceTable ||
      hoveredTable.value === rel.targetTable;

    paths.push({
      id: `${rel.constraintName}-${index}`,
      path,
      isActive: !!isRelationHovered
    });
  });

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
      const colCount =
        schemaDetails.value?.tables.find((t) => t.name === name)?.columns.length || 5;
      const height = HEADER_HEIGHT + colCount * ROW_HEIGHT;
      maxY = Math.max(maxY, pos.y + height);
    }
  });

  const padding = 50;
  const width = maxX - minX + padding * 2;
  const height = maxY - minY + padding * 2;

  const shiftX = padding - minX;
  const shiftY = padding - minY;

  // Construct dynamic SVG string
  let svgContent = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" style="background-color: #0b0f19; font-family: sans-serif;">`;

  // Add CSS styles
  svgContent += `
    <style>
      .table-card { fill: #131b2e; stroke: #2a364f; stroke-width: 1.5px; rx: 8px; }
      .header-bg { fill: #1e2942; rx: 8px; }
      .text-title { fill: #ffffff; font-size: 13px; font-weight: bold; }
      .text-col { fill: #a0aec0; font-size: 11px; }
      .text-type { fill: #718096; font-size: 10px; font-style: italic; }
      .pk-icon { fill: #d69e2e; }
      .edge { fill: none; stroke: #4a5568; stroke-width: 1.5px; opacity: 0.6; }
      .edge-active { fill: none; stroke: #3182ce; stroke-width: 2px; opacity: 1.0; }
    </style>
  `;

  // Draw connection edges shifted
  schemaDetails.value.relations.forEach((rel) => {
    const sourcePos = tablePositions.value[rel.sourceTable];
    const targetPos = tablePositions.value[rel.targetTable];
    if (!sourcePos || !targetPos) return;

    const sourceTableData = schemaDetails.value?.tables.find((t) => t.name === rel.sourceTable);
    const targetTableData = schemaDetails.value?.tables.find((t) => t.name === rel.targetTable);
    if (!sourceTableData || !targetTableData) return;

    const sourceColIdx = sourceTableData.columns.findIndex((c) => c.name === rel.sourceColumn);
    const targetColIdx = targetTableData.columns.findIndex((c) => c.name === rel.targetColumn);
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

    // Base card shadow/container
    svgContent += `<g>`;
    svgContent += `<rect x="${cardX}" y="${cardY}" width="${CARD_WIDTH}" height="${cardHeight}" class="table-card" />`;
    // Header
    svgContent += `<rect x="${cardX}" y="${cardY}" width="${CARD_WIDTH}" height="${HEADER_HEIGHT}" class="header-bg" />`;
    svgContent += `<text x="${cardX + 12}" y="${cardY + 24}" class="text-title">${table.name}</text>`;

    // Columns
    table.columns.forEach((col, idx) => {
      const colY = cardY + HEADER_HEIGHT + idx * ROW_HEIGHT + ROW_HEIGHT / 2 + 4;
      let textX = cardX + 12;

      // Draw PK gold circle if primary key
      if (col.isPrimaryKey) {
        svgContent += `<circle cx="${cardX + 16}" cy="${colY - 3}" r="4" class="pk-icon" />`;
        textX = cardX + 26;
      }

      svgContent += `<text x="${textX}" y="${colY}" class="text-col">${col.name}</text>`;
      // Data type right aligned
      svgContent += `<text x="${cardX + CARD_WIDTH - 12}" y="${colY}" text-anchor="end" class="text-type">${col.dataType}</text>`;
    });
    svgContent += `</g>`;
  });

  svgContent += `</svg>`;

  // Download standard blob
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
          v-tooltip.bottom="'Regrid Auto Layout'"
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
      <!-- Background Interactive Grid dots -->
      <div
        class="absolute inset-0 z-0 bg-[#070b13]"
        :style="{
          backgroundImage: 'radial-gradient(circle, rgba(255,255,255,0.035) 1px, transparent 1px)',
          backgroundSize: '24px 24px',
          backgroundPosition: `${panX}px ${panY}px`
        }"
      ></div>

      <!-- Transforming Interactive Canvas -->
      <div
        ref="canvasRef"
        class="pointer-events-none absolute inset-0 z-10 origin-top-left"
        :style="{
          transform: `translate(${panX}px, ${panY}px) scale(${zoom})`
        }"
      >
        <!-- SVG Connections Layer -->
        <svg class="pointer-events-none absolute inset-0 h-full w-full overflow-visible">
          <defs>
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

          <path
            v-for="edge in connectionPaths"
            :key="edge.id"
            :d="edge.path"
            :class="
              edge.isActive
                ? 'stroke-indigo-500 stroke-[2px] opacity-100'
                : 'stroke-border/40 stroke-[1.5px] opacity-60'
            "
            fill="none"
            :marker-end="edge.isActive ? 'url(#arrow-active)' : 'url(#arrow)'"
            class="transition-all duration-150"
          />
        </svg>

        <!-- HTML Table Cards Layer -->
        <div class="pointer-events-auto absolute inset-0">
          <div
            v-for="table in filteredTables"
            :key="table.name"
            class="table-card bg-surface-elevated/95 border-border/40 group absolute flex cursor-grab flex-col rounded-xl border shadow-xl backdrop-blur-md transition-colors transition-shadow hover:border-indigo-500/50 hover:shadow-2xl active:cursor-grabbing"
            :class="{
              'border-indigo-500/40 ring-2 ring-indigo-500/30': hoveredTable === table.name,
              'opacity-40 hover:opacity-100': hoveredTable && hoveredTable !== table.name
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
              class="bg-surface/90 border-border/30 flex h-10 items-center justify-between rounded-t-xl border-b px-3 select-none"
            >
              <div class="flex min-w-0 items-center gap-1.5">
                <Table
                  class="text-text-tertiary h-3.5 w-3.5 shrink-0 transition-colors group-hover:text-indigo-400"
                />
                <span
                  v-tooltip.top="table.name"
                  class="text-text-primary truncate text-xs font-semibold"
                >
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
                  'bg-indigo-500/5 font-medium text-indigo-400':
                    hoveredColumn?.tableName === table.name &&
                    hoveredColumn?.columnName === col.name
                }"
                @mouseenter="hoveredColumn = { tableName: table.name, columnName: col.name }"
                @mouseleave="hoveredColumn = null"
              >
                <!-- Name & Key Indicators -->
                <div
                  class="group/col relative flex min-w-0 items-center gap-1.5"
                  v-tooltip.top="
                    getForeignKeyReference(table.name, col.name)
                      ? `References ${getForeignKeyReference(table.name, col.name)!.targetTable}.${getForeignKeyReference(table.name, col.name)!.targetColumn}`
                      : null
                  "
                >
                  <Key
                    v-if="col.isPrimaryKey"
                    class="h-3 w-3 shrink-0 text-amber-500 drop-shadow-[0_0_2px_rgba(245,158,11,0.3)] filter"
                  />
                  <Link
                    v-else-if="getForeignKeyReference(table.name, col.name)"
                    class="h-2.5 w-2.5 shrink-0 cursor-pointer text-indigo-400 drop-shadow-[0_0_2px_rgba(99,102,241,0.3)] filter"
                    @click.stop="
                      handleTableDoubleClick(
                        getForeignKeyReference(table.name, col.name)!.targetTable
                      )
                    "
                  />
                  <span
                    class="truncate"
                    :class="
                      col.isPrimaryKey
                        ? 'font-semibold text-amber-500'
                        : getForeignKeyReference(table.name, col.name)
                          ? 'font-medium text-indigo-400'
                          : 'text-text-secondary'
                    "
                  >
                    {{ col.name }}
                  </span>
                </div>

                <!-- Data type -->
                <span
                  v-tooltip.left="col.dataType"
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
