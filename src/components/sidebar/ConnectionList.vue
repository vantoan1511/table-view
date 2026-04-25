<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections';
import { MoreVertical } from 'lucide-vue-next';

const connectionsStore = useConnectionsStore()

const colorMap: Record<string, string> = {
  indigo: 'bg-conn-indigo',
  blue: 'bg-conn-blue',
  teal: 'bg-conn-teal',
  green: 'bg-conn-green',
  amber: 'bg-conn-amber',
  orange: 'bg-conn-orange',
  pink: 'bg-conn-pink',
  gray: 'bg-conn-gray',
}
</script>

<template>
  <div class="px-2 pb-1">
    <button v-for="conn in connectionsStore.connections" :key="conn.id"
      class="group flex items-center gap-2.5 w-full px-2.5 py-2 rounded-lg cursor-pointer text-left transition-all duration-150"
      :class="connectionsStore.activeConnectionId === conn.id
          ? 'bg-active border border-primary/20'
          : 'hover:bg-hover border border-transparent'
        " @click="connectionsStore.setActiveConnection(conn.id)">
      <!-- Color dot / connected indicator -->
      <span class="w-2.5 h-2.5 rounded-full shrink-0"
        :class="conn.isConnected ? 'bg-success' : colorMap[conn.color] ?? 'bg-conn-gray'"></span>

      <!-- Info -->
      <div class="flex-1 min-w-0">
        <div class="text-[13px] font-medium truncate"
          :class="connectionsStore.activeConnectionId === conn.id ? 'text-primary' : 'text-text-primary'">
          {{ conn.name }}
        </div>
        <div class="text-[11px] text-text-tertiary truncate">
          {{ conn.username }}@{{ conn.host }}:{{ conn.port }}
        </div>
      </div>

      <!-- More button -->
      <span
        class="flex items-center justify-center w-5 h-5 rounded opacity-0 group-hover:opacity-100 text-text-tertiary hover:text-text-secondary hover:bg-border transition-opacity">
        <MoreVertical :size="13" />
      </span>
    </button>
  </div>
</template>
