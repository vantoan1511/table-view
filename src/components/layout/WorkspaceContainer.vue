<script setup lang="ts">
import PanelHeader from '@/components/layout/PanelHeader.vue';
import ResizeHandle from '@/components/ui/ResizeHandle.vue';
import { useLayoutStore } from '@/stores/layout';
import { defineAsyncComponent } from 'vue';

const IndexesPanel = defineAsyncComponent(() => import('@/components/panels/IndexesPanel.vue'));
const OutputPanel = defineAsyncComponent(() => import('@/components/panels/OutputPanel.vue'));
const PropertiesPanel = defineAsyncComponent(
  () => import('@/components/panels/PropertiesPanel.vue')
);
const TimelinePanel = defineAsyncComponent(() => import('@/components/panels/TimelinePanel.vue'));
const ValueViewer = defineAsyncComponent(() => import('@/components/panels/ValueViewer.vue'));

const layoutStore = useLayoutStore();
</script>

<template>
  <div class="bg-background relative flex min-h-0 min-w-0 flex-1 flex-col">
    <!-- Top Area (Main + Right) -->
    <div class="flex min-h-0 min-w-0 flex-1">
      <!-- Main Editor / Grid Area -->
      <div class="relative flex min-h-0 min-w-0 flex-1 flex-col">
        <slot name="main" />
      </div>

      <!-- Right Panel (Inspector) -->
      <template v-if="layoutStore.isRightVisible && layoutStore.rightPanel">
        <ResizeHandle
          orientation="horizontal"
          :model-value="layoutStore.rightPanel.size"
          reverse
          @update:model-value="layoutStore.updatePanelSize('inspector', $event)"
        />
        <div
          class="bg-sidebar border-border animate-slide-in-right flex flex-col border-l shadow-lg"
          :style="{ width: `${layoutStore.rightPanel.size}px` }"
        >
          <PanelHeader
            :panel="layoutStore.rightPanel"
            @minimize="layoutStore.togglePanel('inspector')"
          />
          <div class="flex flex-1 flex-col overflow-hidden">
            <PropertiesPanel v-if="layoutStore.rightPanel.activeTabId === 'properties'" />
            <IndexesPanel v-else-if="layoutStore.rightPanel.activeTabId === 'indexes'" />
            <div v-else class="text-text-tertiary flex-1 p-4 text-sm">
              Select a tab to view content.
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Bottom Panel (Console) -->
    <template v-if="layoutStore.isBottomVisible && layoutStore.bottomPanel">
      <ResizeHandle
        orientation="vertical"
        :model-value="layoutStore.bottomPanel.size"
        reverse
        @update:model-value="layoutStore.updatePanelSize('console', $event)"
      />
      <div
        class="bg-sidebar border-border animate-slide-in-up flex flex-col border-t shadow-lg"
        :style="{ height: `${layoutStore.bottomPanel.size}px` }"
      >
        <PanelHeader
          :panel="layoutStore.bottomPanel"
          @minimize="layoutStore.togglePanel('console')"
        />
        <div class="bg-background flex flex-1 flex-col overflow-hidden">
          <OutputPanel v-if="layoutStore.bottomPanel.activeTabId === 'output'" />
          <TimelinePanel v-else-if="layoutStore.bottomPanel.activeTabId === 'timeline'" />
          <ValueViewer v-else-if="layoutStore.bottomPanel.activeTabId === 'value'" />
          <div v-else class="text-text-tertiary flex-1 p-4 text-sm">
            Select a tab to view content.
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
