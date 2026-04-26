<script setup lang="ts">
import { useTabsStore } from '@/stores/tabs'
import Docker from '@/components/ui/Docker.vue'
import { LayoutGrid, X } from 'lucide-vue-next'

const tabsStore = useTabsStore()

const restoreTab = (id: string) => {
  tabsStore.restoreTab(id)
}

const closeTab = (id: string) => {
  tabsStore.closeTab(id)
}
</script>

<template>
  <Docker v-if="tabsStore.minimizedTabs.length > 0" position="bottom" class="shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.05)] z-20">
    <div class="text-[11px] font-semibold text-text-tertiary px-2 uppercase tracking-wider select-none shrink-0">
      Minimized
    </div>
    
    <button
      v-for="tab in tabsStore.minimizedTabs"
      :key="tab.id"
      class="group flex items-center gap-1.5 px-2.5 py-1 bg-muted border border-border rounded-md text-[12px] text-text-secondary hover:bg-hover hover:text-text-primary hover:border-border-strong transition-colors max-w-[200px] shrink-0 cursor-pointer"
      @click="restoreTab(tab.id)"
      title="Click to restore"
    >
      <LayoutGrid :size="12" class="text-primary opacity-70 group-hover:opacity-100" />
      <span class="truncate flex-1 text-left">{{ tab.title }}</span>
      <span
        class="flex items-center justify-center w-4 h-4 rounded text-text-tertiary opacity-0 group-hover:opacity-100 hover:bg-border hover:text-text-primary transition-all ml-1"
        @click.stop="closeTab(tab.id)"
        title="Close tab"
      >
        <X :size="12" />
      </span>
    </button>
  </Docker>
</template>
