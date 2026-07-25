<script setup lang="ts">
import type { Panel } from '@/types';

import { useLayoutStore } from '@/stores/layout';

import { Minus } from 'lucide-vue-next';

const props = defineProps<{
  panel: Panel;
}>();

const emit = defineEmits<{
  (e: 'minimize'): void;
}>();

const layoutStore = useLayoutStore();
</script>

<template>
  <div
    class="bg-sidebar border-border group flex h-9 items-center justify-between border-b px-3 select-none"
  >
    <div class="flex h-full items-center gap-2 overflow-hidden">
      <!-- Tabs Area -->
      <div class="no-scrollbar flex h-full items-center gap-0.5 overflow-x-auto">
        <button
          v-for="tab in panel.tabs"
          :key="tab.id"
          class="h-full border-b-2 px-3 text-[12px] font-medium whitespace-nowrap transition-colors"
          :class="
            panel.activeTabId === tab.id
              ? 'border-primary text-primary bg-primary/5'
              : 'text-text-secondary hover:text-text-primary hover:bg-hover/50 border-transparent'
          "
          @click="layoutStore.setActiveTab(props.panel.id, tab.id)"
        >
          {{ tab.title }}
        </button>
      </div>
    </div>

    <div class="flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
      <!-- Minimize (Hide) -->
      <button
        v-tooltip.top="'Hide Panel'"
        class="text-text-tertiary hover:text-text-primary rounded p-1 hover:bg-white/10"
        @click="emit('minimize')"
      >
        <Minus :size="14" />
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
