<script setup lang="ts">
import MinimizedDock from '@/components/layout/MinimizedDock.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import StatusBar from '@/components/layout/StatusBar.vue'
import TabContent from '@/components/layout/TabContent.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import WorkspaceContainer from '@/components/layout/WorkspaceContainer.vue'
import ResizeHandle from '@/components/ui/ResizeHandle.vue'

import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
import { useConnectionsStore } from '@/stores/connections'
import { useGridStore } from '@/stores/grid'
import { useLayoutStore } from '@/stores/layout'
import { useSchemaStore } from '@/stores/schema'
import { useTabsStore } from '@/stores/tabs'
import { useUpdaterStore } from '@/stores/updater'
import * as Neutralino from '@neutralinojs/lib'
import { defineAsyncComponent, onMounted, watch } from 'vue'

// Lazy load secondary components
const NewConnectionModal = defineAsyncComponent(() => import('@/components/modals/NewConnectionModal.vue'))
const GlobalErrorDialog = defineAsyncComponent(() => import('@/components/ui/GlobalErrorDialog.vue'))
const ToastContainer = defineAsyncComponent(() => import('@/components/ui/ToastContainer.vue'))
const UpdaterDialog = defineAsyncComponent(() => import('@/components/ui/UpdaterDialog.vue'))
const AboutDialog = defineAsyncComponent(() => import('@/components/ui/AboutDialog.vue'))
const TabSelectorDialog = defineAsyncComponent(() => import('@/components/ui/TabSelectorDialog.vue'))

const tabsStore = useTabsStore()
const gridStore = useGridStore()
const connectionsStore = useConnectionsStore()
const layoutStore = useLayoutStore()
const schemaStore = useSchemaStore()
const updaterStore = useUpdaterStore()

useKeyboardShortcuts()


// When active tab changes, sync connection and load table data
watch(
  () => [tabsStore.activeTab, connectionsStore.connections.length] as const,
  async ([tab, connCount]) => {
    if (!tab || connCount === 0) return

    // Sync UI to tab's connection context
    if (tab.connectionId) {
      const conn = connectionsStore.connections.find(c => c.id === tab.connectionId)

      // Ensure we are connected to this database in the bridge
      if (!conn?.isConnected || tab.connectionId !== connectionsStore.activeConnectionId) {
        try {
          await connectionsStore.setActiveConnection(tab.connectionId)
        } catch (err) {
          console.error("Failed to sync connection for tab:", err)
          return
        }
      }

      // Sync schema selection for this specific connection
      if (tab.schema) {
        const currentSelected = schemaStore.selectedSchemaByConnection[tab.connectionId]
        if (tab.schema !== currentSelected) {
          schemaStore.setSelectedSchema(tab.schema, tab.connectionId)
          if (conn?.type === 'oracle') {
            await schemaStore.loadSchema(schemaStore.loadedAllDatabases, tab.connectionId, tab.schema)
          }
        }
      }
    }

    if (tab.type === 'table' && tab.tableName) {
      gridStore.isLoading = true
      gridStore.loadTable(tab.tableName, tab.connectionId, tab.schema, tab.dbName)
    }
  },
  { immediate: true },
)

onMounted(async () => {
  // Load initial data
  await connectionsStore.loadConnections()
  await layoutStore.init()

  // Initialize background services
  if (window.NL_PORT) {
    updaterStore.init()
    // Check for updates after 5 seconds to not block startup
    setTimeout(() => {
      updaterStore.checkForUpdates()
    }, 5000)
  }

  // Remove splash screen with a smooth fade-out
  const loader = document.getElementById('app-loader')
  if (loader) {
    loader.classList.add('fade-out')
    setTimeout(() => loader.remove(), 600)
  }

  // Load persisted tabs
  tabsStore.loadTabsFromStorage()

  // Handle window close
  if (window.NL_PORT) {
    Neutralino.events.on('windowClose', async () => {
      // Always exit immediately now, state is auto-persisted via watch
      exitApp()
    })
  }
})

const exitApp = async () => {
  if (window.NL_PORT) {
    try {
      await Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.shutdown', {})
    } catch (e) {
      console.error('Failed to send shutdown signal:', e)
    }
    Neutralino.app.exit()
  }
}
</script>

<template>
  <div id="app-shell" class="flex flex-col h-screen overflow-hidden bg-surface"
    :style="{ '--sidebar-width': `${layoutStore.sidebarWidth}px` }" @contextmenu.prevent>
    <!-- Main Top Area -->
    <div class="flex flex-1 min-h-0">
      <!-- Sidebar -->
      <Sidebar v-if="layoutStore.isSidebarVisible" />

      <!-- Sidebar Resize Handle -->
      <ResizeHandle v-if="layoutStore.isSidebarVisible" orientation="horizontal" :model-value="layoutStore.sidebarWidth"
        @update:model-value="layoutStore.setSidebarWidth($event)" />

      <!-- Content Area (Tabs + Main View) -->
      <div class="flex flex-col flex-1 min-w-0 min-h-0 relative">
        <!-- Title Bar (Tabs) -->
        <TitleBar />

        <!-- Workspace Container (Main + Panels) -->
        <WorkspaceContainer>
          <template #main>
            <!-- Primary Editor / Grid -->
            <TabContent :tab="tabsStore.activeTab" />

            <!-- Minimized Tabs Dock (Existing system) -->
            <MinimizedDock />
          </template>
        </WorkspaceContainer>

      </div>
    </div>

    <!-- Status Bar -->
    <StatusBar />

    <!-- Modals -->
    <NewConnectionModal />
    <GlobalErrorDialog />
    <ToastContainer />
    <UpdaterDialog />
    <AboutDialog />
    <TabSelectorDialog />
  </div>
</template>
