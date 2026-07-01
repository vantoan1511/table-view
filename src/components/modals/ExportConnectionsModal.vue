<script setup lang="ts">
import Button from '@/components/ui/Button.vue';
import Checkbox from '@/components/ui/Checkbox.vue';
import { useConnectionsStore } from '@/stores/connections';
import { X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const connectionsStore = useConnectionsStore();

const selectedIds = ref<string[]>([]);
const includePasswords = ref(false);

watch(
  () => connectionsStore.showExportModal,
  (show) => {
    if (show) {
      selectedIds.value = connectionsStore.connections.map((c) => c.id);
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
  <Teleport to="body">
    <div
      v-if="connectionsStore.showExportModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-sm"
      @click.self="handleClose"
    >
      <div
        class="bg-surface shadow-modal animate-in modal-container flex max-h-[90vh] w-120 flex-col overflow-hidden rounded-xl"
      >
        <!-- Header -->
        <div class="border-border flex items-center justify-between border-b px-5 py-3.5">
          <h2 class="text-text-primary text-[15px] font-semibold">Export Connections</h2>
          <Button variant="ghost" size="icon" @click="handleClose">
            <X :size="16" />
          </Button>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-y-auto px-5 py-4">
          <div class="border-border rounded-lg border">
            <div class="border-border bg-sidebar/50 flex items-center gap-3 border-b px-3 py-2">
              <Checkbox :model-value="isAllSelected" @update:model-value="toggleSelectAll" />
              <span class="text-text-secondary text-[13px] font-medium">Select All</span>
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
                />
                <div class="flex items-center gap-2">
                  <div
                    class="h-2 w-2 rounded-full"
                    :class="conn.color ? `bg-conn-${conn.color}` : 'bg-conn-gray'"
                  />
                  <span class="text-text-primary text-[13px]">{{ conn.name }}</span>
                  <span class="text-text-tertiary font-mono text-[11px]"
                    >{{ conn.host }}:{{ conn.port }}</span
                  >
                </div>
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
            <Checkbox v-model="includePasswords" class="mt-0.5" />
            <div class="flex flex-col">
              <span class="text-text-primary text-[13px] font-medium"
                >Include passwords / credentials</span
              >
              <span class="text-text-tertiary text-[11px]">
                Passwords will be encrypted, but anyone with Table View can import and use them.
              </span>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="border-border bg-muted flex items-center justify-between border-t px-5 py-3">
          <div class="text-text-tertiary text-[12px]">
            {{ selectedIds.length }} connection(s) selected
          </div>
          <div class="flex items-center gap-2">
            <Button variant="secondary" @click="handleClose">Cancel</Button>
            <Button variant="primary" :disabled="selectedIds.length === 0" @click="handleExport">
              Export
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
