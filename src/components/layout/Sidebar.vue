<script setup lang="ts">
import Button from 'primevue/button';
import IconField from 'primevue/iconfield';
import InputIcon from 'primevue/inputicon';
import InputText from 'primevue/inputtext';
import DatabaseTree from '@/components/sidebar/DatabaseTree.vue';
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
        fluid
        id="btn-new-connection"
        class="text-sm"
        @click="connectionsStore.toggleConnectionModal(true)"
      >
        <Plus :size="14" />
        New Connection
      </Button>
    </div>

    <!-- Search filter -->
    <div class="border-border shrink-0 border-b px-2.5 py-2">
      <IconField class="w-full">
        <InputIcon>
          <Search :size="12" />
        </InputIcon>
        <InputText
          :model-value="schemaStore.filterQuery"
          placeholder="Search tables"
          size="small"
          fluid
          class="text-[12px]"
          @update:model-value="(val) => schemaStore.setFilter(val ?? '')"
        />
      </IconField>
    </div>

    <!-- Unified tree -->
    <DatabaseTree class="min-h-0 flex-1" />
  </aside>
</template>
