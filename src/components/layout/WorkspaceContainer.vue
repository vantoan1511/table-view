<script setup lang="ts">
import { computed } from 'vue'
import { useLayoutStore } from '@/stores/layout'
import ResizeHandle from '@/components/ui/ResizeHandle.vue'
import PanelHeader from '@/components/layout/PanelHeader.vue'
import OutputPanel from '@/components/panels/OutputPanel.vue'
import TimelinePanel from '@/components/panels/TimelinePanel.vue'
import PropertiesPanel from '@/components/panels/PropertiesPanel.vue'
import IndexesPanel from '@/components/panels/IndexesPanel.vue'

const layoutStore = useLayoutStore()

const bottomPanel = computed(() => layoutStore.bottomPanel)
const rightPanel = computed(() => layoutStore.rightPanel)

const isBottomVisible = computed(() => bottomPanel.value && bottomPanel.value.isVisible && !bottomPanel.value.isMinimized)
const isRightVisible = computed(() => rightPanel.value && rightPanel.value.isVisible && !rightPanel.value.isMinimized)
</script>

<template>
  <div class="flex flex-col flex-1 min-w-0 min-h-0 bg-background relative">
    <!-- Top Area (Main + Right) -->
    <div class="flex flex-1 min-h-0 min-w-0">
      <!-- Main Editor / Grid Area -->
      <div class="flex-1 min-w-0 min-h-0 flex flex-col relative">
        <slot name="main" />
      </div>

      <!-- Right Panel -->
      <template v-if="isRightVisible && rightPanel">
        <ResizeHandle
          orientation="horizontal"
          :model-value="rightPanel.size"
          reverse
          @update:model-value="layoutStore.updatePanelSize(rightPanel.id, $event)"
        />
        <div 
          class="flex flex-col bg-sidebar border-l border-border animate-slide-in-right"
          :style="{ width: `${rightPanel.size}px` }"
        >
          <PanelHeader 
            :panel="rightPanel"
            @minimize="layoutStore.minimizePanel(rightPanel.id)"
            @close="layoutStore.togglePanel(rightPanel.id)"
            @reposition="layoutStore.movePanel(rightPanel.id, $event)"
          />
          <div class="flex-1 overflow-hidden flex flex-col">
            <PropertiesPanel v-if="rightPanel.activeTabId === 'properties'" />
            <IndexesPanel v-else-if="rightPanel.activeTabId === 'indexes'" />
            <!-- Fallback if tab ID is unknown -->
            <div v-else class="flex-1 p-4 text-sm text-muted-foreground">
              Select a tab to view content.
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Bottom Panel -->
    <template v-if="isBottomVisible && bottomPanel">
      <ResizeHandle
        orientation="vertical"
        :model-value="bottomPanel.size"
        reverse
        @update:model-value="layoutStore.updatePanelSize(bottomPanel.id, $event)"
      />
      <div 
        class="flex flex-col bg-sidebar border-t border-border animate-slide-in-up"
        :style="{ height: `${bottomPanel.size}px` }"
      >
        <PanelHeader 
          :panel="bottomPanel"
          @minimize="layoutStore.minimizePanel(bottomPanel.id)"
          @close="layoutStore.togglePanel(bottomPanel.id)"
          @reposition="layoutStore.movePanel(bottomPanel.id, $event)"
        />
        <div class="flex-1 overflow-hidden flex flex-col bg-background">
          <OutputPanel v-if="bottomPanel.activeTabId === 'output'" />
          <TimelinePanel v-else-if="bottomPanel.activeTabId === 'timeline'" />
          <div v-else class="flex-1 p-4 text-sm text-muted-foreground">
            Select a tab to view content.
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
