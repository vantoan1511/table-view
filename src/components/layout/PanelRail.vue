<script setup lang="ts">
import { useLayoutStore } from '@/stores/layout'
import { Terminal, Info, ChevronLeft, ChevronRight } from 'lucide-vue-next'

const layoutStore = useLayoutStore()

const getIcon = (id: string) => {
  switch (id) {
    case 'console': return Terminal
    case 'inspector': return Info
    default: return Terminal
  }
}
</script>

<template>
  <div 
    v-if="layoutStore.minimizedPanels.length > 0"
    class="flex flex-col items-center py-3 w-10 border-l border-border bg-sidebar gap-4 shadow-[inline_-1px_0_0_0_rgba(0,0,0,0.1)]"
  >
    <button
      v-for="panel in layoutStore.minimizedPanels"
      :key="panel.id"
      class="p-2 rounded-lg text-text-tertiary hover:text-primary hover:bg-primary/10 transition-all group relative active:scale-95"
      @click="layoutStore.restorePanel(panel.id)"
    >
      <component :is="getIcon(panel.id)" :size="20" stroke-width="2" />
      
      <!-- Hover Tooltip/Label -->
      <div class="absolute right-full mr-3 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[11px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-all translate-x-1 group-hover:translate-x-0 z-50">
        {{ panel.title }}
        <!-- Small arrow -->
        <div class="absolute top-1/2 -right-1 -translate-y-1/2 w-2 h-2 bg-surface border-t border-r border-border rotate-45"></div>
      </div>
    </button>
  </div>
</template>

<style scoped>
.vertical-text {
  writing-mode: vertical-rl;
  text-orientation: mixed;
}
</style>
