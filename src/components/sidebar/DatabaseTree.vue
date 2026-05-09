<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections';
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';
import { useToastStore } from '@/stores/toast';
import type { Connection } from '@/types';
import { Layers } from 'lucide-vue-next';
import { computed, ref } from 'vue';
import ConfirmDialog from '../ui/ConfirmDialog.vue';
import ConnectionContextMenu from './ConnectionContextMenu.vue';
import ConnectionNode from './ConnectionNode.vue';

const connectionsStore = useConnectionsStore();
const schemaStore = useSchemaStore();
const tabsStore = useTabsStore();
const toastStore = useToastStore();

// ─── Tree Expansion State ─────────────────────────────────────────────────────
const expandedConnections = ref<Record<string, boolean>>({});

const toggleConnection = async (conn: Connection) => {
  const wasExpanded = expandedConnections.value[conn.id];
  expandedConnections.value[conn.id] = !wasExpanded;

  if (!wasExpanded) {
    if (!conn.isConnected) {
      try {
        await connectionsStore.setActiveConnection(conn.id);
      } catch (err: any) {
        expandedConnections.value[conn.id] = false;
        toastStore.addToast({
          title: 'Connection Failed',
          message: err.message,
          severity: 'error',
          variation: 'filled',
          position: 'bottom-center'
        });
      }
    } else {
      if (connectionsStore.activeConnectionId !== conn.id) {
        await connectionsStore.setActiveConnection(conn.id);
      }

      if (!schemaStore.hasSchemaLoaded(conn.id)) {
        schemaStore.loadSchema(conn.displayAllDatabases, conn.id);
      }
    }
  }
};

// ─── Context Menu ─────────────────────────────────────────────────────────────
const contextMenu = ref({ show: false, x: 0, y: 0, connId: null as string | null });
const showDeleteConfirm = ref(false);
const idToDelete = ref<string | null>(null);

const connectionName = computed(
  () => connectionsStore.connections.find((c) => c.id === idToDelete.value)?.name || ''
);

const closeContextMenu = () => {
  contextMenu.value.show = false;
  contextMenu.value.connId = null;
};

const onContextMenu = (event: MouseEvent, id: string) => {
  let x = event.clientX;
  let y = event.clientY;
  if (x + 200 > window.innerWidth) x -= 200;
  if (y + 200 > window.innerHeight) y -= 200;
  contextMenu.value = { show: true, x, y, connId: id };
};

const handleContextAction = (action: string) => {
  const id = contextMenu.value.connId;
  closeContextMenu();
  if (!id) return;

  if (action === 'sql') {
    tabsStore.openSqlEditor(id);
  } else if (action === 'edit') {
    const conn = connectionsStore.connections.find((c) => c.id === id);
    if (conn) connectionsStore.toggleConnectionModal(true, conn);
  } else if (action === 'duplicate') {
    const conn = connectionsStore.connections.find((c) => c.id === id);
    if (conn) {
      const newConn = {
        ...conn,
        id: crypto.randomUUID(),
        name: `${conn.name} (Copy)`,
        isConnected: false
      };
      connectionsStore.connections.push(newConn);
      connectionsStore.saveConnections();
    }
  } else if (action === 'refresh') {
    schemaStore.loadSchema(undefined, id);
  } else if (action === 'delete') {
    idToDelete.value = id;
    showDeleteConfirm.value = true;
  }
};

const confirmDelete = () => {
  if (!idToDelete.value) return;
  const id = idToDelete.value;
  const idx = connectionsStore.connections.findIndex((c) => c.id === id);
  if (idx !== -1) {
    connectionsStore.connections.splice(idx, 1);
    if (connectionsStore.activeConnectionId === id) {
      connectionsStore.activeConnectionId = null;
    }
    connectionsStore.saveConnections();
    delete schemaStore.schemasByConnection[id];
    toastStore.addToast({
      message: 'Connection deleted.',
      severity: 'success',
      variation: 'subtle'
    });
  }
  showDeleteConfirm.value = false;
  idToDelete.value = null;
};
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col select-none">
    <div class="flex-1 overflow-y-auto py-1">
      <ConnectionNode
        v-for="conn in connectionsStore.connections"
        :key="conn.id"
        :connection="conn"
        :is-expanded="!!expandedConnections[conn.id]"
        @toggle="toggleConnection(conn)"
        @contextmenu="onContextMenu($event, conn.id)"
      />

      <!-- Empty state -->
      <div
        v-if="connectionsStore.connections.length === 0"
        class="flex flex-col items-center justify-center gap-3 px-4 py-12"
      >
        <Layers :size="32" class="text-text-tertiary opacity-50" />
        <p class="text-text-tertiary text-center text-[12px]">
          No connections configured.<br />Click <strong>+</strong> to add one.
        </p>
      </div>
    </div>

    <!-- Context Menu -->
    <ConnectionContextMenu
      :show="contextMenu.show"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
      @action="handleContextAction"
    />

    <!-- Delete Confirmation -->
    <ConfirmDialog
      v-if="showDeleteConfirm"
      title="Delete Connection"
      :message="`Are you sure you want to delete '${connectionName}'? This action cannot be undone.`"
      confirm-label="Delete"
      variant="danger"
      @confirm="confirmDelete"
      @cancel="showDeleteConfirm = false"
    />
  </div>
</template>
