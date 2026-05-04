<script setup lang="ts">
import type { Panel } from '@/types';
import {
  Minus,
} from 'lucide-vue-next';

const props = defineProps<{
  panel: Panel
}>()

const emit = defineEmits<{
  (e: 'minimize'): void
}>()
</script>

<template>
  <div class="flex items-center justify-between px-3 h-9 bg-sidebar border-b border-border select-none group">
    <div class="flex items-center gap-2 overflow-hidden h-full">
      <!-- Tabs Area -->
      <div class="flex items-center gap-0.5 overflow-x-auto no-scrollbar h-full">
        <button v-for="tab in panel.tabs" :key="tab.id"
          class="px-3 h-full text-[12px] font-medium transition-colors whitespace-nowrap border-b-2" :class="panel.activeTabId === tab.id
            ? 'border-primary text-primary bg-primary/5'
            : 'border-transparent text-text-secondary hover:text-text-primary hover:bg-hover/50'"
          @click="panel.activeTabId = tab.id">
          {{ tab.title }}
        </button>
      </div>
    </div>

    <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
      <!-- Minimize (Hide) -->
      <button class="p-1 rounded hover:bg-white/10 text-text-tertiary hover:text-text-primary relative group/tooltip"
        @click="emit('minimize')">
        <Minus :size="14" />
        <!-- Tooltip -->
        <div
          class="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 px-2 py-1 rounded bg-surface border border-border shadow-xl text-text-primary text-[10px] font-medium whitespace-nowrap opacity-0 group-hover/tooltip:opacity-100 pointer-events-none transition-all translate-y-1 group-hover/tooltip:translate-y-0 z-50">
          Hide Panel
          <div
            class="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-surface border-b border-r border-border rotate-45">
          </div>
        </div>
      </button>
    </div>
  </div>
</template>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}

.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
