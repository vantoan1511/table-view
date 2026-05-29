<script setup lang="ts">
import ContextMenu from '@/components/ui/ContextMenu.vue';
import { useAboutStore } from '@/stores/about';
import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useLayoutStore } from '@/stores/layout';
import { useToastStore } from '@/stores/toast';
import { useUpdaterStore } from '@/stores/updater';
import * as Neutralino from '@neutralinojs/lib';
import {
  Info,
  Loader2,
  Moon,
  PanelBottom,
  PanelRight,
  RefreshCw,
  Settings,
  Sun
} from 'lucide-vue-next';
import { computed, onMounted, ref } from 'vue';
import Tooltip from '../ui/Tooltip.vue';

const aboutStore = useAboutStore();
const layoutStore = useLayoutStore();
const connectionsStore = useConnectionsStore();
const gridStore = useGridStore();
const updaterStore = useUpdaterStore();
const toastStore = useToastStore();
const isDark = ref(false);

const showSettingsMenu = ref(false);
const settingsMenuPos = ref({ x: 0, y: 0 });

const toggleSettingsMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  // Position above the status bar
  settingsMenuPos.value = { x: rect.left, y: rect.top - 8 };
  showSettingsMenu.value = !showSettingsMenu.value;
};

onMounted(async () => {
  // Load saved theme preference
  if (window.NL_PORT) {
    try {
      const theme = await Neutralino.storage.getData('theme');
      if (theme === 'dark') {
        isDark.value = true;
        document.documentElement.classList.add('dark');
      }
    } catch {
      // No theme stored yet, default to light
    }
  }
});

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
  isDark.value = !isDark.value;
  document.documentElement.classList.toggle('dark', isDark.value);
  if (window.NL_PORT) {
    await Neutralino.storage.setData('theme', isDark.value ? 'dark' : 'light');
  }
};

const handleCheckForUpdates = async () => {
  const hadUpdate = !!updaterStore.updateAvailable;
  await updaterStore.checkForUpdates(true);
  // If no update was found after checking, show a friendly toast
  if (!updaterStore.updateAvailable && !hadUpdate) {
    toastStore.addToast({
      title: 'Up to date',
      message: `You are running the latest version.`,
      severity: 'success',
      variation: 'filled',
      position: 'bottom-center',
      ttl: 3000
    });
  }
};

const openSettings = () => {
  toastStore.addToast({
    title: 'Settings',
    message: 'Settings panel coming soon.',
    severity: 'info',
    variation: 'outlined',
    position: 'bottom-center'
  });
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
      <template v-if="connectionsStore.activeConnection?.isConnected">
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
      <Tooltip text="Toggle Console (Ctrl+J)" preferred-position="left">
        <button
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
      </Tooltip>
      <Tooltip text="Toggle Inspector (Ctrl+I)" preferred-position="top">
        <button
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
      </Tooltip>
    </div>
  </footer>
</template>
