<script setup lang="ts">
import ConnectionList from '@/components/sidebar/ConnectionList.vue'
import SchemaTree from '@/components/sidebar/SchemaTree.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useSchemaStore } from '@/stores/schema'
import {
  Plus,
  ChevronDown,
  Search,
} from 'lucide-vue-next'

const connectionsStore = useConnectionsStore()
const schemaStore = useSchemaStore()
</script>

<template>
  <aside class="flex flex-col w-[var(--sidebar-width)] bg-sidebar border-r border-border shrink-0 overflow-hidden">
    <!-- Connection Dropdown -->
    <div class="flex items-center gap-2 px-3 py-2.5 border-b border-border">
      <button
        class="flex items-center gap-2 flex-1 px-2.5 py-1.5 bg-surface border border-border rounded-lg text-[13px] text-text-primary hover:border-border-strong transition-colors cursor-pointer"
      >
        <span
          class="w-2.5 h-2.5 rounded-full shrink-0"
          :class="connectionsStore.activeConnection?.isConnected ? 'bg-success' : 'bg-text-tertiary'"
        ></span>
        <span class="flex-1 text-left truncate font-medium">
          {{ connectionsStore.activeConnection?.name ?? 'No Connection' }}
        </span>
        <ChevronDown :size="14" class="text-text-tertiary shrink-0" />
      </button>
    </div>

    <!-- Connections Header -->
    <div class="flex items-center justify-between px-3 py-2">
      <span class="text-[11px] font-semibold text-text-tertiary uppercase tracking-wider">Connections</span>
      <button
        class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-text-secondary hover:bg-hover"
        @click="connectionsStore.toggleConnectionModal(true)"
      >
        <Plus :size="14" />
      </button>
    </div>

    <!-- Connection List -->
    <ConnectionList />

    <!-- Database Section -->
    <div class="px-3 py-2 border-t border-border">
      <span class="text-[11px] font-semibold text-text-tertiary uppercase tracking-wider">Database</span>
      <div class="mt-1.5">
        <button class="flex items-center gap-2 w-full px-2.5 py-1.5 bg-surface border border-border rounded-lg text-[13px] text-text-primary hover:border-border-strong transition-colors cursor-pointer">
          <span class="flex-1 text-left">{{ schemaStore.selectedSchema }}</span>
          <ChevronDown :size="14" class="text-text-tertiary" />
        </button>
      </div>
    </div>

    <!-- Search Filter -->
    <div class="px-3 py-1.5">
      <div class="flex items-center gap-2 px-2.5 py-1.5 bg-surface border border-border rounded-lg text-[13px]">
        <Search :size="13" class="text-text-tertiary shrink-0" />
        <input
          type="text"
          placeholder="Filter objects..."
          class="flex-1 bg-transparent border-none outline-none text-[13px] text-text-primary placeholder-text-tertiary"
          :value="schemaStore.filterQuery"
          @input="schemaStore.setFilter(($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <!-- Schema Tree -->
    <div class="flex-1 overflow-y-auto">
      <SchemaTree />
    </div>
  </aside>
</template>
