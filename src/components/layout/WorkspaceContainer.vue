<script setup lang="ts">
import { computed } from 'vue'
import { useLayoutStore } from '@/stores/layout'
import ResizeHandle from '@/components/ui/ResizeHandle.vue'
import PanelHeader from '@/components/layout/PanelHeader.vue'
import Skeleton from '@/components/ui/Skeleton.vue'

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
          <div class="flex-1 overflow-auto p-4">
            <div class="text-xs text-muted-foreground uppercase tracking-wider font-semibold mb-4">
              {{ rightPanel?.tabs.find(t => t.id === rightPanel?.activeTabId)?.title }}
            </div>
            <!-- Placeholder for panel content -->
            <div class="flex flex-col gap-3">
              <Skeleton v-for="i in 5" :key="i" height="2rem" />
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
        <div class="flex-1 overflow-auto p-4 font-mono text-sm">
           <div class="text-xs text-muted-foreground uppercase tracking-wider font-semibold mb-2">
            {{ bottomPanel?.tabs.find(t => t.id === bottomPanel?.activeTabId)?.title }}
          </div>
          <div class="text-green-400/80">$ Query executed successfully in 42ms</div>
          <div class="text-muted-foreground">SELECT * FROM users LIMIT 100;</div>
        </div>
      </div>
    </template>
  </div>
</template>
