<script setup lang="ts">
import Checkbox from '@/components/ui/Checkbox.vue';
import { useConnectionsStore } from '@/stores/connections';
import type { Connection } from '@/types';
import { AlertCircle, X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const connectionsStore = useConnectionsStore();

type ImportItem = {
  connection: Connection;
  selected: boolean;
  conflict: boolean;
  resolution: 'copy' | 'overwrite';
};

const items = ref<ImportItem[]>([]);

watch(
  () => connectionsStore.showImportModal,
  (show) => {
    if (show) {
      items.value = connectionsStore.importedConnections.map((conn) => {
        const conflict = connectionsStore.connections.some(
          (c) => c.id === conn.id || c.name === conn.name
        );
        return {
          connection: conn,
          selected: true,
          conflict,
          resolution: 'copy'
        };
      });
    }
  }
);

const isAllSelected = computed(() => {
  return items.value.length > 0 && items.value.every((i) => i.selected);
});

const toggleSelectAll = () => {
  const newVal = !isAllSelected.value;
  items.value.forEach((i) => {
    i.selected = newVal;
  });
};

const handleClose = () => {
  connectionsStore.toggleImportModal(false);
};

const handleImport = async () => {
  const selectedItems = items.value.filter((i) => i.selected);
  if (selectedItems.length > 0) {
    await connectionsStore.importConnections(selectedItems);
  }
};

const selectedCount = computed(() => items.value.filter((i) => i.selected).length);
</script>

<template>
  <Teleport to="body">
    <div
      v-if="connectionsStore.showImportModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-sm"
      @click.self="handleClose"
    >
      <div
        class="bg-surface shadow-modal animate-in modal-container flex max-h-[90vh] w-140 flex-col overflow-hidden rounded-xl"
      >
        <!-- Header -->
        <div class="border-border flex items-center justify-between border-b px-5 py-3.5">
          <h2 class="text-text-primary text-[15px] font-semibold">Import Connections</h2>
          <Button rounded variant="text" severity="secondary" @click="handleClose">
            <template #icon>
              <X class="h-4 w-4" />
            </template>
          </Button>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-y-auto px-5 py-4">
          <div class="border-border rounded-lg border">
            <div class="border-border bg-sidebar/50 flex items-center gap-3 border-b px-3 py-2">
              <Checkbox :model-value="isAllSelected" @update:model-value="toggleSelectAll" />
              <span class="text-text-secondary text-[13px] font-medium">Select All</span>
            </div>
            <div class="max-h-[50vh] overflow-y-auto">
              <div
                v-for="(item, index) in items"
                :key="item.connection.id + '-' + index"
                class="border-border hover:bg-sidebar/30 flex flex-col border-b px-3 py-2 last:border-b-0"
              >
                <div class="flex items-center gap-3">
                  <Checkbox v-model="item.selected" />
                  <div class="flex flex-1 items-center gap-2">
                    <div
                      class="h-2 w-2 rounded-full"
                      :class="
                        item.connection.color ? `bg-conn-${item.connection.color}` : 'bg-conn-gray'
                      "
                    />
                    <span class="text-text-primary text-[13px]">{{ item.connection.name }}</span>
                    <span class="text-text-tertiary font-mono text-[11px]"
                      >{{ item.connection.host }}:{{ item.connection.port }}</span
                    >
                  </div>
                </div>

                <div v-if="item.conflict" class="bg-warning/10 mt-2 ml-7 rounded-md px-3 py-2">
                  <div class="text-warning flex items-center gap-1.5">
                    <AlertCircle :size="14" />
                    <span class="text-[12px] font-medium"
                      >Conflict: Connection with this name or ID already exists</span
                    >
                  </div>
                  <div class="mt-2 flex items-center gap-2">
                    <span class="text-text-secondary text-[12px]">Resolution:</span>
                    <select
                      v-model="item.resolution"
                      class="border-border text-text-primary bg-surface focus:border-primary focus:ring-primary/20 cursor-pointer appearance-none rounded-md border px-2 py-1 text-[12px] transition-all outline-none focus:ring-1"
                      :class="{
                        'border-danger/30 bg-danger/5 text-danger font-medium':
                          item.resolution === 'overwrite'
                      }"
                    >
                      <option value="copy" class="text-text-primary bg-surface">
                        Import as copy
                      </option>
                      <option value="overwrite" class="text-danger bg-surface">
                        Overwrite existing
                      </option>
                    </select>
                  </div>
                </div>
              </div>
              <div
                v-if="items.length === 0"
                class="text-text-tertiary px-3 py-4 text-center text-[13px]"
              >
                No valid connections found in file.
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="border-border bg-muted flex items-center justify-between border-t px-5 py-3">
          <div class="text-text-tertiary text-[12px]">
            {{ selectedCount }} connection(s) selected
          </div>
          <div class="flex items-center gap-2">
            <Button variant="text" severity="secondary" size="small" @click="handleClose">
              Cancel
            </Button>
            <Button :disabled="selectedCount === 0" size="small" @click="handleImport">
              Import
            </Button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.animate-in {
  animation: modal-in 0.2s ease-out;
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: scale(0.96) translateY(8px);
  }

  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>
