<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections'
import { useGridStore } from '@/stores/grid'
import {
  Settings,
  RefreshCw,
} from 'lucide-vue-next'

const connectionsStore = useConnectionsStore()
const gridStore = useGridStore()
</script>

<template>
  <footer class="flex items-center h-[var(--statusbar-height)] bg-surface border-t border-border px-3 text-[11px] text-text-secondary shrink-0">
    <!-- Left: icons -->
    <div class="flex items-center gap-2">
      <button class="flex items-center justify-center w-6 h-6 rounded hover:bg-hover text-text-tertiary hover:text-text-secondary">
        <Settings :size="13" />
      </button>
      <button class="flex items-center justify-center w-6 h-6 rounded hover:bg-hover text-text-tertiary hover:text-text-secondary">
        <RefreshCw :size="13" />
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
