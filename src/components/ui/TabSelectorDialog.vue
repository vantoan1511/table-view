<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections';
import { useTabsStore } from '@/stores/tabs';
import type { Tab } from '@/types';
import { LayoutGrid, Plus, Trash2, X } from 'lucide-vue-next';
import { computed } from 'vue';

const tabsStore = useTabsStore();
const connectionsStore = useConnectionsStore();

const connection = computed(() =>
  connectionsStore.connections.find((c) => c.id === tabsStore.selectorConnectionId)
);

const connectionTabs = computed(() =>
  tabsStore.tabs.filter(
    (t: Tab) => t.type === 'sql' && t.connectionId === tabsStore.selectorConnectionId
  )
);

const close = () => {
  tabsStore.showTabSelector = false;
  tabsStore.selectorConnectionId = null;
};

const selectTab = (id: string) => {
  const tab = tabsStore.tabs.find((t: Tab) => t.id === id);
  if (tab) {
    tab.closed = false;
    tab.minimized = false;
    tabsStore.setActiveTab(id);
  }
  close();
};

const createNew = () => {
  if (tabsStore.selectorConnectionId) {
    tabsStore.openSqlEditor(tabsStore.selectorConnectionId, '', true, true);
  }
  close();
};

const deleteTab = (id: string) => {
  tabsStore.deleteTab(id);
};
</script>

<template>
  <div
    v-if="tabsStore.showTabSelector"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
    @click.self="close"
  >
    <div
      class="bg-surface border-border flex max-h-[80vh] w-full max-w-md flex-col overflow-hidden rounded-xl border shadow-2xl"
    >
      <!-- Header -->
      <div class="border-border bg-muted flex items-center justify-between border-b p-4">
        <div class="flex items-center gap-2">
          <div class="h-2 w-2 rounded-full" :class="'bg-' + (connection?.color || 'primary')"></div>
          <h3 class="text-text-primary text-[15px] font-semibold">
            Open Editor: {{ connection?.name }}
          </h3>
        </div>
        <button
          class="hover:bg-hover text-text-tertiary rounded-md p-1 transition-colors"
          @click="close"
        >
          <X :size="18" />
        </button>
      </div>

      <!-- List -->
      <div class="custom-scrollbar flex-1 space-y-1 overflow-y-auto p-2">
        <div
          v-for="tab in connectionTabs"
          :key="tab.id"
          class="group hover:bg-hover hover:border-border flex cursor-pointer items-center gap-3 rounded-lg border border-transparent p-3 transition-colors"
          @click="selectTab(tab.id)"
        >
          <div
            class="bg-muted text-primary group-hover:bg-surface rounded-md p-2 transition-colors"
          >
            <LayoutGrid :size="16" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="text-text-primary truncate text-[14px] font-medium">{{ tab.title }}</div>
            <div class="text-text-tertiary truncate text-[11px]">
              {{ tab.filePath || (tab.isDraft ? 'Draft' : 'Autosaved') }}
              <span v-if="tab.closed" class="text-warning ml-2">• Closed</span>
            </div>
          </div>
          <button
            class="hover:text-danger hover:bg-danger/10 rounded-md p-2 opacity-0 transition-all group-hover:opacity-100"
            @click.stop="deleteTab(tab.id)"
            title="Delete permanently"
          >
            <Trash2 :size="14" />
          </button>
        </div>

        <div
          v-if="connectionTabs.length === 0"
          class="text-text-tertiary py-8 text-center text-[13px] italic"
        >
          No editors found for this connection.
        </div>
      </div>

      <!-- Footer -->
      <div class="border-border bg-muted flex items-center justify-between border-t p-4">
        <span class="text-text-tertiary text-[12px]"
          >{{ connectionTabs.length }} editors found</span
        >
        <button
          class="bg-primary hover:bg-primary-hover text-text-inverse flex items-center gap-1.5 rounded-lg px-4 py-2 text-[13px] font-medium shadow-sm transition-colors"
          @click="createNew"
        >
          <Plus :size="16" />
          Create New Editor
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-strong);
}
</style>
