<script setup lang="ts">
import Sidebar from '@/components/layout/Sidebar.vue'
import StatusBar from '@/components/layout/StatusBar.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import MinimizedDock from '@/components/layout/MinimizedDock.vue'
import TabContent from '@/components/layout/TabContent.vue'
import SplitPanel from '@/components/ui/SplitPanel.vue'
import NewConnectionModal from '@/components/modals/NewConnectionModal.vue'
import GlobalErrorDialog from '@/components/ui/GlobalErrorDialog.vue'
import ToastContainer from '@/components/ui/ToastContainer.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useGridStore } from '@/stores/grid'
import { useTabsStore } from '@/stores/tabs'
import { onMounted, watch } from 'vue'

const tabsStore = useTabsStore()
const gridStore = useGridStore()
const connectionsStore = useConnectionsStore()

// ─── Drop Zone ────────────────────────────────────────────────────────────────
function onDrop() {
  if (tabsStore.draggingTabId) {
    tabsStore.moveTabToBottom(tabsStore.draggingTabId)
    tabsStore.draggingTabId = null
  }
}

// ─── Bottom Panel Actions ─────────────────────────────────────────────────────
function minimizeBottomPanel() {
  if (tabsStore.bottomTabId) {
    const id = tabsStore.bottomTabId
    tabsStore.closeBottomPanel()
    tabsStore.minimizeTab(id)
  }
}

// When active tab changes to a table tab, load its data
watch(
  () => tabsStore.activeTab,
  (tab) => {
    if (tab?.type === 'table' && tab.tableName) {
      gridStore.loadTable(tab.tableName)
    }
  },
  { immediate: true },
)

onMounted(async () => {
  await connectionsStore.loadConnections()

  // Global Keyboard Shortcuts
  window.addEventListener('keydown', (e) => {
    const isMod = e.ctrlKey || e.metaKey

    // Ctrl+W: Close active tab
    if (isMod && e.key === 'w') {
      e.preventDefault()
      if (tabsStore.activeTabId) {
        tabsStore.closeTab(tabsStore.activeTabId)
      }
    }

    // Ctrl+K: Focus search (placeholder for now)
    if (isMod && e.key === 'k') {
      e.preventDefault()
      // TODO: focus search input
    }

    // Ctrl+R: Refresh data
    if (isMod && e.key === 'r') {
      e.preventDefault()
      if (tabsStore.activeTab?.type === 'table') {
        gridStore.loadTable(tabsStore.activeTab.tableName!)
      }
    }
  })
})
</script>

<template>
  <div id="app-shell" class="flex flex-col h-screen overflow-hidden bg-surface" @contextmenu.prevent>
    <!-- Main Top Area -->
    <div class="flex flex-1 min-h-0">
      <!-- Sidebar (spans full height above status bar) -->
      <Sidebar />

      <!-- Content Area (Tabs + Main View) -->
      <div class="flex flex-col flex-1 min-w-0 min-h-0 relative">
        <!-- Title Bar (Tabs) -->
        <TitleBar />

        <!-- Main View (uses SplitPanel for split / single view) -->
        <SplitPanel
          :split="!!tabsStore.bottomTab"
          :split-ratio="tabsStore.splitRatio"
          :bottom-title="tabsStore.bottomTab?.title ?? ''"
          @update:split-ratio="tabsStore.splitRatio = $event"
          @move-to-top="tabsStore.moveBottomToTop"
          @minimize="minimizeBottomPanel"
          @close="tabsStore.closeBottomPanel"
        >
          <template #top>
            <TabContent :tab="tabsStore.activeTab" />
          </template>
          <template #bottom>
            <TabContent :tab="tabsStore.bottomTab" />
          </template>
        </SplitPanel>

        <!-- Minimized Tabs Dock -->
        <MinimizedDock />

        <!-- Drop Zone Overlay (scoped to content area, only visible when dragging) -->
        <div
          v-if="tabsStore.draggingTabId"
          class="absolute bottom-0 left-0 right-0 h-1/2 z-50 bg-primary/5 border-t-2 border-primary border-dashed flex items-center justify-center pointer-events-auto transition-all"
          @dragover.prevent
          @drop="onDrop"
        >
          <div class="bg-surface/80 backdrop-blur px-6 py-3 rounded-xl border border-primary/20 text-primary font-medium shadow-lg pointer-events-none">
            Drop here to open in split view
          </div>
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <StatusBar />

    <!-- Modals -->
    <NewConnectionModal />
    <GlobalErrorDialog />
    <ToastContainer />
  </div>
</template>
