<script setup lang="ts">
import MinimizedDock from '@/components/layout/MinimizedDock.vue';
import Sidebar from '@/components/layout/Sidebar.vue';
import StatusBar from '@/components/layout/StatusBar.vue';
import TabContent from '@/components/layout/TabContent.vue';
import TitleBar from '@/components/layout/TitleBar.vue';
import WorkspaceContainer from '@/components/layout/WorkspaceContainer.vue';
import ResizeHandle from '@/components/ui/ResizeHandle.vue';

import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useTabSync } from '@/composables/useTabSync';
import { useConnectionsStore } from '@/stores/connections';
import { useLayoutStore } from '@/stores/layout';
import { useTabsStore } from '@/stores/tabs';
import { useUpdaterStore } from '@/stores/updater';
import * as Neutralino from '@neutralinojs/lib';
import { defineAsyncComponent, onMounted } from 'vue';

// Lazy load secondary components
const NewConnectionModal = defineAsyncComponent(
  () => import('@/components/modals/NewConnectionModal.vue')
);
const GlobalErrorDialog = defineAsyncComponent(
  () => import('@/components/ui/GlobalErrorDialog.vue')
);
const ToastContainer = defineAsyncComponent(() => import('@/components/ui/ToastContainer.vue'));
const UpdaterDialog = defineAsyncComponent(() => import('@/components/ui/UpdaterDialog.vue'));
const AboutDialog = defineAsyncComponent(() => import('@/components/ui/AboutDialog.vue'));
const TabSelectorDialog = defineAsyncComponent(
  () => import('@/components/ui/TabSelectorDialog.vue')
);

const tabsStore = useTabsStore();
const connectionsStore = useConnectionsStore();
const layoutStore = useLayoutStore();
const updaterStore = useUpdaterStore();

useKeyboardShortcuts();
useTabSync();

onMounted(async () => {
  // Load initial data and persisted state in parallel to optimize startup time
  await Promise.all([
    connectionsStore.loadConnections(),
    layoutStore.init(),
    tabsStore.loadTabsFromStorage()
  ]);

  // Initialize background services
  if (window.NL_PORT) {
    updaterStore.init();
    // Check for updates after 5 seconds to not block startup
    setTimeout(() => {
      updaterStore.checkForUpdates();
    }, 5000);
  }

  // Remove splash screen with a smooth fade-out
  const loader = document.getElementById('app-loader');
  if (loader) {
    loader.classList.add('fade-out');
    setTimeout(() => loader.remove(), 600);
  }

  // Handle window close
  if (window.NL_PORT) {
    Neutralino.events.on('windowClose', async () => {
      // Always exit immediately now, state is auto-persisted via watch
      exitApp();
    });
  }
});

const exitApp = async () => {
  if (window.NL_PORT) {
    try {
      await Neutralino.extensions.dispatch(
        'com.github.vantoan1511.tableview.db-bridge',
        'dbBridge.shutdown',
        {}
      );
    } catch (e) {
      console.error('Failed to send shutdown signal:', e);
    }
    Neutralino.app.exit();
  }
};
</script>

<template>
  <div
    id="app-shell"
    class="bg-surface flex h-screen flex-col overflow-hidden"
    :style="{ '--sidebar-width': `${layoutStore.sidebarWidth}px` }"
    @contextmenu.prevent
  >
    <!-- Main Top Area -->
    <div class="flex min-h-0 flex-1">
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
      <div class="relative flex min-h-0 min-w-0 flex-1 flex-col">
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
