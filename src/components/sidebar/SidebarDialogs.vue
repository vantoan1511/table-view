<script setup lang="ts">
import Button from '../ui/Button.vue';
import Checkbox from '../ui/Checkbox.vue';
import ConfirmDialog from '../ui/ConfirmDialog.vue';
import CreateDatabaseDialog from '../ui/CreateDatabaseDialog.vue';
import CreateSchemaDialog from '../ui/CreateSchemaDialog.vue';
import CreateTableDialog from '../ui/CreateTableDialog.vue';

import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useSchemaStore } from '@/stores/schema';
import { useToastStore } from '@/stores/toast';

import { AlertTriangle, Trash2, X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const props = defineProps<{
  idToDelete: string | null;
  tableToDelete: {
    name: string;
    connId: string;
    dbName?: string;
    schemaName?: string;
  } | null;
  schemaToDelete: {
    name: string;
    connId: string;
    dbName?: string;
  } | null;
  databaseToDelete: {
    name: string;
    connId: string;
  } | null;
}>();

const emit = defineEmits<{
  (e: 'update:idToDelete', value: string | null): void;
  (e: 'update:tableToDelete', value: any | null): void;
  (e: 'update:schemaToDelete', value: any | null): void;
  (e: 'update:databaseToDelete', value: any | null): void;
  (e: 'closeDeleteConfirm'): void;
  (e: 'closeTableDeleteConfirm'): void;
  (e: 'closeSchemaDeleteConfirm'): void;
  (e: 'closeDatabaseDeleteConfirm'): void;
}>();

const connectionsStore = useConnectionsStore();
const gridStore = useGridStore();
const schemaStore = useSchemaStore();
const toastStore = useToastStore();

const connectionName = computed(
  () => connectionsStore.connections.find((c) => c.id === props.idToDelete)?.name || ''
);

// 2-step database deletion state
const confirmStep = ref(1);
const typedDbName = ref('');
const backedUpChecked = ref(false);

const isDeletingActiveDb = computed(() => {
  if (!props.databaseToDelete) return false;
  const { name, connId } = props.databaseToDelete;
  const activeConn = connectionsStore.connections.find((c) => c.id === connId);
  return activeConn?.database === name || gridStore.activeDbName === name;
});

// Reset confirmation state when databaseToDelete changes
watch(
  () => props.databaseToDelete,
  () => {
    confirmStep.value = 1;
    typedDbName.value = '';
    backedUpChecked.value = false;
  }
);

const confirmDelete = async () => {
  if (!props.idToDelete) return;
  const id = props.idToDelete;
  await connectionsStore.removeConnection(id);
  schemaStore.removeConnection(id);

  toastStore.addToast({
    message: 'Connection deleted.',
    severity: 'success',
    variation: 'subtle'
  });

  emit('update:idToDelete', null);
  emit('closeDeleteConfirm');
};

const confirmTableDelete = async () => {
  if (!props.tableToDelete) return;
  const { name, connId, dbName, schemaName } = props.tableToDelete;
  try {
    await gridStore.dropTable(name, connId, schemaName, dbName);
  } finally {
    emit('update:tableToDelete', null);
    emit('closeTableDeleteConfirm');
  }
};

const confirmSchemaDelete = async () => {
  if (!props.schemaToDelete) return;
  const { name, connId, dbName } = props.schemaToDelete;
  try {
    await gridStore.dropSchema(connId, name, dbName);
  } finally {
    emit('update:schemaToDelete', null);
    emit('closeSchemaDeleteConfirm');
  }
};

const closeDatabaseDelete = () => {
  emit('update:databaseToDelete', null);
  emit('closeDatabaseDeleteConfirm');
  confirmStep.value = 1;
  typedDbName.value = '';
  backedUpChecked.value = false;
};

const confirmDatabaseDelete = async () => {
  if (!props.databaseToDelete) return;
  const { name, connId } = props.databaseToDelete;
  try {
    await gridStore.dropDatabase(connId, name);
  } finally {
    closeDatabaseDelete();
  }
};
</script>

<template>
  <div>
    <!-- Delete Confirmation -->
    <ConfirmDialog
      v-if="idToDelete"
      title="Delete Connection"
      :message="`Are you sure you want to delete '${connectionName}'? This action cannot be undone.`"
      confirm-label="Delete"
      variant="danger"
      @confirm="confirmDelete"
      @cancel="
        emit('update:idToDelete', null);
        emit('closeDeleteConfirm');
      "
    />

    <ConfirmDialog
      v-if="tableToDelete"
      title="Delete Table"
      :message="`Are you sure you want to drop table '${tableToDelete?.name}'? This action cannot be undone.`"
      confirm-label="Drop Table"
      variant="danger"
      @confirm="confirmTableDelete"
      @cancel="
        emit('update:tableToDelete', null);
        emit('closeTableDeleteConfirm');
      "
    />

    <ConfirmDialog
      v-if="schemaToDelete"
      title="Delete Schema"
      :message="`Are you sure you want to drop schema '${schemaToDelete?.name}'? This action will delete all objects within it and cannot be undone.`"
      confirm-label="Drop Schema"
      variant="danger"
      @confirm="confirmSchemaDelete"
      @cancel="
        emit('update:schemaToDelete', null);
        emit('closeSchemaDeleteConfirm');
      "
    />

    <Teleport to="body">
      <div v-if="databaseToDelete" class="fixed inset-0 z-200 flex items-center justify-center">
        <!-- Scrim -->
        <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" @click="closeDatabaseDelete" />

        <!-- Dialog Container -->
        <div
          class="bg-surface shadow-modal border-border relative z-10 mx-4 w-full max-w-md animate-[scale-in_0.15s_ease-out] overflow-hidden rounded-xl border text-left"
          role="dialog"
          aria-modal="true"
        >
          <!-- Header -->
          <div class="flex items-start gap-3 p-5 pb-3">
            <div class="bg-danger-light text-danger shrink-0 rounded-lg p-2">
              <Trash2 :size="18" />
            </div>
            <div class="min-w-0 flex-1 pt-0.5">
              <h3 class="text-text-primary text-[14px] leading-tight font-semibold">
                Delete Database: Step {{ confirmStep }} of 2
              </h3>
              <p class="text-text-tertiary mt-0.5 text-[11px]">Highly Critical Action</p>
            </div>
            <Button variant="ghost" size="icon" @click="closeDatabaseDelete">
              <X :size="16" />
            </Button>
          </div>

          <!-- Divider -->
          <div class="bg-border mx-5 h-px" />

          <!-- Body Content (Step 1) -->
          <div v-if="confirmStep === 1" class="flex flex-col gap-4 p-5 py-4">
            <div
              class="bg-danger/10 border-danger/20 text-danger flex items-start gap-2.5 rounded-lg border p-3.5 text-[13px] leading-relaxed"
            >
              <AlertTriangle :size="16" class="mt-0.5 shrink-0" />
              <div>
                <p class="text-danger font-semibold">WARNING: Permanent Data Loss</p>
                <p class="text-text-secondary mt-1 opacity-90">
                  You are about to drop database
                  <strong class="text-text-primary underline">{{ databaseToDelete.name }}</strong
                  >. This will immediately destroy all tables, schemas, views, and stored data. This
                  action is <strong class="text-danger font-bold uppercase">irreversible</strong>.
                </p>
                <p
                  v-if="isDeletingActiveDb"
                  class="bg-danger/20 border-danger/20 text-text-primary mt-2 rounded border p-1.5 text-[12px] font-medium"
                >
                  Note: This is the currently open/active database. Dropping it will close active
                  connections and switch your session context.
                </p>
              </div>
            </div>

            <div class="flex flex-col gap-1.5">
              <label class="text-text-secondary text-[12px] font-medium">
                To confirm, type
                <span
                  class="bg-hover text-text-primary border-border rounded border px-1.5 py-0.5 font-mono text-[11px] font-semibold select-all"
                  >{{ databaseToDelete.name }}</span
                >
                below:
              </label>
              <input
                v-model="typedDbName"
                type="text"
                placeholder="Enter database name"
                class="bg-hover border-border focus:border-primary/50 text-text-primary w-full rounded-lg border px-3 py-2 text-[13px] transition-colors outline-none"
                @keyup.enter="typedDbName === databaseToDelete.name && (confirmStep = 2)"
              />
            </div>
          </div>

          <!-- Body Content (Step 2) -->
          <div v-else class="flex flex-col gap-4 p-5 py-4">
            <div
              class="bg-warning/10 border-warning/20 text-warning flex items-start gap-2.5 rounded-lg border p-3.5 text-[13px] leading-relaxed"
            >
              <AlertTriangle :size="16" class="mt-0.5 shrink-0" />
              <div>
                <p class="text-warning font-semibold">Step 2: Backup Verification</p>
                <p class="text-text-secondary mt-1 opacity-90">
                  Have you backed up your data? We highly recommend exporting or copying important
                  information before proceeding.
                </p>
              </div>
            </div>

            <div
              class="hover:bg-hover border-border flex cursor-pointer items-start gap-3 rounded-lg border p-3.5 transition-colors"
              @click="backedUpChecked = !backedUpChecked"
            >
              <Checkbox v-model="backedUpChecked" class="mt-1 shrink-0" @click.stop />
              <span class="text-text-secondary text-[13px] leading-relaxed select-none">
                I have backed up my database and understand that this action is permanent and cannot
                be undone.
              </span>
            </div>
          </div>

          <!-- Divider -->
          <div class="bg-border mx-5 h-px" />

          <!-- Footer -->
          <div class="flex items-center justify-end gap-2 p-4">
            <Button variant="secondary" @click="closeDatabaseDelete">Cancel</Button>

            <!-- Step 1 Button -->
            <Button
              v-if="confirmStep === 1"
              variant="danger"
              :disabled="typedDbName !== databaseToDelete.name"
              @click="confirmStep = 2"
            >
              Next Step
            </Button>

            <!-- Step 2 Button -->
            <Button
              v-else
              variant="danger"
              :disabled="!backedUpChecked"
              @click="confirmDatabaseDelete"
            >
              Confirm & Drop Database
            </Button>
          </div>
        </div>
      </div>
    </Teleport>

    <CreateTableDialog
      v-if="gridStore.showCreateTableDialog && gridStore.createTableTarget"
      :connection-id="gridStore.createTableTarget.connectionId"
      :schema="gridStore.createTableTarget.schema"
      :db="gridStore.createTableTarget.db"
      @close="gridStore.showCreateTableDialog = false"
    />

    <CreateSchemaDialog
      v-if="gridStore.showCreateSchemaDialog && gridStore.createSchemaTarget"
      :connection-id="gridStore.createSchemaTarget.connectionId"
      :db="gridStore.createSchemaTarget.db"
      @close="gridStore.showCreateSchemaDialog = false"
    />

    <CreateDatabaseDialog
      v-if="gridStore.showCreateDatabaseDialog && gridStore.createDatabaseTarget"
      :connection-id="gridStore.createDatabaseTarget.connectionId"
      @close="gridStore.showCreateDatabaseDialog = false"
    />
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
