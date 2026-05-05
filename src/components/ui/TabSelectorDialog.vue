<script setup lang="ts">
import { useTabsStore } from '@/stores/tabs'
import { useConnectionsStore } from '@/stores/connections'
import { computed } from 'vue'
import type { Tab } from '@/types'
import { LayoutGrid, Plus, X, Trash2 } from 'lucide-vue-next'

const tabsStore = useTabsStore()
const connectionsStore = useConnectionsStore()

const connection = computed(() => 
  connectionsStore.connections.find(c => c.id === tabsStore.selectorConnectionId)
)

const connectionTabs = computed(() => 
  tabsStore.tabs.filter((t: Tab) => t.type === 'sql' && t.connectionId === tabsStore.selectorConnectionId)
)

const close = () => {
  tabsStore.showTabSelector = false
  tabsStore.selectorConnectionId = null
}

const selectTab = (id: string) => {
  const tab = tabsStore.tabs.find((t: Tab) => t.id === id)
  if (tab) {
    tab.closed = false
    tab.minimized = false
    tabsStore.setActiveTab(id)
  }
  close()
}

const createNew = () => {
  if (tabsStore.selectorConnectionId) {
    tabsStore.openSqlEditor(tabsStore.selectorConnectionId, '', true, true)
  }
  close()
}

const deleteTab = (id: string) => {
  tabsStore.deleteTab(id)
}
</script>

<template>
  <div v-if="tabsStore.showTabSelector" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm" @click.self="close">
    <div class="bg-surface border border-border rounded-xl shadow-2xl w-full max-w-md overflow-hidden flex flex-col max-h-[80vh]">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-border bg-muted">
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full" :class="'bg-' + (connection?.color || 'primary')"></div>
          <h3 class="text-[15px] font-semibold text-text-primary">
            Open Editor: {{ connection?.name }}
          </h3>
        </div>
        <button class="p-1 hover:bg-hover rounded-md text-text-tertiary transition-colors" @click="close">
          <X :size="18" />
        </button>
      </div>

      <!-- List -->
      <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
        <div v-for="tab in connectionTabs" :key="tab.id" 
          class="group flex items-center gap-3 p-3 rounded-lg hover:bg-hover cursor-pointer transition-colors border border-transparent hover:border-border"
          @click="selectTab(tab.id)">
          <div class="p-2 bg-muted rounded-md text-primary group-hover:bg-surface transition-colors">
            <LayoutGrid :size="16" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[14px] font-medium text-text-primary truncate">{{ tab.title }}</div>
            <div class="text-[11px] text-text-tertiary truncate">
              {{ tab.filePath || (tab.isDraft ? 'Draft' : 'Autosaved') }}
              <span v-if="tab.closed" class="ml-2 text-warning">• Closed</span>
            </div>
          </div>
          <button class="p-2 opacity-0 group-hover:opacity-100 hover:text-danger transition-all rounded-md hover:bg-danger/10" @click.stop="deleteTab(tab.id)" title="Delete permanently">
            <Trash2 :size="14" />
          </button>
        </div>

        <div v-if="connectionTabs.length === 0" class="py-8 text-center text-text-tertiary italic text-[13px]">
          No editors found for this connection.
        </div>
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-border bg-muted flex justify-between items-center">
        <span class="text-[12px] text-text-tertiary">{{ connectionTabs.length }} editors found</span>
        <button class="flex items-center gap-1.5 px-4 py-2 bg-primary hover:bg-primary-hover text-text-inverse rounded-lg text-[13px] font-medium transition-colors shadow-sm" @click="createNew">
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
