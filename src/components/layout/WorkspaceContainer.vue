<script setup lang="ts">
import { useLayoutStore } from '@/stores/layout'
import ResizeHandle from '@/components/ui/ResizeHandle.vue'
import PanelHeader from '@/components/layout/PanelHeader.vue'
import OutputPanel from '@/components/panels/OutputPanel.vue'
import TimelinePanel from '@/components/panels/TimelinePanel.vue'
import ValueViewer from '@/components/panels/ValueViewer.vue'
import PropertiesPanel from '@/components/panels/PropertiesPanel.vue'
import IndexesPanel from '@/components/panels/IndexesPanel.vue'

const layoutStore = useLayoutStore()
</script>

<template>
  <div class="flex flex-col flex-1 min-w-0 min-h-0 bg-background relative">
    <!-- Top Area (Main + Right) -->
    <div class="flex flex-1 min-h-0 min-w-0">
      <!-- Main Editor / Grid Area -->
      <div class="flex-1 min-w-0 min-h-0 flex flex-col relative">
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
          class="flex flex-col bg-sidebar border-l border-border animate-slide-in-right shadow-lg"
          :style="{ width: `${layoutStore.rightPanel.size}px` }"
        >
          <PanelHeader 
            :panel="layoutStore.rightPanel"
            @minimize="layoutStore.togglePanel('inspector')"
          />
          <div class="flex-1 overflow-hidden flex flex-col">
            <PropertiesPanel v-if="layoutStore.rightPanel.activeTabId === 'properties'" />
            <IndexesPanel v-else-if="layoutStore.rightPanel.activeTabId === 'indexes'" />
            <div v-else class="flex-1 p-4 text-sm text-text-tertiary">
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
        class="flex flex-col bg-sidebar border-t border-border animate-slide-in-up shadow-lg"
        :style="{ height: `${layoutStore.bottomPanel.size}px` }"
      >
        <PanelHeader 
          :panel="layoutStore.bottomPanel"
          @minimize="layoutStore.togglePanel('console')"
        />
        <div class="flex-1 overflow-hidden flex flex-col bg-background">
          <OutputPanel v-if="layoutStore.bottomPanel.activeTabId === 'output'" />
          <TimelinePanel v-else-if="layoutStore.bottomPanel.activeTabId === 'timeline'" />
          <ValueViewer v-else-if="layoutStore.bottomPanel.activeTabId === 'value'" />
          <div v-else class="flex-1 p-4 text-sm text-text-tertiary">
            Select a tab to view content.
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
