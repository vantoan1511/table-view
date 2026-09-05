<script setup lang="ts">
import Dialog from 'primevue/dialog';
import Select from 'primevue/select';
import Checkbox from 'primevue/checkbox';
import Button from 'primevue/button';
import { useConnectionsStore } from '@/stores/connections';
import type { Connection } from '@/types';
import { AlertCircle } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const connectionsStore = useConnectionsStore();

type ImportItem = {
  connection: Connection;
  selected: boolean;
  conflict: boolean;
  resolution: 'copy' | 'overwrite';
};

const resolutionOptions = [
  { label: 'Import as copy', value: 'copy' },
  { label: 'Overwrite existing', value: 'overwrite' }
];

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
  <Dialog
    :visible="connectionsStore.showImportModal"
    modal
    :closable="true"
    header="Import Connections"
    :style="{ width: '36rem' }"
    @update:visible="
      (val) => {
        if (!val) handleClose();
      }
    "
  >
    <div class="py-2">
      <div class="border-border rounded-lg border">
        <div class="border-border bg-sidebar/50 flex items-center gap-3 border-b px-3 py-2">
          <Checkbox
            :model-value="isAllSelected"
            @update:model-value="toggleSelectAll"
            inputId="select-all-import"
            binary
          />
          <label
            for="select-all-import"
            class="text-text-secondary cursor-pointer text-[13px] font-medium select-none"
            >Select All</label
          >
        </div>
        <div class="max-h-[50vh] overflow-y-auto">
          <div
            v-for="(item, index) in items"
            :key="item.connection.id + '-' + index"
            class="border-border hover:bg-sidebar/30 flex flex-col border-b px-3 py-2 last:border-b-0"
          >
            <div class="flex items-center gap-3">
              <Checkbox v-model="item.selected" :inputId="'import-conn-' + index" binary />
              <label
                :for="'import-conn-' + index"
                class="flex flex-1 cursor-pointer items-center gap-2 select-none"
              >
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
              </label>
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
                <Select
                  v-model="item.resolution"
                  :options="resolutionOptions"
                  optionLabel="label"
                  optionValue="value"
                  size="small"
                  class="text-[12px]"
                />
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
    <template #footer>
      <div class="flex w-full items-center justify-between pt-2">
        <div class="text-text-tertiary text-[12px]">{{ selectedCount }} connection(s) selected</div>
        <div class="flex items-center gap-2">
          <Button variant="text" severity="secondary" size="small" @click="handleClose">
            Cancel
          </Button>
          <Button :disabled="selectedCount === 0" size="small" @click="handleImport">
            Import
          </Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>
