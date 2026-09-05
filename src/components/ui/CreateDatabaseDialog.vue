<script setup lang="ts">
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { DbType } from '@/types';
import { AlertCircle, Info } from 'lucide-vue-next';
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
  <Dialog
    visible
    modal
    header="Create Database"
    :style="{ width: '26rem' }"
    :closable="true"
    @update:visible="
      (val) => {
        if (!val) emit('close');
      }
    "
  >
    <div class="flex flex-col gap-4 py-2">
      <!-- Database/Schema Name -->
      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <label class="text-text-secondary block text-[13px] font-medium">
            {{ isOracle ? 'Schema / User Name' : 'Database Name' }}
          </label>
          <span
            v-if="submitted && !dbName.trim()"
            class="text-danger flex items-center gap-1 text-[11px] font-medium"
          >
            <AlertCircle :size="12" />
            Required
          </span>
        </div>
        <InputText
          v-model.trim="dbName"
          type="text"
          placeholder="Enter name..."
          class="w-full"
          :invalid="submitted && !dbName.trim()"
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
              v-tooltip.top="'An admin account will be created with the same name as the database.'"
            />
          </div>
          <span
            v-if="submitted && !password.trim()"
            class="text-danger flex items-center gap-1 text-[11px] font-medium"
          >
            <AlertCircle :size="12" />
            Required
          </span>
        </div>
        <InputText
          v-model.trim="password"
          type="password"
          placeholder="Enter password for the new schema user..."
          class="w-full"
          :invalid="submitted && !password.trim()"
          @keyup.enter="handleCreate"
        />
        <p class="text-text-tertiary mt-1.5 flex items-start gap-1.5 text-[12px] leading-relaxed">
          <Info :size="12" class="mt-0.5 shrink-0" />
          <span>
            A new Oracle User/Schema will be created. The password is required to secure the new
            workspace.
          </span>
        </p>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2 pt-2">
        <Button variant="outlined" severity="secondary" @click="$emit('close')"> Cancel </Button>
        <Button severity="primary" @click="handleCreate"> Create Database </Button>
      </div>
    </template>
  </Dialog>
</template>
