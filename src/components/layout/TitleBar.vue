<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections'
import { useTabsStore } from '@/stores/tabs'
import {
  Plus,
  Search,
  LayoutGrid,
  X,
  Code2,
  Home,
  MoreVertical,
} from 'lucide-vue-next'

const connectionsStore = useConnectionsStore()
const tabsStore = useTabsStore()

function getTabIcon(type: string) {
  switch (type) {
    case 'table': return LayoutGrid
    case 'sql': return Code2
    default: return Home
  }
}
</script>

<template>
  <header class="flex items-center h-[var(--titlebar-height)] bg-surface border-b border-border px-3 gap-2 shrink-0">
    <!-- New Tab Button -->
    <button
      id="btn-new-tab"
      class="flex items-center justify-center w-8 h-8 rounded-md text-text-secondary hover:bg-hover hover:text-text-primary"
      title="New Tab"
      @click="tabsStore.openSqlEditor()"
    >
      <Plus :size="18" />
    </button>

    <!-- Tab Strip -->
    <nav class="flex items-center gap-0.5 flex-1 min-w-0 overflow-x-auto px-1" id="tab-strip">
      <button
        v-for="tab in tabsStore.tabs"
        :key="tab.id"
        class="group flex items-center gap-1.5 px-3 h-[var(--tab-height)] rounded-lg text-[13px] whitespace-nowrap shrink-0 cursor-pointer transition-all duration-150"
        :class="
          tabsStore.activeTabId === tab.id
            ? 'bg-active text-primary font-medium'
            : 'text-text-secondary hover:bg-hover hover:text-text-primary'
        "
        @click="tabsStore.setActiveTab(tab.id)"
      >
        <component :is="getTabIcon(tab.type)" :size="14" class="shrink-0" />
        <span class="truncate max-w-[120px]">{{ tab.title }}</span>
        <span
          class="flex items-center justify-center w-4 h-4 rounded opacity-0 group-hover:opacity-100 hover:bg-border-strong transition-opacity"
          @click.stop="tabsStore.closeTab(tab.id)"
        >
          <X :size="12" />
        </span>
      </button>

      <!-- Add Tab -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md text-text-tertiary hover:bg-hover hover:text-text-secondary shrink-0"
        @click="tabsStore.openSqlEditor()"
      >
        <Plus :size="14" />
      </button>
    </nav>

    <!-- Right Actions -->
    <div class="flex items-center gap-2 shrink-0">
      <!-- Search -->
      <div class="flex items-center gap-1.5 px-2.5 py-1.5 bg-muted border border-border rounded-lg text-text-tertiary text-[12px]">
        <Search :size="13" />
        <span>Search</span>
        <kbd class="ml-1 px-1 py-0.5 bg-surface border border-border rounded text-[10px] font-mono">⌘K</kbd>
      </div>

      <!-- New Connection Button -->
      <button
        id="btn-new-connection"
        class="flex items-center gap-1.5 px-3.5 py-1.5 bg-primary hover:bg-primary-hover text-text-inverse rounded-lg text-[12px] font-medium cursor-pointer transition-colors shadow-sm"
        @click="connectionsStore.toggleConnectionModal(true)"
      >
        New Connection
      </button>

      <!-- More -->
      <button class="flex items-center justify-center w-7 h-7 rounded-md text-text-secondary hover:bg-hover">
        <MoreVertical :size="16" />
      </button>
    </div>
  </header>
</template>
