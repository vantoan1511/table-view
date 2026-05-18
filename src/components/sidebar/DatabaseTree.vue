<script setup lang="ts">
import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';
import { useToastStore } from '@/stores/toast';
import type { Connection } from '@/types';
import { Layers } from 'lucide-vue-next';
import { provide, ref } from 'vue';
import ConnectionContextMenu from './ConnectionContextMenu.vue';
import ConnectionNode from './ConnectionNode.vue';
import DatabaseContextMenu from './DatabaseContextMenu.vue';
import SchemaContextMenu from './SchemaContextMenu.vue';
import SidebarDialogs from './SidebarDialogs.vue';
import TableContextMenu from './TableContextMenu.vue';

const connectionsStore = useConnectionsStore();
const gridStore = useGridStore();
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
      } catch {
        // setActiveConnection already shows a toast — just collapse the node
        expandedConnections.value[conn.id] = false;
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
type EntityType = 'connection' | 'database' | 'schema' | 'table';

const contextMenu = ref({
  show: false,
  x: 0,
  y: 0,
  type: null as EntityType | null,
  connId: null as string | null,
  dbName: null as string | null,
  schemaName: null as string | null,
  tableName: null as string | null
});

const showDeleteConfirm = ref(false);
const showTableDeleteConfirm = ref(false);
const showSchemaDeleteConfirm = ref(false);
const showDatabaseDeleteConfirm = ref(false);

const idToDelete = ref<string | null>(null);
const tableToDelete = ref<{
  name: string;
  connId: string;
  dbName?: string;
  schemaName?: string;
} | null>(null);

const schemaToDelete = ref<{
  name: string;
  connId: string;
  dbName?: string;
} | null>(null);

const databaseToDelete = ref<{
  name: string;
  connId: string;
} | null>(null);

const closeContextMenu = () => {
  contextMenu.value.show = false;
  contextMenu.value.type = null;
  contextMenu.value.connId = null;
};

const onEntityContextMenu = (
  event: MouseEvent,
  type: EntityType,
  payload: { connId: string; dbName?: string; schemaName?: string; tableName?: string }
) => {
  let x = event.clientX;
  let y = event.clientY;
  if (x + 200 > window.innerWidth) x -= 200;
  if (y + 200 > window.innerHeight) y -= 200;
  contextMenu.value = {
    show: true,
    x,
    y,
    type,
    connId: payload.connId,
    dbName: payload.dbName ?? null,
    schemaName: payload.schemaName ?? null,
    tableName: payload.tableName ?? null
  };
};

provide('onEntityContextMenu', onEntityContextMenu);

const onContextMenu = (event: MouseEvent, id: string) => {
  onEntityContextMenu(event, 'connection', { connId: id });
};

const handleContextAction = async (action: string) => {
  const { type, connId, dbName, schemaName, tableName } = contextMenu.value;
  closeContextMenu();
  if (!connId || !type) return;

  if (type === 'connection') {
    if (action === 'sql') {
      tabsStore.openSqlEditor(connId);
    } else if (action === 'edit') {
      const conn = connectionsStore.connections.find((c) => c.id === connId);
      if (conn) connectionsStore.toggleConnectionModal(true, conn);
    } else if (action === 'duplicate') {
      const conn = connectionsStore.connections.find((c) => c.id === connId);
      if (conn) {
        const newConn = {
          ...conn,
          id: crypto.randomUUID(),
          name: `${conn.name} (Copy)`,
          isConnected: false
        };
        connectionsStore.addConnection(newConn);
      }
    } else if (action === 'refresh') {
      schemaStore.loadSchema(false, connId);
    } else if (action === 'createDatabase') {
      gridStore.createDatabaseTarget = { connectionId: connId };
      gridStore.showCreateDatabaseDialog = true;
    } else if (action === 'delete') {
      idToDelete.value = connId;
      showDeleteConfirm.value = true;
    }
  } else if (type === 'database') {
    if (action === 'refresh' && dbName) {
      schemaStore.refreshDbSchema(connId, dbName);
    } else if (action === 'createSchema') {
      gridStore.createSchemaTarget = {
        connectionId: connId,
        db: dbName || undefined
      };
      gridStore.showCreateSchemaDialog = true;
    } else if (action === 'deleteDatabase' && dbName) {
      databaseToDelete.value = {
        name: dbName,
        connId
      };
      showDatabaseDeleteConfirm.value = true;
    }
  } else if (type === 'schema') {
    if (action === 'refresh') {
      if (dbName) {
        await schemaStore.loadDbSchema(connId, dbName, true);
      } else {
        await schemaStore.loadSchema(false, connId, schemaName || undefined);
      }
    } else if (action === 'createTable') {
      gridStore.createTableTarget = {
        connectionId: connId,
        schema: schemaName || '',
        db: dbName || undefined
      };
      gridStore.showCreateTableDialog = true;
    } else if (action === 'createSchema') {
      gridStore.createSchemaTarget = {
        connectionId: connId,
        db: dbName || undefined
      };
      gridStore.showCreateSchemaDialog = true;
    } else if (action === 'deleteSchema' && schemaName) {
      schemaToDelete.value = {
        name: schemaName,
        connId,
        dbName: dbName || undefined
      };
      showSchemaDeleteConfirm.value = true;
    }
  } else if (type === 'table') {
    if (action === 'refresh' && tableName) {
      if (
        tabsStore.activeTab?.type === 'table' &&
        tabsStore.activeTab.tableName === tableName &&
        tabsStore.activeTab.connectionId === connId
      ) {
        await gridStore.loadTable(tableName);
      } else {
        if (dbName) {
          await schemaStore.loadDbSchema(connId, dbName, true);
        } else {
          await schemaStore.loadSchema(false, connId);
        }
      }
    } else if (action === 'alterTable' && tableName) {
      tabsStore.openTableTab(tableName, schemaName || undefined, connId, dbName || undefined);
      gridStore.showAlterTableDialog = true;
    } else if (action === 'deleteTable' && tableName) {
      tableToDelete.value = {
        name: tableName,
        connId,
        dbName: dbName || undefined,
        schemaName: schemaName || undefined
      };
      showTableDeleteConfirm.value = true;
    }
  }
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

    <!-- Context Menus -->
    <ConnectionContextMenu
      v-if="contextMenu.type === 'connection'"
      :show="contextMenu.show"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
      @action="handleContextAction"
    />
    <DatabaseContextMenu
      v-if="contextMenu.type === 'database'"
      :show="contextMenu.show"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
      @action="handleContextAction"
    />
    <SchemaContextMenu
      v-if="contextMenu.type === 'schema'"
      :show="contextMenu.show"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
      @action="handleContextAction"
    />
    <TableContextMenu
      v-if="contextMenu.type === 'table'"
      :show="contextMenu.show"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
      @action="handleContextAction"
    />

    <!-- Dialogs -->
    <SidebarDialogs
      v-model:id-to-delete="idToDelete"
      v-model:table-to-delete="tableToDelete"
      v-model:schema-to-delete="schemaToDelete"
      v-model:database-to-delete="databaseToDelete"
      @close-delete-confirm="showDeleteConfirm = false"
      @close-table-delete-confirm="showTableDeleteConfirm = false"
      @close-schema-delete-confirm="showSchemaDeleteConfirm = false"
      @close-database-delete-confirm="showDatabaseDeleteConfirm = false"
    />
  </div>
</template>
