<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';

import { DbType } from '@/types';

import { AlertCircle, Info, X } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const props = defineProps<{
  connectionId: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gridStore = useGridStore();
const connectionsStore = useConnectionsStore();

const connection = computed(() =>
  connectionsStore.connections.find((c) => c.id === props.connectionId)
);

const isOracle = computed(() => connection.value?.type === DbType.ORACLE);

const dbName = ref('');
const password = ref('');
const submitted = ref(false);

const handleCreate = async () => {
  submitted.value = true;
  if (!dbName.value.trim()) return;
  if (isOracle.value && !password.value.trim()) return;

  try {
    await gridStore.createDatabase(
      props.connectionId,
      dbName.value.trim(),
      isOracle.value ? password.value.trim() : undefined
    );
    emit('close');
  } catch {
    // Error handled by store/toast
  }
};
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
    <div
      class="bg-surface border-border flex w-100 animate-[scale-in_0.15s_ease-out] flex-col overflow-hidden rounded-xl border shadow-2xl"
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
      <div class="flex flex-col gap-4 p-5">
        <!-- Database/Schema Name -->
        <div>
          <div class="mb-1.5 flex items-center justify-between">
            <label class="text-text-secondary block text-[13px] font-medium">
              {{ isOracle ? 'Schema / User Name' : 'Database Name' }}
            </label>
            <span
              v-if="submitted && !dbName.trim()"
              class="text-danger animate-fade-in-scale flex items-center gap-1 text-[11px] font-medium"
            >
              <AlertCircle :size="12" />
              Required
            </span>
          </div>
          <input
            v-model.trim="dbName"
            type="text"
            placeholder="Enter name..."
            class="bg-muted border-border text-text-primary focus:border-primary w-full rounded-lg border px-4 py-2 text-[14px] transition-colors outline-none"
            :class="{ 'border-danger! !focus:border-danger': submitted && !dbName.trim() }"
            @keyup.enter="handleCreate"
            autofocus
          />
        </div>

        <!-- Oracle Password Field -->
        <div v-if="isOracle">
          <div class="mb-1.5 flex items-center justify-between">
            <div class="flex items-center gap-1.5">
              <label class="text-text-secondary block text-[13px] font-medium">Password</label>
              <Info
                :size="14"
                class="text-text-tertiary cursor-help"
                title="An admin account will be created with the same name as the database."
              />
            </div>
            <span
              v-if="submitted && !password.trim()"
              class="text-danger animate-fade-in-scale flex items-center gap-1 text-[11px] font-medium"
            >
              <AlertCircle :size="12" />
              Required
            </span>
          </div>
          <input
            v-model.trim="password"
            type="password"
            placeholder="Enter password for the new schema user..."
            class="bg-muted border-border text-text-primary focus:border-primary w-full rounded-lg border px-4 py-2 text-[14px] transition-colors outline-none"
            :class="{ 'border-danger! !focus:border-danger': submitted && !password.trim() }"
            @keyup.enter="handleCreate"
          />
          <p class="text-text-tertiary mt-1.5 flex items-start gap-1.5 text-[12px] leading-relaxed">
            <Info :size="12" class="mt-0.5 shrink-0" />
            <span
              >A new Oracle User/Schema will be created. The password is required to secure the new
              workspace.</span
            >
          </p>
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
        >
          Create Database
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes scale-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
