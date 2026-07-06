import { OracleConnectType, OracleRole, type Connection } from '@/types';
import * as Neutralino from '@neutralinojs/lib';
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

import { NativeService } from '@/services/native';
import { decryptPassword, encryptPassword } from '@/utils/crypto';
import { useToastStore } from './toast';

export const useConnectionsStore = defineStore('connections', () => {
  const connections = ref<Connection[]>([]);
  const activeConnectionId = ref<string | null>(null);
  const showNewConnectionModal = ref(false);
  const showExportModal = ref(false);
  const preSelectedExportId = ref<string | null>(null);
  const showImportModal = ref(false);
  const importedConnections = ref<Connection[]>([]);

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
      } catch {
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

  const toggleExportModal = (show?: boolean, connId?: string) => {
    showExportModal.value = show ?? !showExportModal.value;
    preSelectedExportId.value = connId ?? null;
  };

  const toggleImportModal = (show?: boolean) => {
    showImportModal.value = show ?? !showImportModal.value;
    if (!showImportModal.value) {
      importedConnections.value = [];
    }
  };

  const exportConnections = async (selectedIds: string[], includePasswords: boolean) => {
    if (!window.NL_PORT) return;
    const toastStore = useToastStore();

    try {
      const filePath = await NativeService.os.showSaveDialog('Export Connections Profile', {
        filters: [{ name: 'JSON files', extensions: ['json'] }]
      });

      if (!filePath) return;

      const toExport = connections.value
        .filter((c) => selectedIds.includes(c.id))
        .map((c) => ({
          ...c,
          password: includePasswords ? encryptPassword(c.password) : ''
        }));

      await NativeService.fs.writeFile(filePath, JSON.stringify(toExport, null, 2));

      toastStore.addToast({
        severity: 'success',
        title: 'Export Successful',
        message: `Exported ${toExport.length} connection(s) to ${filePath}`
      });
    } catch (err: any) {
      toastStore.addToast({
        severity: 'error',
        title: 'Export Failed',
        message: err.message
      });
    }
  };

  const selectImportFile = async () => {
    if (!window.NL_PORT) return;
    const toastStore = useToastStore();

    try {
      const filePaths = await NativeService.os.showOpenDialog('Import Connection Profile', {
        filters: [{ name: 'JSON files', extensions: ['json'] }],
        multiSelections: false
      });

      if (!filePaths || filePaths.length === 0 || !filePaths[0]) return;

      const content = await NativeService.fs.readFile(filePaths[0]);
      if (!content) return;

      const parsed = JSON.parse(content) as Partial<Connection>[];
      if (!Array.isArray(parsed)) {
        throw new Error('Invalid JSON format: Expected an array of connections.');
      }

      // Validate basic connection structure
      const validConnections: Connection[] = [];
      for (const item of parsed) {
        if (!item.id || !item.name || !item.type || !item.host) {
          throw new Error(
            'Invalid JSON format: Missing required connection fields (id, name, type, host).'
          );
        }
        item.isConnected = false;
        // Passwords are left alone here, they will be decrypted when imported
        validConnections.push(item as Connection);
      }

      importedConnections.value = validConnections;
      toggleImportModal(true);
    } catch (err: any) {
      toastStore.addToast({
        severity: 'error',
        title: 'Import Failed',
        message: err.message
      });
    }
  };

  const importConnections = async (
    items: { connection: Connection; selected: boolean; resolution: 'copy' | 'overwrite' }[]
  ) => {
    const toastStore = useToastStore();
    let importedCount = 0;

    for (const item of items) {
      if (!item.selected) continue;

      const incoming = { ...item.connection };
      incoming.password = decryptPassword(incoming.password);

      if (item.resolution === 'overwrite') {
        const existingIndex = connections.value.findIndex(
          (c) => c.id === incoming.id || c.name === incoming.name
        );
        if (existingIndex !== -1) {
          connections.value[existingIndex] = incoming;
        } else {
          connections.value.push(incoming);
        }
      } else {
        // copy resolution
        const nameExists = connections.value.some((c) => c.name === incoming.name);
        const idExists = connections.value.some((c) => c.id === incoming.id);

        if (nameExists || idExists) {
          incoming.id = generateId();

          if (nameExists) {
            // Find a unique name
            let nameSuffix = 1;
            let originalName = incoming.name;
            // Remove existing copy suffix if present
            const copyMatch = originalName.match(/ \(Copy(?: \d+)?\)$/);
            if (copyMatch) {
              originalName = originalName.substring(0, originalName.length - copyMatch[0].length);
            }

            let newName = `${originalName} (Copy)`;
            while (connections.value.some((c) => c.name === newName)) {
              nameSuffix++;
              newName = `${originalName} (Copy ${nameSuffix})`;
            }
            incoming.name = newName;
          }
        }

        connections.value.push(incoming);
      }
      importedCount++;
    }

    await saveConnections();
    toggleImportModal(false);

    toastStore.addToast({
      severity: 'success',
      title: 'Import Successful',
      message: `Imported ${importedCount} connection(s).`
    });
  };

  const updateConnection = async (id: string, updates: Partial<Connection>) => {
    const conn = connections.value.find((c) => c.id === id);
    if (conn) {
      const credentialsChanged =
        (updates.host !== undefined && updates.host !== conn.host) ||
        (updates.port !== undefined && updates.port !== conn.port) ||
        (updates.database !== undefined && updates.database !== conn.database) ||
        (updates.username !== undefined && updates.username !== conn.username) ||
        (updates.password !== undefined && updates.password !== conn.password) ||
        (updates.type !== undefined && updates.type !== conn.type) ||
        (updates.oracleConnectType !== undefined &&
          updates.oracleConnectType !== conn.oracleConnectType) ||
        (updates.oracleRole !== undefined && updates.oracleRole !== conn.oracleRole);

      Object.assign(conn, updates);
      await saveConnections();

      // If the connection is active and connected, invalidate + reload its schema
      // so settings like displayAllDatabases take effect immediately.
      if (conn.isConnected && activeConnectionId.value === id) {
        const { useSchemaStore } = await import('./schema');
        const schemaStore = useSchemaStore();
        // Clear the cached schema so the tree reloads cleanly
        delete schemaStore.schemasByConnection[id];

        if (credentialsChanged) {
          try {
            await setActiveConnection(id);
          } catch (err) {
            console.error('Failed to reconnect after credentials update:', err);
          }
        } else {
          schemaStore.loadSchema(conn.displayAllDatabases, id);
        }
      }
    }
  };

  const disconnectConnection = async (id: string) => {
    const conn = connections.value.find((c) => c.id === id);
    if (!conn) return;

    if (window.NL_PORT) {
      const { BridgeService } = await import('@/services/bridge');
      try {
        await BridgeService.request('dbBridge.disconnect', 'dbBridge.disconnectResult', {
          connectionId: id
        });
      } catch (error: any) {
        console.error('Failed to disconnect on backend:', error.message);
      }
    }

    conn.isConnected = false;

    if (activeConnectionId.value === id) {
      const nextConnected = connections.value.find((c) => c.isConnected && c.id !== id);
      activeConnectionId.value = nextConnected?.id ?? null;
    }

    const { useSchemaStore } = await import('./schema');
    const schemaStore = useSchemaStore();
    schemaStore.removeConnection(id);

    const toastStore = useToastStore();
    toastStore.addToast({
      severity: 'success',
      title: 'Disconnected',
      message: `Successfully disconnected from ${conn.name}`
    });
  };

  return {
    connections,
    activeConnectionId,
    activeConnection,
    connectedConnections,
    showNewConnectionModal,
    showExportModal,
    preSelectedExportId,
    showImportModal,
    importedConnections,
    connectionToEdit,
    loadConnections,
    saveConnections,
    setActiveConnection,
    disconnectConnection,
    addConnection,
    removeConnection,
    updateConnection,
    toggleConnectionModal,
    toggleExportModal,
    toggleImportModal,
    exportConnections,
    selectImportFile,
    importConnections,
    generateId
  };
});
