<script setup lang="ts">
import Docker from '@/components/ui/Docker.vue';
import { useTabsStore } from '@/stores/tabs';
import { LayoutGrid, X } from '@lucide/vue';

const tabsStore = useTabsStore();

const restoreTab = (id: string) => {
  tabsStore.restoreTab(id);
};

const closeTab = (id: string) => {
  tabsStore.closeTab(id);
};
</script>

<template>
  <Docker
    v-if="tabsStore.minimizedTabs.length > 0"
    position="bottom"
    class="z-20 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.05)]"
  >
    <div
      class="text-text-tertiary shrink-0 px-2 text-[11px] font-semibold tracking-wider uppercase select-none"
    >
      Minimized
    </div>

    <button
      v-for="tab in tabsStore.minimizedTabs"
      :key="tab.id"
      v-tooltip.top="'Click to restore'"
      class="group bg-muted border-border text-text-secondary hover:bg-hover hover:text-text-primary hover:border-border-strong flex max-w-[200px] shrink-0 cursor-pointer items-center gap-1.5 rounded-md border px-2.5 py-1 text-[12px] transition-colors"
      @click="restoreTab(tab.id)"
    >
      <LayoutGrid :size="12" class="text-primary opacity-70 group-hover:opacity-100" />
      <span class="flex-1 truncate text-left">{{ tab.title }}</span>
      <span
        v-tooltip.top="'Close tab'"
        class="text-text-tertiary hover:bg-border hover:text-text-primary ml-1 flex h-4 w-4 items-center justify-center rounded opacity-0 transition-all group-hover:opacity-100"
        @click.stop="closeTab(tab.id)"
      >
        <X :size="12" />
      </span>
    </button>
  </Docker>
</template>
