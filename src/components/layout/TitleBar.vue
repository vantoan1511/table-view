<script setup lang="ts">
import { useAboutStore } from '@/stores/about'
import { useConnectionsStore } from '@/stores/connections'
import { useTabsStore } from '@/stores/tabs'
import * as Neutralino from '@neutralinojs/lib'
import {
  Home,
  Info,
  LayoutGrid,
  LogOut,
  Minus,
  MoreVertical,
  Plus,
  Search,
  Settings,
  X,
} from 'lucide-vue-next'
import { onMounted, onUnmounted, ref } from 'vue'

const aboutStore = useAboutStore()
const connectionsStore = useConnectionsStore()
const tabsStore = useTabsStore()

// ─── Search Ref ──────────────────────────────────────────────────────────────
const searchInputRef = ref<HTMLElement | null>(null)

// ─── Tab Context Menu ────────────────────────────────────────────────────────
const showMenu = ref(false)
const menuPos = ref({ x: 0, y: 0 })
const menuTargetTabId = ref<string | null>(null)

// ─── More Menu ───────────────────────────────────────────────────────────────
const showMoreMenu = ref(false)
const moreMenuPos = ref({ x: 0, y: 0 })

const onContextMenu = (e: MouseEvent, tabId: string) => {
  e.preventDefault()
  showMenu.value = true
  showMoreMenu.value = false
  menuPos.value = { x: e.clientX, y: e.clientY }
  menuTargetTabId.value = tabId
}

const closeMenu = () => {
  showMenu.value = false
  showMoreMenu.value = false
}

const closeTab = () => {
  if (menuTargetTabId.value) tabsStore.closeTab(menuTargetTabId.value)
  closeMenu()
}

const closeOthers = () => {
  if (menuTargetTabId.value) {
    const toClose = tabsStore.tabs.filter(t => t.id !== menuTargetTabId.value).map(t => t.id)
    toClose.forEach(id => tabsStore.closeTab(id))
  }
  closeMenu()
}

const closeAll = () => {
  const toClose = tabsStore.tabs.map(t => t.id)
  toClose.forEach(id => tabsStore.closeTab(id))
  closeMenu()
}

const openNewQueryConsole = () => {
  tabsStore.openSqlEditor()
}

const minimizeTab = () => {
  if (menuTargetTabId.value) tabsStore.minimizeTab(menuTargetTabId.value)
  closeMenu()
}

const onDragStart = (event: DragEvent, tabId: string) => {
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', tabId)
  }
  tabsStore.draggingTabId = tabId
}

const onDragEnd = () => {
  tabsStore.draggingTabId = null
}

// ─── Home Button ─────────────────────────────────────────────────────────────
const goHome = () => {
  // Open a home tab if it doesn't already exist
  const homeTab = tabsStore.tabs.find(t => t.type === 'home')
  if (homeTab) {
    tabsStore.setActiveTab(homeTab.id)
  } else if (tabsStore.tabs.length > 0) {
    // Fallback: navigate to the first tab
    tabsStore.setActiveTab(tabsStore.tabs[0]?.id || '')
  }
}

// ─── More Menu ───────────────────────────────────────────────────────────────
const toggleMoreMenu = (e: MouseEvent) => {
  e.stopPropagation()
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  moreMenuPos.value = { x: rect.right - 160, y: rect.bottom + 5 }
  showMoreMenu.value = !showMoreMenu.value
  showMenu.value = false
}

const openSettings = () => {
  // TODO: Open a settings modal/view
  console.log('Settings not yet implemented')
  closeMenu()
}

const showAbout = () => {
  aboutStore.open()
  closeMenu()
}

const quitApp = async () => {
  if (window.NL_PORT) {
    try {
      await Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.shutdown', {})
    } catch (e) {
      console.error('Failed to send shutdown signal:', e)
    }
    Neutralino.app.exit()
  }
  closeMenu()
}

onMounted(() => {
  window.addEventListener('click', closeMenu)
  // ⌘K / Ctrl+K to focus the search input
  window.addEventListener('keydown', (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault()
      searchInputRef.value?.focus()
    }
  })
})

onUnmounted(() => {
  window.removeEventListener('click', closeMenu)
})
</script>

<template>
  <header class="flex items-center h-(--titlebar-height) bg-muted border-b border-border pl-2 pr-3 shrink-0">

    <div class="flex items-center h-full mr-2">
      <!-- Home Button -->
      <button
        class="flex items-center justify-center w-8 h-8 rounded-md text-text-secondary hover:bg-hover hover:text-text-primary cursor-pointer transition-colors"
        title="Home" @click="goHome">
        <Home :size="16" />
      </button>
    </div>

    <!-- Tab Strip -->
    <nav class="flex items-end flex-1 min-w-0 overflow-hidden h-full" id="tab-strip">
      <button v-for="tab in tabsStore.mainTabs" :key="tab.id" draggable="true"
        class="group relative flex items-center gap-1.5 px-3 h-[calc(var(--titlebar-height)-8px)] min-w-[120px] max-w-[200px] rounded-t-lg text-[13px] whitespace-nowrap shrink-0 cursor-pointer transition-colors border border-transparent border-b-0"
        :class="tabsStore.activeTabId === tab.id
          ? 'bg-surface border-border z-10 font-medium text-text-primary after:absolute after:-bottom-px after:left-0 after:right-0 after:h-px after:bg-surface'
          : 'text-text-secondary hover:bg-hover/50 hover:text-text-primary'
          " @click="tabsStore.setActiveTab(tab.id)" @contextmenu.prevent="onContextMenu($event, tab.id)"
        @dragstart="onDragStart($event, tab.id)" @dragend="onDragEnd">
        <LayoutGrid :size="14" class="shrink-0"
          :class="tabsStore.activeTabId === tab.id ? 'text-primary' : 'text-text-tertiary'" />
        <span class="truncate flex-1 text-left">{{ tab.title }}</span>
        <span
          class="flex items-center justify-center w-4 h-4 rounded opacity-0 group-hover:opacity-100 hover:bg-border transition-opacity"
          @click.stop="tabsStore.closeTab(tab.id)">
          <X :size="12" />
        </span>
      </button>

      <button
        class="flex items-center justify-center w-8 h-[calc(var(--titlebar-height)-8px)] rounded-t-lg text-text-tertiary hover:bg-hover/50 hover:text-text-secondary shrink-0 mb-px cursor-pointer"
        @click="openNewQueryConsole">
        <Plus :size="14" />
      </button>
    </nav>

    <!-- Right Actions -->
    <div class="flex items-center gap-2 shrink-0 ml-2">
      <!-- Search -->
      <div
        class="flex items-center gap-1.5 px-2.5 py-1.5 bg-surface border border-border rounded-lg text-text-tertiary text-[12px] focus-within:border-primary/50 transition-colors">
        <Search :size="13" />
        <input ref="searchInputRef" type="text" placeholder="Search..."
          class="bg-transparent outline-none text-[12px] text-text-primary placeholder-text-tertiary w-20 focus:w-32 transition-all" />
        <kbd class="ml-1 px-1 py-0.5 bg-muted border border-border rounded text-[10px] font-mono">⌘K</kbd>
      </div>

      <!-- New Connection Button -->
      <button id="btn-new-connection"
        class="flex items-center gap-1.5 px-3.5 py-1.5 bg-primary hover:bg-primary-hover text-text-inverse rounded-lg text-[12px] font-medium cursor-pointer transition-colors shadow-sm"
        @click="connectionsStore.toggleConnectionModal(true)">
        New Connection
      </button>

      <!-- More -->
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md text-text-secondary hover:bg-hover cursor-pointer relative"
        @click.stop="toggleMoreMenu">
        <MoreVertical :size="16" />
      </button>
    </div>

    <!-- More Context Menu -->
    <div v-if="showMoreMenu"
      class="fixed z-100 bg-surface border border-border rounded-lg shadow-lg py-1 min-w-[160px] text-[12px]"
      :style="{ left: moreMenuPos.x + 'px', top: moreMenuPos.y + 'px' }" @click.stop>
      <button class="w-full text-left px-3 py-1.5 hover:bg-hover text-text-primary flex items-center gap-2"
        @click="openSettings">
        <Settings :size="13" />
        Settings
      </button>
      <button class="w-full text-left px-3 py-1.5 hover:bg-hover text-text-primary flex items-center gap-2"
        @click="showAbout">
        <Info :size="13" />
        About
      </button>
      <div class="h-px bg-border my-1" />
      <button class="w-full text-left px-3 py-1.5 hover:bg-hover text-danger flex items-center gap-2" @click="quitApp">
        <LogOut :size="13" />
        Quit
      </button>
    </div>

    <!-- Context Menu Overlay -->
    <div v-if="showMenu"
      class="fixed z-100 bg-surface border border-border rounded-lg shadow-lg py-1 min-w-[160px] text-[12px]"
      :style="{ left: menuPos.x + 'px', top: menuPos.y + 'px' }" @click.stop>
      <button class="w-full text-left px-3 py-1.5 hover:bg-hover text-text-primary flex items-center gap-2"
        @click="closeTab">
        <X :size="13" />
        Close Tab
      </button>
      <button class="w-full text-left px-3 py-1.5 hover:bg-hover text-text-primary flex items-center gap-2"
        @click="minimizeTab">
        <Minus :size="13" />
        Minimize
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
