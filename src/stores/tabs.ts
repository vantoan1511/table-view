import { TabType, type Tab } from '@/types';
import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';

import { NativeService } from '@/services/nativeService';
import { useConnectionsStore } from './connections';
import { useSchemaStore } from './schema';
import { useTabPersistence } from './tabs/useTabPersistence';
import { useToastStore } from './toast';

export const useTabsStore = defineStore('tabs', () => {
  const connectionsStore = useConnectionsStore();
  const schemaStore = useSchemaStore();
  const toastStore = useToastStore();

  const tabs = ref<Tab[]>([]);
  const activeTabId = ref<string>('');
  const draggingTabId = ref<string | null>(null);
  const isAppClosing = ref(false);
  const showTabSelector = ref(false);
  const selectorConnectionId = ref<string | null>(null);

  const persistence = useTabPersistence(tabs, activeTabId);

  watch(activeTabId, () => persistence.saveTabsToStorage());

  const activeTab = computed(() => tabs.value.find((t) => t.id === activeTabId.value) ?? null);
  const mainTabs = computed(() => tabs.value.filter((t) => !t.minimized && !t.closed));
  const minimizedTabs = computed(() => tabs.value.filter((t) => t.minimized && !t.closed));

  const setActiveTab = (id: string) => {
    activeTabId.value = id;
  };

  const minimizeTab = (id: string) => {
    const tab = tabs.value.find((t) => t.id === id);
    if (tab && !tab.minimized) {
      tab.minimized = true;
      if (activeTabId.value === id) {
        const next = mainTabs.value[mainTabs.value.length - 1];
        activeTabId.value = next?.id ?? '';
      }
    }
  };

  const restoreTab = (id: string) => {
    const tab = tabs.value.find((t) => t.id === id);
    if (tab && tab.minimized) {
      tab.minimized = false;
      activeTabId.value = tab.id;
    }
  };

  const reorderTab = (fromId: string, toId: string) => {
    const fromIdx = tabs.value.findIndex((t) => t.id === fromId);
    const toIdx = tabs.value.findIndex((t) => t.id === toId);
    if (fromIdx !== -1 && toIdx !== -1 && fromIdx !== toIdx) {
      const removed = tabs.value.splice(fromIdx, 1);
      const movedTab = removed[0];
      if (movedTab) {
        tabs.value.splice(toIdx, 0, movedTab);
      }
    }
  };

  const renameTab = (id: string, newTitle: string) => {
    const tab = tabs.value.find((t) => t.id === id && t.type === TabType.SQL);
    if (tab) {
      tab.title = newTitle;
    }
  };

  const openTableTab = (
    tableName: string,
    schemaName?: string,
    connectionId?: string,
    dbName?: string
  ) => {
    const targetSchema = schemaName || schemaStore.selectedSchema;
    const targetConnectionId = connectionId ?? connectionsStore.activeConnectionId ?? undefined;
    const cached = tabs.value.find(
      (t) =>
        t.type === TabType.TABLE &&
        t.tableName === tableName &&
        t.schema === schemaName &&
        t.connectionId === connectionId &&
        t.dbName === dbName
    );
    if (cached) {
      cached.closed = false;
      cached.minimized = false;
      activeTabId.value = cached.id;
      return;
    }
    const tab: Tab = {
      id: `tab-${targetSchema}-${tableName}-${Date.now()}`,
      type: TabType.TABLE,
      title: targetSchema ? `${targetSchema}.${tableName}` : tableName,
      tableName,
      connectionId: targetConnectionId,
      schema: targetSchema,
      dbName
    };
    tabs.value.push(tab);
    activeTabId.value = tab.id;
  };

  const openDiagramTab = (schemaName: string, connectionId?: string, dbName?: string) => {
    const targetConnectionId = connectionId ?? connectionsStore.activeConnectionId ?? undefined;
    const cached = tabs.value.find(
      (t) =>
        t.type === TabType.DIAGRAM &&
        t.schema === schemaName &&
        t.connectionId === connectionId &&
        t.dbName === dbName
    );
    if (cached) {
      cached.closed = false;
      cached.minimized = false;
      activeTabId.value = cached.id;
      return;
    }
    const tab: Tab = {
      id: `tab-diagram-${schemaName}-${Date.now()}`,
      type: TabType.DIAGRAM,
      title: `${schemaName} (Diagram)`,
      connectionId: targetConnectionId,
      schema: schemaName,
      dbName
    };
    tabs.value.push(tab);
    activeTabId.value = tab.id;
  };

  const openIndexTab = (
    indexName: string,
    tableName: string,
    schemaName?: string,
    connectionId?: string,
    dbName?: string
  ) => {
    const targetSchema = schemaName || schemaStore.selectedSchema;
    const targetConnectionId = connectionId ?? connectionsStore.activeConnectionId ?? undefined;
    const cached = tabs.value.find(
      (t) =>
        t.type === TabType.INDEX &&
        t.indexName === indexName &&
        t.tableName === tableName &&
        t.schema === schemaName &&
        t.connectionId === connectionId &&
        t.dbName === dbName
    );
    if (cached) {
      cached.closed = false;
      cached.minimized = false;
      activeTabId.value = cached.id;
      return;
    }
    const tab: Tab = {
      id: `tab-index-${targetSchema}-${tableName}-${indexName}-${Date.now()}`,
      type: TabType.INDEX,
      title: `${tableName}: ${indexName}`,
      indexName,
      tableName,
      connectionId: targetConnectionId,
      schema: targetSchema,
      dbName
    };
    tabs.value.push(tab);
    activeTabId.value = tab.id;
  };

  const getNextEditorNumber = (connectionName: string) => {
    const prefix = `${connectionName}-`;
    const existingNumbers = tabs.value
      .filter((t) => t.title.startsWith(prefix))
      .map((t) => {
        const numPart = t.title.substring(prefix.length);
        const n = parseInt(numPart);
        return isNaN(n) ? 0 : n;
      });
    return existingNumbers.length > 0 ? Math.max(...existingNumbers) + 1 : 1;
  };

  const openSqlEditor = (
    connectionId?: string,
    query: string = '',
    isDraft: boolean = true,
    forceNew: boolean = false,
    dbName?: string
  ) => {
    const connId = connectionId || connectionsStore.activeConnectionId || undefined;

    if (!forceNew && connId) {
      const existingTabs = tabs.value.filter(
        (t) => t.type === 'sql' && t.connectionId === connId && t.dbName === dbName
      );
      if (existingTabs.length > 1) {
        selectorConnectionId.value = connId;
        showTabSelector.value = true;
        return;
      } else if (existingTabs.length === 1) {
        const tab = existingTabs[0];
        if (tab) {
          tab.closed = false;
          tab.minimized = false;
          activeTabId.value = tab.id;
          if (query) tab.query = query;
        }
        return;
      }
    }

    const connection = connectionsStore.connections.find((c) => c.id === connId);
    const connName = connection?.name || 'query';
    const nextNum = getNextEditorNumber(connName);

    const tab: Tab = {
      id: `tab-sql-${Date.now()}`,
      type: TabType.SQL,
      title: `${connName}-${nextNum}`,
      connectionId: connId,
      schema: schemaStore.selectedSchema,
      query,
      isDraft,
      isDirty: false
    };
    tabs.value.push(tab);
    activeTabId.value = tab.id;
  };

  const updateTabQuery = (id: string, query: string) => {
    const tab = tabs.value.find((t) => t.id === id);
    if (tab && tab.type === 'sql') {
      if (tab.query !== query) {
        tab.query = query;
        tab.isDirty = true;
      }
    }
  };

  const updateTabAutoCommit = (id: string, autoCommit: boolean) => {
    const tab = tabs.value.find((t) => t.id === id);
    if (tab && tab.type === 'sql') {
      tab.autoCommit = autoCommit;
    }
  };

  const saveSqlTab = (id: string) => {
    const tab = tabs.value.find((t) => t.id === id);
    if (tab && tab.type === 'sql') {
      tab.isDirty = false;
    }
  };

  const exportSqlTab = async (id: string) => {
    const tab = tabs.value.find((t) => t.id === id);
    if (!tab || tab.type !== 'sql') return;

    const res = await NativeService.os.showSaveDialog('Export SQL Query', {
      filters: [{ name: 'SQL Files', extensions: ['sql'] }]
    });
    if (!res) return;

    try {
      await NativeService.fs.writeFile(res, tab.query || '');
      tab.filePath = res;
      tab.isDirty = false;
      const filename = res.split(/[\\/]/).pop() || tab.title;
      tab.title = filename.replace(/\.sql$/i, '');
    } catch (err: any) {
      console.error('Failed to export SQL file:', err);
      toastStore.addToast({
        severity: 'error',
        title: 'Export Failed',
        message: err.message || 'Could not save the SQL file.'
      });
    }
  };

  const closeTab = (id: string) => {
    const tab = tabs.value.find((t) => t.id === id);
    if (tab) {
      tab.closed = true;
      tab.minimized = false;
      if (activeTabId.value === id) {
        const next = mainTabs.value[mainTabs.value.length - 1];
        activeTabId.value = next?.id ?? '';
      }
    }
  };

  const deleteTab = (id: string) => {
    const idx = tabs.value.findIndex((t) => t.id === id && t.type === TabType.SQL);
    if (idx !== -1) {
      tabs.value.splice(idx, 1);
      if (activeTabId.value === id) {
        const next = mainTabs.value[mainTabs.value.length - 1];
        activeTabId.value = next?.id ?? '';
      }
    }
  };

  const getById = (tabId?: string) => {
    if (!tabId) return null;
    return tabs.value.find((t) => t.id === tabId);
  };

  return {
    tabs,
    activeTabId,
    activeTab,
    draggingTabId,
    isAppClosing,
    mainTabs,
    minimizedTabs,
    setActiveTab,
    reorderTab,
    openTableTab,
    openDiagramTab,
    openIndexTab,
    openSqlEditor,
    updateTabQuery,
    updateTabAutoCommit,
    saveSqlTab,
    exportSqlTab,
    renameTab,
    loadTabsFromStorage: persistence.loadTabsFromStorage,
    closeTab,
    deleteTab,
    minimizeTab,
    restoreTab,
    getById,
    showTabSelector,
    selectorConnectionId
  };
});
