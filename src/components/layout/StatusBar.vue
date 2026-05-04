<script setup lang="ts">
import { useAboutStore } from '@/stores/about'
import { useConnectionsStore } from '@/stores/connections'
import { useGridStore } from '@/stores/grid'
import { useToastStore } from '@/stores/toast'
import { useUpdaterStore } from '@/stores/updater'
import * as Neutralino from '@neutralinojs/lib'
import {
  ArrowUpCircle,
  Info,
  Loader2,
  Moon,
  PanelBottom,
  PanelRight,
  RefreshCw,
  Settings,
  Sun
} from 'lucide-vue-next'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { useLayoutStore } from '@/stores/layout'
import { computed, onMounted, ref } from 'vue'

const aboutStore = useAboutStore()
const layoutStore = useLayoutStore()
const connectionsStore = useConnectionsStore()
const gridStore = useGridStore()
const updaterStore = useUpdaterStore()
const toastStore = useToastStore()
const isDark = ref(false)

const showSettingsMenu = ref(false)
const settingsMenuPos = ref({ x: 0, y: 0 })

const toggleSettingsMenu = (e: MouseEvent) => {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  // Position above the status bar
  settingsMenuPos.value = { x: rect.left, y: rect.top - 8 }
  showSettingsMenu.value = !showSettingsMenu.value
}

onMounted(async () => {
  // Load saved theme preference
  if (window.NL_PORT) {
    try {
      const theme = await Neutralino.storage.getData('theme')
      if (theme === 'dark') {
        isDark.value = true
        document.documentElement.classList.add('dark')
      }
    } catch {
      // No theme stored yet, default to light
    }
  }
})

// ─── Row Count ────────────────────────────────────────────────────────────
// Show the total rows from the active table grid, or SQL row count if a query was run
const displayRowCount = computed(() => {
  if (gridStore.sqlRowCount > 0 && gridStore.activeTableName === '') {
    return gridStore.sqlRowCount
  }
  return gridStore.totalRows
})

const displayExecTime = computed(() => {
  if (gridStore.sqlExecutionTime > 0 && gridStore.activeTableName === '') {
    return gridStore.sqlExecutionTime
  }
  return gridStore.executionTime
})

const toggleDarkMode = async () => {
  isDark.value = !isDark.value
  document.documentElement.classList.toggle('dark', isDark.value)
  if (window.NL_PORT) {
    await Neutralino.storage.setData('theme', isDark.value ? 'dark' : 'light')
  }
}

const handleCheckForUpdates = async () => {
  const hadUpdate = !!updaterStore.updateAvailable
  await updaterStore.checkForUpdates(true)
  // If no update was found after checking, show a friendly toast
  if (!updaterStore.updateAvailable && !hadUpdate) {
    toastStore.addToast({
      title: 'Up to date',
      message: `You are running the latest version.`,
      severity: 'success',
      variation: 'filled',
      position: 'bottom-center',
      ttl: 3000,
    })
  }
}

const openSettings = () => {
  toastStore.addToast({
    title: 'Settings',
    message: 'Settings panel coming soon.',
    severity: 'info',
    variation: 'outlined',
    position: 'bottom-center',
  })
}
</script>

<template>
  <footer
    class="flex items-center h-(--statusbar-height) bg-surface border-t border-border px-3 text-[11px] text-text-secondary shrink-0">
    <!-- Left: icons -->
    <div class="flex items-center gap-2">
      <button
        class="flex items-center justify-center w-6 h-6 rounded hover:bg-hover text-text-tertiary hover:text-text-secondary cursor-pointer transition-colors"
        @click.stop="toggleSettingsMenu" title="Settings & About">
        <Settings :size="13" />
      </button>

      <ContextMenu :show="showSettingsMenu" :x="settingsMenuPos.x" :y="settingsMenuPos.y"
        @close="showSettingsMenu = false" width-class="w-48 !-translate-y-full">
        <div class="px-3 py-1.5 text-[10px] text-text-tertiary uppercase font-bold tracking-wider">Application</div>
        <button @click="openSettings(); showSettingsMenu = false"
          class="w-full text-left px-3 py-2 hover:bg-hover text-text-primary flex items-center gap-2 transition-colors">
          <Settings :size="13" class="text-text-tertiary" /> Preferences
        </button>
        <button @click="aboutStore.open(); showSettingsMenu = false"
          class="w-full text-left px-3 py-2 hover:bg-hover text-text-primary flex items-center gap-2 transition-colors">
          <Info :size="13" class="text-text-tertiary" /> About Table View
        </button>

        <div class="h-px bg-border my-1.5 mx-2" />
        <div class="px-3 py-1.5 text-[10px] text-text-tertiary uppercase font-bold tracking-wider">Appearance</div>
        <button @click="toggleDarkMode(); showSettingsMenu = false"
          class="w-full text-left px-3 py-2 hover:bg-hover text-text-primary flex items-center gap-2 transition-colors">
          <Sun v-if="isDark" :size="13" class="text-text-tertiary" />
          <Moon v-else :size="13" class="text-text-tertiary" />
          {{ isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode' }}
        </button>

        <div class="h-px bg-border my-1.5 mx-2" />
        <button @click="handleCheckForUpdates(); showSettingsMenu = false"
          class="w-full text-left px-3 py-2 hover:bg-hover text-text-primary flex items-center gap-2 transition-colors">
          <RefreshCw :size="13" class="text-text-tertiary" :class="{ 'animate-spin': updaterStore.isChecking }" />
          Check for Updates
        </button>
      </ContextMenu>

    </div>

    <!-- Center: connection info -->
    <div class="flex items-center gap-3 ml-4 flex-1">
      <template v-if="connectionsStore.activeConnection?.isConnected">
        <span class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-success shadow-[0_0_8px_rgba(34,197,94,0.4)]"></span>
          <span class="font-medium text-text-primary">Connected</span>
        </span>
        <span class="text-border-strong opacity-30">│</span>
        <span>{{ connectionsStore.activeConnection.type.toUpperCase() }}</span>
        <span class="text-border-strong opacity-30">│</span>
        <span>{{ connectionsStore.activeConnection.host }}:{{ connectionsStore.activeConnection.port }}</span>
        <span class="text-border-strong opacity-30">│</span>
        <span>{{ gridStore.activeTableSchema || 'default' }}</span>
      </template>
      <template v-else>
        <span class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-text-tertiary/30"></span>
          <span>Not connected</span>
        </span>
      </template>
    </div>

    <!-- Right: query stats -->
    <div class="flex items-center gap-3 shrink-0" v-if="connectionsStore.activeConnection?.isConnected">
      <span class="flex items-center gap-1 text-text-tertiary">
        <Loader2 v-if="gridStore.isLoading" :size="11" class="animate-spin text-primary" />
        <template v-else>
           <svg class="w-3 h-3 text-primary opacity-70" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <polyline points="12 6 12 12 16 14" />
          </svg>
          {{ displayExecTime }} ms
        </template>
      </span>
      <span class="flex items-center gap-1 text-text-tertiary">
        <svg class="w-3 h-3 text-primary opacity-70" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20 6 9 17 4 12" />
        </svg>
        {{ displayRowCount.toLocaleString() }} rows
      </span>
    </div>

    <!-- Right: Panel Toggles -->
    <div class="flex items-center gap-1.5 ml-3 pl-3 border-l border-border shrink-0">
      <button
        class="flex items-center justify-center w-6 h-6 rounded hover:bg-hover transition-colors cursor-pointer"
        :class="layoutStore.isBottomVisible ? 'text-primary bg-primary/10' : 'text-text-tertiary hover:text-text-secondary'"
        @click="layoutStore.togglePanel('console')" title="Toggle Console (Ctrl+J)">
        <PanelBottom :size="14" />
      </button>
      <button
        class="flex items-center justify-center w-6 h-6 rounded hover:bg-hover transition-colors cursor-pointer"
        :class="layoutStore.isRightVisible ? 'text-primary bg-primary/10' : 'text-text-tertiary hover:text-text-secondary'"
        @click="layoutStore.togglePanel('inspector')" title="Toggle Inspector (Ctrl+I)">
        <PanelRight :size="14" />
      </button>
    </div>
  </footer>
</template>
