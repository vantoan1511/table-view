<script setup lang="ts">
import Sidebar from '@/components/layout/Sidebar.vue'
import StatusBar from '@/components/layout/StatusBar.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import MinimizedDock from '@/components/layout/MinimizedDock.vue'
import TabContent from '@/components/layout/TabContent.vue'
import NewConnectionModal from '@/components/modals/NewConnectionModal.vue'
import GlobalErrorDialog from '@/components/ui/GlobalErrorDialog.vue'
import ToastContainer from '@/components/ui/ToastContainer.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useGridStore } from '@/stores/grid'
import { useTabsStore } from '@/stores/tabs'
import { ArrowUp, Minus, X } from 'lucide-vue-next'
import { onMounted, ref, watch } from 'vue'

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

function minimizeBottomPanel() {
  if (tabsStore.bottomTabId) {
    const id = tabsStore.bottomTabId
    tabsStore.closeBottomPanel()
    tabsStore.minimizeTab(id)
  }
}

// ─── Resizable Split Divider ──────────────────────────────────────────────────
const contentAreaRef = ref<HTMLElement | null>(null)

function onDividerMouseDown(e: MouseEvent) {
  e.preventDefault()
  const container = contentAreaRef.value
  if (!container) return

  const startY = e.clientY
  const startRatio = tabsStore.splitRatio
  const containerRect = container.getBoundingClientRect()

  const onMouseMove = (ev: MouseEvent) => {
    const deltaY = ev.clientY - startY
    const deltaPct = (deltaY / containerRect.height) * 100
    tabsStore.splitRatio = Math.max(20, Math.min(80, startRatio + deltaPct))
  }

  const onMouseUp = () => {
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
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
      <div ref="contentAreaRef" class="flex flex-col flex-1 min-w-0 min-h-0 relative">
        <!-- Title Bar (Tabs) -->
        <TitleBar />

        <!-- Main View (split or single) -->
        <template v-if="tabsStore.bottomTab">
          <!-- Split View: Top Panel -->
          <main class="flex flex-col min-w-0 min-h-0 overflow-hidden" :style="{ flex: `0 0 ${tabsStore.splitRatio}%` }">
            <TabContent :tab="tabsStore.activeTab" />
          </main>

          <!-- Resize Divider -->
          <div
            class="h-[5px] bg-border hover:bg-primary/40 cursor-row-resize transition-colors shrink-0 flex items-center justify-center group"
            @mousedown="onDividerMouseDown"
          >
            <div class="w-8 h-[3px] rounded bg-text-tertiary/30 group-hover:bg-primary/60 transition-colors"></div>
          </div>

          <!-- Split View: Bottom Panel -->
          <div class="flex flex-col min-w-0 min-h-0 flex-1 overflow-hidden">
            <!-- Bottom panel header -->
            <div class="flex items-center gap-2 px-3 py-1 bg-muted border-b border-border shrink-0">
              <span class="text-[12px] font-medium text-text-primary truncate">{{ tabsStore.bottomTab.title }}</span>
              <div class="flex items-center gap-0.5 ml-auto">
                <button
                  class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-text-primary hover:bg-hover transition-colors cursor-pointer"
                  title="Move to top"
                  @click="tabsStore.moveBottomToTop"
                >
                  <ArrowUp :size="12" />
                </button>
                <button
                  class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-text-primary hover:bg-hover transition-colors cursor-pointer"
                  title="Minimize"
                  @click="minimizeBottomPanel"
                >
                  <Minus :size="12" />
                </button>
                <button
                  class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-danger hover:bg-danger-light transition-colors cursor-pointer"
                  title="Close panel"
                  @click="tabsStore.closeBottomPanel"
                >
                  <X :size="12" />
                </button>
              </div>
            </div>
            <TabContent :tab="tabsStore.bottomTab" />
          </div>
        </template>

        <!-- Single View (no split) -->
        <template v-else>
          <main class="flex flex-col flex-1 min-w-0 min-h-0">
            <TabContent :tab="tabsStore.activeTab" />
          </main>
        </template>

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
