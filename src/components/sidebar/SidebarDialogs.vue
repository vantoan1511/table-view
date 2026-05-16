<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useSchemaStore } from '@/stores/schema';
import { useToastStore } from '@/stores/toast';
import { computed } from 'vue';
import ConfirmDialog from '../ui/ConfirmDialog.vue';
import CreateTableDialog from '../ui/CreateTableDialog.vue';
import CreateSchemaDialog from '../ui/CreateSchemaDialog.vue';

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

const confirmDatabaseDelete = async () => {
  if (!props.databaseToDelete) return;
  const { name, connId } = props.databaseToDelete;
  try {
    await gridStore.dropDatabase(connId, name);
  } finally {
    emit('update:databaseToDelete', null);
    emit('closeDatabaseDeleteConfirm');
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
      @cancel="emit('closeDeleteConfirm')"
    />

    <ConfirmDialog
      v-if="tableToDelete"
      title="Delete Table"
      :message="`Are you sure you want to drop table '${tableToDelete?.name}'? This action cannot be undone.`"
      confirm-label="Drop Table"
      variant="danger"
      @confirm="confirmTableDelete"
      @cancel="emit('closeTableDeleteConfirm')"
    />

    <ConfirmDialog
      v-if="schemaToDelete"
      title="Delete Schema"
      :message="`Are you sure you want to drop schema '${schemaToDelete?.name}'? This action will delete all objects within it and cannot be undone.`"
      confirm-label="Drop Schema"
      variant="danger"
      @confirm="confirmSchemaDelete"
      @cancel="emit('closeSchemaDeleteConfirm')"
    />

    <ConfirmDialog
      v-if="databaseToDelete"
      title="Delete Database"
      :message="`Are you sure you want to drop database '${databaseToDelete?.name}'? This action cannot be undone.`"
      confirm-label="Drop Database"
      variant="danger"
      @confirm="confirmDatabaseDelete"
      @cancel="emit('closeDatabaseDeleteConfirm')"
    />

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
  </div>
</template>
