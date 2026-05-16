// src/composables/useTabSync.ts
import { watch } from 'vue';
import { useTabsStore } from '@/stores/tabs';
import { useConnectionsStore } from '@/stores/connections';
import { useSchemaStore } from '@/stores/schema';
import { useGridStore } from '@/stores/grid';
import { useToastStore } from '@/stores/toast';
import { DbType, TabType } from '@/types';

export function useTabSync() {
  const tabsStore = useTabsStore();
  const connectionsStore = useConnectionsStore();
  const schemaStore = useSchemaStore();
  const gridStore = useGridStore();
  const toastStore = useToastStore();

  watch(
    () => [tabsStore.activeTab, connectionsStore.connections.length] as const,
    async ([tab, connCount]) => {
      if (!tab || connCount === 0) return;

      // Sync UI to tab's connection context
      if (tab.connectionId) {
        const conn = connectionsStore.connections.find((c) => c.id === tab.connectionId);

        // Ensure we are connected to this database in the bridge
        if (!conn?.isConnected || tab.connectionId !== connectionsStore.activeConnectionId) {
          try {
            await connectionsStore.setActiveConnection(tab.connectionId);
          } catch (err: any) {
            console.error('Failed to sync connection for tab:', err);
            toastStore.addToast({
              severity: 'error',
              title: 'Connection Failed',
              message: err.message || 'Could not connect to the database for the active tab.'
            });
            return;
          }
        }

        // Sync schema selection for this specific connection
        if (tab.schema) {
          const currentSelected = schemaStore.selectedSchemaByConnection[tab.connectionId];
          if (tab.schema !== currentSelected) {
            schemaStore.setSelectedSchema(tab.schema, tab.connectionId);
            if (conn?.type === DbType.ORACLE) {
              await schemaStore.loadSchema(
                schemaStore.loadedAllDatabases,
                tab.connectionId,
                tab.schema
              );
            }
          }
        }
      }

      if (tab.type === TabType.TABLE && tab.tableName) {
        gridStore.isLoading = true;
        gridStore.loadTable(tab.tableName, tab.connectionId, tab.schema, tab.dbName);
      }
    },
    { immediate: true }
  );
}
