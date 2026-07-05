<script setup lang="ts">
import DatabaseTree from '@/components/sidebar/DatabaseTree.vue';
import Button from '@/components/ui/Button.vue';
import { useConnectionsStore } from '@/stores/connections';
import { useSchemaStore } from '@/stores/schema';
import { Plus, Search } from 'lucide-vue-next';

const connectionsStore = useConnectionsStore();
const schemaStore = useSchemaStore();
</script>

<template>
  <aside
    class="bg-sidebar border-border flex w-(--sidebar-width) shrink-0 flex-col overflow-hidden border-r"
  >
    <!-- Header -->
    <div class="border-border shrink-0 border-b px-2.5 py-2.5">
      <Button
        id="btn-new-connection"
        variant="primary"
        class="w-full"
        :icon="Plus"
        @click="connectionsStore.toggleConnectionModal(true)"
      >
        New Connection
      </Button>
    </div>

    <!-- Search filter -->
    <div class="border-border shrink-0 border-b px-2.5 py-2">
      <div
        class="bg-surface border-border focus-within:border-primary/50 flex items-center gap-2 rounded-lg border px-2.5 py-1.5 transition-colors"
      >
        <Search :size="12" class="text-text-tertiary shrink-0" />
        <input
          type="text"
          placeholder="Search tables"
          class="text-text-primary placeholder-text-tertiary flex-1 border-none bg-transparent text-[12px] outline-none"
          :value="schemaStore.filterQuery"
          @input="schemaStore.setFilter(($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <!-- Unified tree -->
    <DatabaseTree class="min-h-0 flex-1" />
  </aside>
</template>
