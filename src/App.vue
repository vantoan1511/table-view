<script setup lang="ts">
import Sidebar from '@/components/layout/Sidebar.vue'
import StatusBar from '@/components/layout/StatusBar.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import MinimizedDock from '@/components/layout/MinimizedDock.vue'
import TabContent from '@/components/layout/TabContent.vue'
import WorkspaceContainer from '@/components/layout/WorkspaceContainer.vue'
import PanelRail from '@/components/layout/PanelRail.vue'
import ResizeHandle from '@/components/ui/ResizeHandle.vue'
import NewConnectionModal from '@/components/modals/NewConnectionModal.vue'
import GlobalErrorDialog from '@/components/ui/GlobalErrorDialog.vue'
import ToastContainer from '@/components/ui/ToastContainer.vue'
import UpdaterDialog from '@/components/ui/UpdaterDialog.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useGridStore } from '@/stores/grid'
import { useTabsStore } from '@/stores/tabs'
import { useLayoutStore } from '@/stores/layout'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
import { onMounted, watch } from 'vue'

const tabsStore = useTabsStore()
const gridStore = useGridStore()
const connectionsStore = useConnectionsStore()
const layoutStore = useLayoutStore()

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
})
</script>

<template>
  <div 
    id="app-shell" 
    class="flex flex-col h-screen overflow-hidden bg-surface" 
    :style="{ '--sidebar-width': `${layoutStore.sidebarWidth}px` }"
    @contextmenu.prevent
  >
    <!-- Main Top Area -->
    <div class="flex flex-1 min-h-0">
      <!-- Sidebar -->
      <Sidebar v-if="layoutStore.isSidebarVisible" />
      
      <!-- Sidebar Resize Handle -->
      <ResizeHandle
        v-if="layoutStore.isSidebarVisible"
        orientation="horizontal"
        :model-value="layoutStore.sidebarWidth"
        @update:model-value="layoutStore.setSidebarWidth($event)"
      />

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

      <!-- Minimized Panel Rail -->
      <PanelRail />
    </div>

    <!-- Status Bar -->
    <StatusBar />

    <!-- Modals -->
    <NewConnectionModal />
    <GlobalErrorDialog />
    <ToastContainer />
    <UpdaterDialog />
  </div>
</template>
