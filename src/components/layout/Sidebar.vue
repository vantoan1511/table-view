<script setup lang="ts">
import DatabaseTree from '@/components/sidebar/DatabaseTree.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useSchemaStore } from '@/stores/schema'
import { Plus, Search } from 'lucide-vue-next'

const connectionsStore = useConnectionsStore()
const schemaStore = useSchemaStore()
</script>

<template>
  <aside class="flex flex-col w-(--sidebar-width) bg-sidebar border-r border-border shrink-0 overflow-hidden">
    <!-- Header -->
    <div class="px-2.5 py-2.5 border-b border-border shrink-0">
      <button id="btn-new-connection"
        class="flex items-center justify-center gap-1.5 w-full px-3 py-1.5 bg-primary hover:bg-primary-hover text-text-inverse rounded-lg text-[12px] font-medium cursor-pointer transition-colors shadow-sm"
        @click="connectionsStore.toggleConnectionModal(true)"
      >
        <Plus :size="14" />
        New Connection
      </button>
    </div>

    <!-- Search filter -->
    <div class="px-2.5 py-2 border-b border-border shrink-0">
      <div class="flex items-center gap-2 px-2.5 py-1.5 bg-surface border border-border rounded-lg focus-within:border-primary/50 transition-colors">
        <Search :size="12" class="text-text-tertiary shrink-0" />
        <input
          type="text"
          placeholder="Search connections..."
          class="flex-1 bg-transparent border-none outline-none text-[12px] text-text-primary placeholder-text-tertiary"
          :value="schemaStore.filterQuery"
          @input="schemaStore.setFilter(($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <!-- Unified tree -->
    <DatabaseTree class="flex-1 min-h-0" />
  </aside>
</template>
