<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConnectionsStore } from '@/stores/connections'
import { useGridStore } from '@/stores/grid'
import { useUpdaterStore } from '@/stores/updater'
import * as Neutralino from '@neutralinojs/lib'
import {
  Settings,
  RefreshCw,
  Moon,
  Sun,
  ArrowUpCircle,
  Loader2
} from 'lucide-vue-next'

const connectionsStore = useConnectionsStore()
const gridStore = useGridStore()
const updaterStore = useUpdaterStore()
const isDark = ref(false)

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

const toggleDarkMode = async () => {
  isDark.value = !isDark.value
  document.documentElement.classList.toggle('dark', isDark.value)
  if (window.NL_PORT) {
    await Neutralino.storage.setData('theme', isDark.value ? 'dark' : 'light')
  }
}
</script>

<template>
  <footer class="flex items-center h-[var(--statusbar-height)] bg-surface border-t border-border px-3 text-[11px] text-text-secondary shrink-0">
    <!-- Left: icons -->
    <div class="flex items-center gap-2">
      <button class="flex items-center justify-center w-6 h-6 rounded hover:bg-hover text-text-tertiary hover:text-text-secondary">
        <Settings :size="13" />
      </button>
      <button class="flex items-center justify-center w-6 h-6 rounded hover:bg-hover text-text-tertiary hover:text-text-secondary"
        @click="updaterStore.checkForUpdates(true)"
        :title="updaterStore.isChecking ? 'Checking for updates...' : 'Check for updates'"
      >
        <Loader2 v-if="updaterStore.isChecking" :size="13" class="animate-spin" />
        <ArrowUpCircle v-else-if="updaterStore.updateAvailable" :size="13" class="text-primary" />
        <RefreshCw v-else :size="13" />
      </button>
      <button
        class="flex items-center justify-center w-6 h-6 rounded hover:bg-hover text-text-tertiary hover:text-text-secondary"
        @click="toggleDarkMode"
        :title="isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'"
      >
        <Sun v-if="isDark" :size="13" />
        <Moon v-else :size="13" />
      </button>
    </div>

    <!-- Center: connection info -->
    <div class="flex items-center gap-3 ml-4 flex-1">
      <template v-if="connectionsStore.activeConnection?.isConnected">
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-success"></span>
          <span class="font-medium text-text-primary">Connected</span>
        </span>
        <span class="text-border-strong">│</span>
        <span>PostgreSQL 15.2</span>
        <span class="text-border-strong">│</span>
        <span>{{ connectionsStore.activeConnection.host }}:{{ connectionsStore.activeConnection.port }}</span>
        <span class="text-border-strong">│</span>
        <span>public</span>
      </template>
      <template v-else>
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-text-tertiary"></span>
          <span>Not connected</span>
        </span>
      </template>
    </div>

    <!-- Right: query stats -->
    <div class="flex items-center gap-3 shrink-0" v-if="connectionsStore.activeConnection?.isConnected">
      <span class="flex items-center gap-1">
        <svg class="w-3 h-3 text-success" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
        {{ gridStore.executionTime }} ms
      </span>
      <span class="flex items-center gap-1">
        <svg class="w-3 h-3 text-success" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20 6 9 17 4 12" />
        </svg>
        {{ gridStore.sqlRowCount }} rows returned
      </span>
    </div>
  </footer>
</template>
