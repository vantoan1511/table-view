<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useConnectionsStore } from '@/stores/connections'
import { useTabsStore } from '@/stores/tabs'
import {
  Plus,
  Search,
  LayoutGrid,
  X,
  Home,
  MoreVertical,
} from 'lucide-vue-next'

const connectionsStore = useConnectionsStore()
const tabsStore = useTabsStore()

// ─── Context Menu ────────────────────────────────────────────────────────────
const showMenu = ref(false)
const menuPos = ref({ x: 0, y: 0 })
const menuTargetTabId = ref<string | null>(null)

function onContextMenu(e: MouseEvent, tabId: string) {
  e.preventDefault()
  showMenu.value = true
  menuPos.value = { x: e.clientX, y: e.clientY }
  menuTargetTabId.value = tabId
}

function closeMenu() {
  showMenu.value = false
}

function closeTab() {
  if (menuTargetTabId.value) tabsStore.closeTab(menuTargetTabId.value)
  closeMenu()
}

function closeOthers() {
  if (menuTargetTabId.value) {
    const toClose = tabsStore.tabs.filter(t => t.id !== menuTargetTabId.value).map(t => t.id)
    toClose.forEach(id => tabsStore.closeTab(id))
  }
  closeMenu()
}

function closeAll() {
  const toClose = tabsStore.tabs.map(t => t.id)
  toClose.forEach(id => tabsStore.closeTab(id))
  closeMenu()
}

onMounted(() => {
  window.addEventListener('click', closeMenu)
})

onUnmounted(() => {
  window.removeEventListener('click', closeMenu)
})
</script>

<template>
  <header class="flex items-center h-[var(--titlebar-height)] bg-muted border-b border-border pl-2 pr-3 shrink-0">
    
    <div class="flex items-center h-full mr-2">
      <!-- Home Button -->
      <button class="flex items-center justify-center w-8 h-8 rounded-md text-text-secondary hover:bg-hover hover:text-text-primary" title="Home">
        <Home :size="16" />
      </button>
    </div>

    <!-- Tab Strip -->
    <nav class="flex items-end flex-1 min-w-0 overflow-hidden h-full" id="tab-strip">
      <button
        v-for="tab in tabsStore.tabs"
        :key="tab.id"
        class="group relative flex items-center gap-1.5 px-3 h-[calc(var(--titlebar-height)-8px)] min-w-[120px] max-w-[200px] rounded-t-lg text-[13px] whitespace-nowrap shrink-0 cursor-pointer transition-colors border border-transparent border-b-0"
        :class="
          tabsStore.activeTabId === tab.id
            ? 'bg-surface border-border z-10 font-medium text-text-primary after:absolute after:bottom-[-1px] after:left-0 after:right-0 after:h-[1px] after:bg-surface'
            : 'text-text-secondary hover:bg-hover/50 hover:text-text-primary'
        "
        @click="tabsStore.setActiveTab(tab.id)"
        @contextmenu.prevent="onContextMenu($event, tab.id)"
      >
        <LayoutGrid :size="14" class="shrink-0" :class="tabsStore.activeTabId === tab.id ? 'text-primary' : 'text-text-tertiary'" />
        <span class="truncate flex-1 text-left">{{ tab.title }}</span>
        <span
          class="flex items-center justify-center w-4 h-4 rounded opacity-0 group-hover:opacity-100 hover:bg-border transition-opacity"
          @click.stop="tabsStore.closeTab(tab.id)"
        >
          <X :size="12" />
        </span>
      </button>
      
      <button class="flex items-center justify-center w-8 h-[calc(var(--titlebar-height)-8px)] rounded-t-lg text-text-tertiary hover:bg-hover/50 hover:text-text-secondary shrink-0 mb-[1px]">
        <Plus :size="14" />
      </button>
    </nav>

    <!-- Right Actions -->
    <div class="flex items-center gap-2 shrink-0 ml-2">
      <!-- Search -->
      <div class="flex items-center gap-1.5 px-2.5 py-1.5 bg-surface border border-border rounded-lg text-text-tertiary text-[12px]">
        <Search :size="13" />
        <span>Search</span>
        <kbd class="ml-1 px-1 py-0.5 bg-muted border border-border rounded text-[10px] font-mono">⌘K</kbd>
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

    <!-- Context Menu Overlay -->
    <div
      v-if="showMenu"
      class="fixed z-[100] bg-surface border border-border rounded-lg shadow-lg py-1 min-w-[160px] text-[12px]"
      :style="{ left: menuPos.x + 'px', top: menuPos.y + 'px' }"
      @click.stop
    >
      <button class="w-full text-left px-3 py-1.5 hover:bg-hover text-text-primary flex items-center gap-2" @click="closeTab">
        <X :size="13" />
        Close Tab
      </button>
      <div class="h-px bg-border my-1"></div>
      <button class="w-full text-left px-3 py-1.5 hover:bg-hover text-text-primary" @click="closeOthers">
        Close Others
      </button>
      <button class="w-full text-left px-3 py-1.5 hover:bg-hover text-text-primary" @click="closeAll">
        Close All
      </button>
    </div>
  </header>
</template>
