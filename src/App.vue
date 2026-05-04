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
import { defineAsyncComponent, onMounted, watch } from 'vue'

// Lazy load secondary components
const NewConnectionModal = defineAsyncComponent(() => import('@/components/modals/NewConnectionModal.vue'))
const GlobalErrorDialog = defineAsyncComponent(() => import('@/components/ui/GlobalErrorDialog.vue'))
const ToastContainer = defineAsyncComponent(() => import('@/components/ui/ToastContainer.vue'))
const UpdaterDialog = defineAsyncComponent(() => import('@/components/ui/UpdaterDialog.vue'))
const AboutDialog = defineAsyncComponent(() => import('@/components/ui/AboutDialog.vue'))

const tabsStore = useTabsStore()
const gridStore = useGridStore()
const connectionsStore = useConnectionsStore()
const layoutStore = useLayoutStore()
const schemaStore = useSchemaStore()
const updaterStore = useUpdaterStore()

useKeyboardShortcuts()

// ─── Drop Zone ────────────────────────────────────────────────────────────────
const onDrop = () => {
  if (tabsStore.draggingTabId) {
    tabsStore.moveTabToBottom(tabsStore.draggingTabId)
    tabsStore.draggingTabId = null
  }
}

// ─── Bottom Panel Actions ─────────────────────────────────────────────────────
const minimizeBottomPanel = () => {
  if (tabsStore.bottomTabId) {
    const id = tabsStore.bottomTabId
    tabsStore.closeBottomPanel()
    tabsStore.minimizeTab(id)
  }
}

// When active tab changes, sync global connection/schema and load table data
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

      // Sync schema selection
      if (tab.schema && tab.schema !== schemaStore.selectedSchema) {
        schemaStore.setSelectedSchema(tab.schema)
        if (conn?.type === 'oracle') {
          await schemaStore.loadSchema(schemaStore.loadedAllSchemas, tab.connectionId, tab.schema)
        }
      }
    }

    if (tab.type === 'table' && tab.tableName) {
      // Trigger loading state early for better UX
      gridStore.isLoading = true
      // Explicitly pass connectionId to avoid using stale global state
      gridStore.loadTable(tab.tableName, tab.connectionId, tab.schema)
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
})
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

        <!-- Drop Zone Overlay (Existing system) -->
        <div v-if="tabsStore.draggingTabId"
          class="absolute bottom-0 left-0 right-0 h-1/2 z-50 bg-primary/5 border-t-2 border-primary border-dashed flex items-center justify-center pointer-events-auto transition-all"
          @dragover.prevent @drop="onDrop">
          <div
            class="bg-surface/80 backdrop-blur px-6 py-3 rounded-xl border border-primary/20 text-primary font-medium shadow-lg pointer-events-none">
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
    <UpdaterDialog />
    <AboutDialog />
  </div>
</template>
