import { OracleConnectType, OracleRole, type Connection } from '@/types';
import * as Neutralino from '@neutralinojs/lib';
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

import { useToastStore } from './toast';
import { decryptPassword, encryptPassword } from '@/utils/crypto';

export const useConnectionsStore = defineStore('connections', () => {
  const connections = ref<Connection[]>([]);
  const activeConnectionId = ref<string | null>(null);
  const showNewConnectionModal = ref(false);

  const activeConnection = computed(
    () => connections.value.find((c) => c.id === activeConnectionId.value) ?? null
  );

  const connectedConnections = computed(() => connections.value.filter((c) => c.isConnected));

  const loadConnections = async () => {
    if (window.NL_PORT) {
      try {
        const data = await Neutralino.storage.getData('connections');
        const loaded = JSON.parse(data) as Connection[];
        // Decrypt passwords and reset connection states on boot
        loaded.forEach((c) => {
          c.password = decryptPassword(c.password);
          c.isConnected = false;
          c.oracleConnectType = c.oracleConnectType || OracleConnectType.SERVICE_NAME;
          c.oracleRole = c.oracleRole || OracleRole.NORMAL;
        });
        connections.value = loaded;
      } catch (err) {
        // Storage not found, initialize empty
        connections.value = [];
      }
    }
  };

  const saveConnections = async () => {
    if (window.NL_PORT) {
      // Encrypt passwords before persisting
      const toSave = connections.value.map((c) => ({
        ...c,
        password: encryptPassword(c.password)
      }));
      await Neutralino.storage.setData('connections', JSON.stringify(toSave));
    }
  };

  const setActiveConnection = async (id: string): Promise<any> => {
    const toastStore = useToastStore();
    // 1. Update active ID immediately so UI reflects intended state
    const previousActiveConnectionId = activeConnectionId.value;
    activeConnectionId.value = id;

    const conn = connections.value.find((c) => c.id === id);
    if (!conn) return;

    if (window.NL_PORT) {
      const { BridgeService } = await import('@/services/bridge');
      try {
        await BridgeService.request('dbBridge.connect', 'dbBridge.connectResult', {
          connectionId: id,
          config: conn
        });

        conn.isConnected = true;
        // Import dynamically to avoid circular dependency
        const { useSchemaStore } = await import('./schema');
        const schemaStore = useSchemaStore();
        schemaStore.setSelectedSchema('');
        schemaStore.loadSchema(conn.displayAllDatabases, id);
        return true;
      } catch (error: any) {
        conn.isConnected = false;
        // Rollback if this specific connection attempt failed
        activeConnectionId.value = previousActiveConnectionId;
        toastStore.addToast({
          severity: 'error',
          title: 'Connection Failed',
          message: error.message
        });
        console.error('Failed to connect:', error.message);
        throw error;
      }
    }
  };

  const addConnection = async (conn: Connection) => {
    connections.value.push(conn);
    await saveConnections();
  };

  const removeConnection = async (id: string) => {
    connections.value = connections.value.filter((c) => c.id !== id);
    if (activeConnectionId.value === id) {
      activeConnectionId.value = connections.value[0]?.id ?? null;
    }
    await saveConnections();
  };

  const connectionToEdit = ref<Connection | null>(null);

  const toggleConnectionModal = (show?: boolean, conn?: Connection) => {
    showNewConnectionModal.value = show ?? !showNewConnectionModal.value;
    connectionToEdit.value = conn ?? null;
  };

  const generateId = (): string => {
    return `conn-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  };

  const updateConnection = async (id: string, updates: Partial<Connection>) => {
    const conn = connections.value.find((c) => c.id === id);
    if (conn) {
      Object.assign(conn, updates);
      await saveConnections();

      // If the connection is active and connected, invalidate + reload its schema
      // so settings like displayAllDatabases take effect immediately.
      if (conn.isConnected && activeConnectionId.value === id) {
        const { useSchemaStore } = await import('./schema');
        const schemaStore = useSchemaStore();
        // Clear the cached schema so the tree reloads cleanly
        delete schemaStore.schemasByConnection[id];
        schemaStore.loadSchema(conn.displayAllDatabases, id);
      }
    }
  };

  return {
    connections,
    activeConnectionId,
    activeConnection,
    connectedConnections,
    showNewConnectionModal,
    connectionToEdit,
    loadConnections,
    saveConnections,
    setActiveConnection,
    addConnection,
    removeConnection,
    updateConnection,
    toggleConnectionModal,
    generateId
  };
});
