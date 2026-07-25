<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections';
import { useTabsStore } from '@/stores/tabs';

import type { Tab } from '@/types';

import { LayoutGrid, Plus, Trash2, X } from '@lucide/vue';
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
          <div
            class="h-2 w-2 rounded-full"
            :class="connection?.color ? 'bg-conn-' + connection.color : 'bg-primary'"
          ></div>
          <h3 class="text-text-primary text-[15px] font-semibold">
            Open Editor: {{ connection?.name }}
          </h3>
        </div>
        <Button rounded variant="text" severity="secondary" @click="close">
          <template #icon>
            <X class="h-4 w-4" />
          </template>
        </Button>
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
          <Button
            rounded
            variant="text"
            severity="secondary"
            class="hover:text-danger! hover:bg-danger/10! opacity-0 group-hover:opacity-100"
            @click.stop="deleteTab(tab.id)"
          >
            <template #icon>
              <Trash2 class="h-4 w-4" />
            </template>
          </Button>
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
        <Button size="small" @click="createNew">
          <Plus class="h-4 w-4" />
          New
        </Button>
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
