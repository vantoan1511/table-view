<script setup lang="ts">
import ContextMenu from '@/components/ui/ContextMenu.vue';

import { useAboutStore } from '@/stores/about';
import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useLayoutStore } from '@/stores/layout';
import { usePreferencesStore } from '@/stores/preferences';
import { useToastStore } from '@/stores/toast';
import { useUpdaterStore } from '@/stores/updater';

import {
  Info,
  Loader2,
  Moon,
  PanelBottom,
  PanelLeft,
  PanelRight,
  RefreshCw,
  Settings,
  Sun
} from 'lucide-vue-next';
import { computed, ref } from 'vue';

const aboutStore = useAboutStore();
const preferencesStore = usePreferencesStore();
const layoutStore = useLayoutStore();
const connectionsStore = useConnectionsStore();
const gridStore = useGridStore();
const updaterStore = useUpdaterStore();
const toastStore = useToastStore();

const isDark = computed(() => {
  if (preferencesStore.settings.theme === 'system') {
    return document.documentElement.classList.contains('dark');
  }
  return preferencesStore.settings.theme === 'dark';
});

const showSettingsMenu = ref(false);
const settingsMenuPos = ref({ x: 0, y: 0 });

const toggleSettingsMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  // Position above the status bar
  settingsMenuPos.value = { x: rect.left, y: rect.top - 8 };
  showSettingsMenu.value = !showSettingsMenu.value;
};

// ─── Row Count ────────────────────────────────────────────────────────────
// Show the total rows from the active table grid, or SQL row count if a query was run
const displayRowCount = computed(() => {
  if (gridStore.sqlRowCount > 0 && gridStore.activeTableName === '') {
    return gridStore.sqlRowCount;
  }
  return gridStore.totalRows;
});

const displayExecTime = computed(() => {
  if (gridStore.sqlExecutionTime > 0 && gridStore.activeTableName === '') {
    return gridStore.sqlExecutionTime;
  }
  return gridStore.executionTime;
});

const toggleDarkMode = async () => {
  const currentTheme = preferencesStore.settings.theme;
  let nextTheme: 'light' | 'dark';
  if (currentTheme === 'system') {
    const systemIsDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    nextTheme = systemIsDark ? 'light' : 'dark';
  } else {
    nextTheme = currentTheme === 'dark' ? 'light' : 'dark';
  }
  await preferencesStore.save({ theme: nextTheme });
};

const handleCheckForUpdates = async () => {
  await updaterStore.checkForUpdates(true);
};

const openSettings = () => {
  preferencesStore.open();
};
</script>

<template>
  <footer
    class="bg-surface border-border text-text-secondary flex h-(--statusbar-height) shrink-0 items-center border-t px-3 text-[11px]"
  >
    <!-- Left: icons -->
    <div class="flex items-center gap-2">
      <button
        class="hover:bg-hover text-text-tertiary hover:text-text-secondary flex h-6 w-6 cursor-pointer items-center justify-center rounded transition-colors"
        @click.stop="toggleSettingsMenu"
        title="Settings & About"
      >
        <Settings :size="13" />
      </button>

      <ContextMenu
        :show="showSettingsMenu"
        :x="settingsMenuPos.x"
        :y="settingsMenuPos.y"
        @close="showSettingsMenu = false"
        width-class="w-48 !-translate-y-full"
      >
        <div class="text-text-tertiary px-3 py-1.5 text-[10px] font-bold tracking-wider uppercase">
          Application
        </div>
        <button
          @click="
            openSettings();
            showSettingsMenu = false;
          "
          class="hover:bg-hover text-text-primary flex w-full items-center gap-2 px-3 py-2 text-left transition-colors"
        >
          <Settings :size="13" class="text-text-tertiary" /> Preferences
        </button>
        <button
          @click="
            aboutStore.open();
            showSettingsMenu = false;
          "
          class="hover:bg-hover text-text-primary flex w-full items-center gap-2 px-3 py-2 text-left transition-colors"
        >
          <Info :size="13" class="text-text-tertiary" /> About Table View
        </button>

        <div class="bg-border mx-2 my-1.5 h-px" />
        <div class="text-text-tertiary px-3 py-1.5 text-[10px] font-bold tracking-wider uppercase">
          Appearance
        </div>
        <button
          @click="
            toggleDarkMode();
            showSettingsMenu = false;
          "
          class="hover:bg-hover text-text-primary flex w-full items-center gap-2 px-3 py-2 text-left transition-colors"
        >
          <Sun v-if="isDark" :size="13" class="text-text-tertiary" />
          <Moon v-else :size="13" class="text-text-tertiary" />
          {{ isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode' }}
        </button>

        <div class="bg-border mx-2 my-1.5 h-px" />
        <button
          @click="
            handleCheckForUpdates();
            showSettingsMenu = false;
          "
          class="hover:bg-hover text-text-primary flex w-full items-center gap-2 px-3 py-2 text-left transition-colors"
        >
          <RefreshCw
            :size="13"
            class="text-text-tertiary"
            :class="{ 'animate-spin': updaterStore.isChecking }"
          />
          Check for Updates
        </button>
      </ContextMenu>
    </div>

    <!-- Center: connection info -->
    <div class="ml-4 flex flex-1 items-center gap-3">
      <template v-if="connectionsStore.connectingConnectionId">
        <span class="flex items-center gap-1.5">
          <Loader2 :size="12" class="text-primary animate-spin" />
          <span class="text-primary font-medium">Connecting...</span>
        </span>
      </template>
      <template v-else-if="connectionsStore.activeConnection?.isConnected">
        <span class="flex items-center gap-1.5">
          <span
            class="bg-success h-2.5 w-2.5 rounded-full shadow-[0_0_8px_rgba(34,197,94,0.4)]"
          ></span>
          <span class="text-text-primary font-medium">Connected</span>
        </span>
        <span class="text-border-strong opacity-30">│</span>
        <span>{{ connectionsStore.activeConnection.type.toUpperCase() }}</span>
        <span class="text-border-strong opacity-30">│</span>
        <span
          >{{ connectionsStore.activeConnection.host }}:{{
            connectionsStore.activeConnection.port
          }}</span
        >
        <span class="text-border-strong opacity-30">│</span>
        <span>{{ gridStore.activeTableSchema || 'default' }}</span>
      </template>
      <template v-else>
        <span class="flex items-center gap-1.5">
          <span class="bg-text-tertiary/30 h-2.5 w-2.5 rounded-full"></span>
          <span>Not connected</span>
        </span>
      </template>
    </div>

    <!-- Right: query stats -->
    <div
      class="flex shrink-0 items-center gap-3"
      v-if="connectionsStore.activeConnection?.isConnected"
    >
      <span class="text-text-tertiary flex items-center gap-1">
        <Loader2 v-if="gridStore.isLoading" :size="11" class="text-primary animate-spin" />
        <template v-else>
          <svg
            class="text-primary h-3 w-3 opacity-70"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="10" />
            <polyline points="12 6 12 12 16 14" />
          </svg>
          {{ displayExecTime }} ms
        </template>
      </span>
      <span class="text-text-tertiary flex items-center gap-1">
        <svg
          class="text-primary h-3 w-3 opacity-70"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
        {{ displayRowCount.toLocaleString() }} rows
      </span>
    </div>

    <!-- Right: Panel Toggles -->
    <div class="border-border ml-3 flex shrink-0 items-center gap-1.5 border-l pl-3">
      <button
        v-tooltip.left="'Toggle Sidebar (Ctrl+B)'"
        class="hover:bg-hover flex h-6 w-6 cursor-pointer items-center justify-center rounded transition-colors"
        :class="
          layoutStore.isSidebarVisible
            ? 'text-primary bg-primary/10'
            : 'text-text-tertiary hover:text-text-secondary'
        "
        @click="layoutStore.toggleSidebar()"
      >
        <PanelLeft :size="14" />
      </button>
      <button
        v-tooltip.left="'Toggle Console (Ctrl+J)'"
        class="hover:bg-hover flex h-6 w-6 cursor-pointer items-center justify-center rounded transition-colors"
        :class="
          layoutStore.isBottomVisible
            ? 'text-primary bg-primary/10'
            : 'text-text-tertiary hover:text-text-secondary'
        "
        @click="layoutStore.togglePanel('console')"
      >
        <PanelBottom :size="14" />
      </button>
      <button
        v-tooltip.top="'Toggle Inspector (Ctrl+I)'"
        class="hover:bg-hover flex h-6 w-6 cursor-pointer items-center justify-center rounded transition-colors"
        :class="
          layoutStore.isRightVisible
            ? 'text-primary bg-primary/10'
            : 'text-text-tertiary hover:text-text-secondary'
        "
        @click="layoutStore.togglePanel('inspector')"
      >
        <PanelRight :size="14" />
      </button>
    </div>
  </footer>
</template>
