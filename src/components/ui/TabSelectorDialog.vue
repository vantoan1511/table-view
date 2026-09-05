<script setup lang="ts">
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import { useConnectionsStore } from '@/stores/connections';
import { useTabsStore } from '@/stores/tabs';

import type { Tab } from '@/types';

import { LayoutGrid, Plus, Trash2 } from 'lucide-vue-next';
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
  <Dialog
    :visible="tabsStore.showTabSelector"
    modal
    :closable="true"
    :style="{ width: '30rem' }"
    @update:visible="
      (val) => {
        if (!val) close();
      }
    "
  >
    <template #header>
      <div class="flex items-center gap-2">
        <div
          class="h-2 w-2 rounded-full"
          :class="connection?.color ? 'bg-conn-' + connection.color : 'bg-primary'"
        ></div>
        <span class="text-[15px] font-semibold"> Open Editor: {{ connection?.name }} </span>
      </div>
    </template>

    <!-- List -->
    <div class="max-h-[50vh] space-y-1 overflow-y-auto py-1">
      <div
        v-for="tab in connectionTabs"
        :key="tab.id"
        class="group hover:bg-hover hover:border-border flex cursor-pointer items-center gap-3 rounded-lg border border-transparent p-2.5 transition-colors"
        @click="selectTab(tab.id)"
      >
        <div class="bg-muted text-primary group-hover:bg-surface rounded-md p-2 transition-colors">
          <LayoutGrid :size="16" />
        </div>
        <div class="min-w-0 flex-1">
          <div class="text-text-primary truncate text-[13px] font-medium">{{ tab.title }}</div>
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
    <template #footer>
      <div class="flex w-full items-center justify-between pt-2">
        <span class="text-text-tertiary text-[12px]">
          {{ connectionTabs.length }} editors found
        </span>
        <Button size="small" @click="createNew">
          <Plus class="h-4 w-4" />
          New
        </Button>
      </div>
    </template>
  </Dialog>
</template>
