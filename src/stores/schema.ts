import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { useConnectionsStore } from './connections';
import { useSchemaActions } from './schema/useSchemaActions';
import { useSchemaCache } from './schema/useSchemaCache';
import { useTreeState } from './schema/useTreeState';

export const useSchemaStore = defineStore('schema', () => {
  const connectionsStore = useConnectionsStore();

  const cache = useSchemaCache();
  const treeState = useTreeState();

  // Legacy global state that actions need
  const loadedAllDatabases = ref(false);

  const actions = useSchemaActions(cache, treeState, connectionsStore, loadedAllDatabases);

  // ─── Active-connection shims (for backward compat) ───────────────────────────
  const activeConnectionId = computed(() => connectionsStore.activeConnectionId);

  const schema = computed(() => {
    const id = activeConnectionId.value;
    return id ? (cache.schemasByConnection.value[id] ?? cache.emptySchema()) : cache.emptySchema();
  });

  const selectedSchema = computed(() => {
    const id = activeConnectionId.value;
    return id ? (treeState.selectedSchemaByConnection.value[id] ?? '') : '';
  });

  // ─── Per-connection filtered lists ───────────────────────────────────────────
  const getFilteredTables = (connectionId: string, schemaName?: string, dbName?: string) => {
    const s = dbName
      ? cache.perDbSchemas.value[connectionId]?.[dbName]
      : cache.schemasByConnection.value[connectionId];

    if (!s) return [];
    const targetSchema =
      schemaName ?? treeState.selectedSchemaByConnection.value[connectionId] ?? '';
    const q = treeState.filterQuery.value.toLowerCase();
    return s.tables.filter(
      (t) => actions.sameSchema(t.schema, targetSchema) && t.name.toLowerCase().includes(q)
    );
  };

  const getFilteredViews = (connectionId: string, schemaName?: string, dbName?: string) => {
    const s = dbName
      ? cache.perDbSchemas.value[connectionId]?.[dbName]
      : cache.schemasByConnection.value[connectionId];

    if (!s) return [];
    const targetSchema =
      schemaName ?? treeState.selectedSchemaByConnection.value[connectionId] ?? '';
    const q = treeState.filterQuery.value.toLowerCase();
    return s.views.filter(
      (v) => actions.sameSchema(v.schema, targetSchema) && v.name.toLowerCase().includes(q)
    );
  };

  const getFilteredFunctions = (connectionId: string, schemaName?: string, dbName?: string) => {
    const s = dbName
      ? cache.perDbSchemas.value[connectionId]?.[dbName]
      : cache.schemasByConnection.value[connectionId];

    if (!s) return [];
    const targetSchema =
      schemaName ?? treeState.selectedSchemaByConnection.value[connectionId] ?? '';
    const q = treeState.filterQuery.value.toLowerCase();
    return s.functions.filter(
      (f) => actions.sameSchema(f.schema, targetSchema) && f.name.toLowerCase().includes(q)
    );
  };

  // Shim for legacy computed consumers
  const filteredTables = computed(() => {
    const id = activeConnectionId.value;
    return id ? getFilteredTables(id) : [];
  });

  const filteredViews = computed(() => {
    const id = activeConnectionId.value;
    return id ? getFilteredViews(id) : [];
  });

  const filteredFunctions = computed(() => {
    const id = activeConnectionId.value;
    return id ? getFilteredFunctions(id) : [];
  });

  // ─── Mutations / Getters wrapper ──────────────────────────────────────────────
  const setSelectedSchema = (s: string, connectionId?: string) => {
    const id = connectionId ?? activeConnectionId.value;
    if (!id) return;
    treeState.selectedSchemaByConnection.value[id] = s;
  };

  const isConnectionLoading = (connectionId: string) => {
    return cache.loadingByConnection.value[connectionId] ?? false;
  };

  const isDbLoading = (connectionId: string, dbName: string) => {
    return cache.loadingDbByConnection.value[connectionId]?.[dbName] ?? false;
  };

  const getDbError = (connectionId: string, dbName: string): string | null => {
    return cache.errorDbByConnection.value[connectionId]?.[dbName] ?? null;
  };

  return {
    // State from cache
    schemasByConnection: cache.schemasByConnection,
    loadingByConnection: cache.loadingByConnection,
    perDbSchemas: cache.perDbSchemas,
    loadingDbByConnection: cache.loadingDbByConnection,
    errorDbByConnection: cache.errorDbByConnection,
    hasSchemaLoaded: cache.hasSchemaLoaded,
    getDbSchema: cache.getDbSchema,
    getDbError,

    // State from treeState
    expandedDbsByConnection: treeState.expandedDbsByConnection,
    expandedSchemasByConnection: treeState.expandedSchemasByConnection,
    selectedSchemaByConnection: treeState.selectedSchemaByConnection,
    filterQuery: treeState.filterQuery,
    expandedGroups: treeState.expandedGroups,
    setFilter: treeState.setFilter,
    setSchemaExpanded: treeState.setSchemaExpanded,
    isSchemaExpanded: treeState.isSchemaExpanded,
    isDbExpanded: treeState.isDbExpanded,
    setDbExpanded: treeState.setDbExpanded,
    collapseAll: treeState.collapseAll,

    // Actions
    loadSchema: actions.loadSchema,
    loadDbSchema: actions.loadDbSchema,
    clearDbSchema: actions.clearDbSchema,
    refreshDbSchema: actions.refreshDbSchema,

    // Store logic
    getFilteredTables,
    getFilteredViews,
    getFilteredFunctions,
    setSelectedSchema,
    isConnectionLoading,
    isDbLoading,
    removeConnection: (id: string) => {
      cache.removeConnection(id);
      treeState.removeConnection(id);
    },

    // Legacy shims
    schema,
    selectedSchema,
    filteredTables,
    filteredViews,
    filteredFunctions,
    loadedAllDatabases
  };
});
