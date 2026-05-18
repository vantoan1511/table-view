<script setup lang="ts">
import { useGridStore } from '@/stores/grid';

import { ref } from 'vue';
import { AlertCircle, X } from 'lucide-vue-next';

const props = defineProps<{
  connectionId: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gridStore = useGridStore();
const dbName = ref('');

const handleCreate = async () => {
  if (!dbName.value.trim()) return;

  try {
    await gridStore.createDatabase(props.connectionId, dbName.value);
    emit('close');
  } catch (err) {
    // Error handled by store/toast
  }
};
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
    <div
      class="bg-surface border-border flex w-100 flex-col overflow-hidden rounded-xl border shadow-2xl"
    >
      <!-- Header -->
      <div class="border-border flex items-center justify-between border-b px-5 py-4">
        <h3 class="text-text-primary text-base font-semibold">Create Database</h3>
        <button
          @click="$emit('close')"
          class="text-text-tertiary hover:text-text-primary transition-colors"
        >
          <X :size="18" />
        </button>
      </div>

      <!-- Body -->
      <div class="p-5">
        <div class="mb-2">
          <div class="mb-1.5 flex items-center justify-between">
            <label class="text-text-secondary block text-[13px] font-medium">Database Name</label>
            <span
              v-if="!dbName.trim()"
              class="text-danger animate-fade-in-scale flex items-center gap-1 text-[11px] font-medium"
            >
              <AlertCircle :size="12" />
              Required
            </span>
          </div>
          <input
            v-model.trim="dbName"
            type="text"
            placeholder="Enter database name..."
            class="bg-muted border-border text-text-primary focus:border-primary w-full rounded-lg border px-4 py-2 text-[14px] transition-colors outline-none"
            :class="{ 'border-danger! !focus:border-danger': !dbName.trim() }"
            @keyup.enter="handleCreate"
            autofocus
          />
        </div>
      </div>

      <!-- Footer -->
      <div class="border-border bg-muted/30 flex items-center justify-end gap-3 border-t px-5 py-4">
        <button
          @click="$emit('close')"
          class="text-text-secondary hover:text-text-primary border-border bg-surface hover:bg-hover rounded-lg border px-4 py-2 text-[13px] font-medium transition-colors"
        >
          Cancel
        </button>
        <button
          @click="handleCreate"
          class="bg-primary hover:bg-primary-hover rounded-lg px-4 py-2 text-[13px] font-medium text-white shadow-sm transition-colors"
          :disabled="!dbName.trim()"
        >
          Create Database
        </button>
      </div>
    </div>
  </div>
</template>
