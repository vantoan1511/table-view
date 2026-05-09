<script setup lang="ts">
import type { Panel } from '@/types';
import { Minus } from 'lucide-vue-next';

const props = defineProps<{
  panel: Panel;
}>();

const emit = defineEmits<{
  (e: 'minimize'): void;
}>();
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
          @click="panel.activeTabId = tab.id"
        >
          {{ tab.title }}
        </button>
      </div>
    </div>

    <div class="flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
      <!-- Minimize (Hide) -->
      <button
        class="text-text-tertiary hover:text-text-primary group/tooltip relative rounded p-1 hover:bg-white/10"
        @click="emit('minimize')"
      >
        <Minus :size="14" />
        <!-- Tooltip -->
        <div
          class="bg-surface border-border text-text-primary pointer-events-none absolute bottom-full left-1/2 z-50 mb-2 -translate-x-1/2 translate-y-1 rounded border px-2 py-1 text-[10px] font-medium whitespace-nowrap opacity-0 shadow-xl transition-all group-hover/tooltip:translate-y-0 group-hover/tooltip:opacity-100"
        >
          Hide Panel
          <div
            class="bg-surface border-border absolute -bottom-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-r border-b"
          ></div>
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
