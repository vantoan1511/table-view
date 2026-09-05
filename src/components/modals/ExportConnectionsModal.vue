<script setup lang="ts">
import Dialog from 'primevue/dialog';
import Checkbox from 'primevue/checkbox';
import Button from 'primevue/button';
import { useConnectionsStore } from '@/stores/connections';
import { computed, ref, watch } from 'vue';

const connectionsStore = useConnectionsStore();

const selectedIds = ref<string[]>([]);
const includePasswords = ref(false);

watch(
  () => connectionsStore.showExportModal,
  (show) => {
    if (show) {
      if (connectionsStore.preSelectedExportId) {
        selectedIds.value = [connectionsStore.preSelectedExportId];
      } else {
        selectedIds.value = connectionsStore.connections.map((c) => c.id);
      }
      includePasswords.value = false;
    }
  }
);

const isAllSelected = computed(() => {
  return (
    connectionsStore.connections.length > 0 &&
    selectedIds.value.length === connectionsStore.connections.length
  );
});

const toggleSelectAll = () => {
  if (isAllSelected.value) {
    selectedIds.value = [];
  } else {
    selectedIds.value = connectionsStore.connections.map((c) => c.id);
  }
};

const handleClose = () => {
  connectionsStore.toggleExportModal(false);
};

const handleExport = async () => {
  if (selectedIds.value.length > 0) {
    await connectionsStore.exportConnections(selectedIds.value, includePasswords.value);
    handleClose();
  }
};
</script>

<template>
  <Dialog
    :visible="connectionsStore.showExportModal"
    modal
    :closable="true"
    header="Export Connections"
    :style="{ width: '32rem' }"
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
            inputId="select-all-export"
            binary
          />
          <label
            for="select-all-export"
            class="text-text-secondary cursor-pointer text-[13px] font-medium select-none"
            >Select All</label
          >
        </div>
        <div class="max-h-60 overflow-y-auto">
          <div
            v-for="conn in connectionsStore.connections"
            :key="conn.id"
            class="border-border hover:bg-sidebar/30 flex items-center gap-3 border-b px-3 py-2 last:border-b-0"
          >
            <Checkbox
              :model-value="selectedIds.includes(conn.id)"
              @update:model-value="
                (val) => {
                  if (val) selectedIds.push(conn.id);
                  else selectedIds = selectedIds.filter((id) => id !== conn.id);
                }
              "
              :inputId="'conn-' + conn.id"
              binary
            />
            <label
              :for="'conn-' + conn.id"
              class="flex flex-1 cursor-pointer items-center gap-2 select-none"
            >
              <div
                class="h-2 w-2 shrink-0 rounded-full"
                :class="conn.color ? `bg-conn-${conn.color}` : 'bg-conn-gray'"
              />
              <span class="text-text-primary text-[13px]">{{ conn.name }}</span>
              <span class="text-text-tertiary font-mono text-[11px]"
                >{{ conn.host }}:{{ conn.port }}</span
              >
            </label>
          </div>
          <div
            v-if="connectionsStore.connections.length === 0"
            class="text-text-tertiary px-3 py-4 text-center text-[13px]"
          >
            No connections available to export.
          </div>
        </div>
      </div>

      <div class="mt-4 flex items-start gap-2">
        <Checkbox
          v-model="includePasswords"
          inputId="include-passwords-export"
          binary
          class="mt-0.5"
        />
        <label for="include-passwords-export" class="flex cursor-pointer flex-col select-none">
          <span class="text-text-primary text-[13px] font-medium"
            >Include passwords / credentials</span
          >
          <span class="text-text-tertiary text-[11px]">
            Passwords will be encrypted, but anyone with Table View can import and use them.
          </span>
        </label>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex w-full items-center justify-between pt-2">
        <div class="text-text-tertiary text-[12px]">
          {{ selectedIds.length }} connection(s) selected
        </div>
        <div class="flex items-center gap-2">
          <Button variant="text" severity="secondary" size="small" @click="handleClose">
            Cancel
          </Button>
          <Button :disabled="selectedIds.length === 0" size="small" @click="handleExport">
            Export
          </Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>
