<script setup lang="ts">
import DataGrid from '@/components/grid/DataGrid.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import StatusBar from '@/components/layout/StatusBar.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import NewConnectionModal from '@/components/modals/NewConnectionModal.vue'
import SqlEditor from '@/components/sql/SqlEditor.vue'
import GlobalErrorDialog from '@/components/ui/GlobalErrorDialog.vue'
import ToastContainer from '@/components/ui/ToastContainer.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useGridStore } from '@/stores/grid'
import { useTabsStore } from '@/stores/tabs'
import { onMounted, watch } from 'vue'

const tabsStore = useTabsStore()
const gridStore = useGridStore()
const connectionsStore = useConnectionsStore()

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
      <!-- Sidebar (now spans full height above status bar) -->
      <Sidebar />

      <!-- Content Area (Tabs + Main View) -->
      <div class="flex flex-col flex-1 min-w-0 min-h-0">
        <!-- Title Bar (Tabs) is now only above the main content -->
        <TitleBar />

        <!-- Main View -->
        <main class="flex flex-col flex-1 min-w-0 min-h-0">
          <!-- Table tab active: show DataGrid on top -->
          <template v-if="tabsStore.activeTab?.type === 'table'">
            <DataGrid />
          </template>

          <!-- No tab: empty state -->
          <template v-else-if="!tabsStore.activeTab">
            <div class="flex flex-col items-center justify-center flex-1 text-text-tertiary">
              <svg class="w-16 h-16 mb-4 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="1">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                <line x1="3" y1="9" x2="21" y2="9" />
                <line x1="9" y1="21" x2="9" y2="9" />
              </svg>
              <p class="text-[14px] font-medium">No tab open</p>
              <p class="text-[12px] mt-1">Select a table from the sidebar or open a SQL editor</p>
            </div>
          </template>

          <!-- SQL Editor — always at the bottom as a persistent panel -->
          <SqlEditor />
        </main>
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
