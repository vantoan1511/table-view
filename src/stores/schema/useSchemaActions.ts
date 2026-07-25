import { BridgeService } from '@/services/bridge';
import type { SchemaInfo } from '@/types';

import { useToastStore } from '../toast';

export function useSchemaActions(
  cache: any,
  treeState: any,
  connectionsStore: any,
  loadedAllDatabases: any
) {
  const toastStore = useToastStore();
  const sameSchema = (a: string, b: string) =>
    a.localeCompare(b, undefined, { sensitivity: 'accent' }) === 0;

  const resolveFallbackSchema = (connectionType?: string, username?: string) => {
    if (connectionType === 'oracle') return username?.toUpperCase() || '';
    if (connectionType === 'postgresql' || connectionType === 'postgres') return 'public';
    return '';
  };

  const loadSchema = async (allDatabases?: boolean, connectionId?: string, schemaName?: string) => {
    const targetConnectionId = connectionId || connectionsStore.activeConnectionId;
    if (!targetConnectionId) return;

    const connection = connectionsStore.connections.find((c: any) => c.id === targetConnectionId);
    const targetAllDatabases =
      allDatabases ?? connection?.displayAllDatabases ?? loadedAllDatabases.value;
    const fallbackSchema = resolveFallbackSchema(connection?.type, connection?.username);
    const requestedSchemaName =
      schemaName || treeState.selectedSchemaByConnection.value[targetConnectionId] || '';
    const targetSchemaName = requestedSchemaName || (targetAllDatabases ? '' : fallbackSchema);

    loadedAllDatabases.value = targetAllDatabases;
    cache.loadingByConnection.value[targetConnectionId] = true;

    try {
      const payload = await BridgeService.request(
        'dbBridge.getSchema',
        'dbBridge.getSchemaResult',
        {
          connectionId: targetConnectionId,
          allDatabases: targetAllDatabases,
          schemaName: targetSchemaName
        }
      );

      if (!payload) {
        throw new Error('Empty response from database bridge');
      }

      const backendSchemas = payload.schemas || [];
      const backendDatabases: any[] = payload.databases || [];
      const defaultObjectSchema = resolveFallbackSchema(connection?.type, connection?.username);

      const existingSchema = cache.schemasByConnection.value[targetConnectionId];

      const mergeObjects = <T extends { schema: string }>(
        existing: T[] | undefined,
        incoming: any[],
        mapFn: (o: any) => T
      ): T[] => {
        const newObjects = incoming.map(mapFn);
        if (existing && targetSchemaName) {
          return [
            ...existing.filter((o) => !sameSchema(o.schema, targetSchemaName)),
            ...newObjects
          ];
        }
        return newObjects;
      };

      const mergedTables = mergeObjects(existingSchema?.tables, payload.tables || [], (t: any) => ({
        name: t.name,
        schema: t.schema || defaultObjectSchema
      }));

      const mergedViews = mergeObjects(existingSchema?.views, payload.views || [], (v: any) => ({
        name: v.name,
        schema: v.schema || defaultObjectSchema
      }));

      const mergedFunctions = mergeObjects(
        existingSchema?.functions,
        payload.functions || [],
        (f: any) => ({
          name: f.name,
          schema: f.schema || defaultObjectSchema,
          returnType: f.type || 'unknown'
        })
      );

      const mergedSchemas =
        backendSchemas.length > 0
          ? backendSchemas.map((s: any) => s.name || s)
          : existingSchema?.schemas || [];

      const nextSchema: SchemaInfo = {
        tables: mergedTables,
        views: mergedViews,
        functions: mergedFunctions,
        schemas: mergedSchemas,
        databases:
          backendDatabases.length > 0
            ? backendDatabases.map((d: any) => d.name || d)
            : existingSchema?.databases
      };

      cache.schemasByConnection.value[targetConnectionId] = nextSchema;

      const availableSchemas = nextSchema.schemas;
      const firstSchemaWithObjects =
        nextSchema.tables[0]?.schema ||
        nextSchema.views[0]?.schema ||
        nextSchema.functions[0]?.schema ||
        '';

      const canUseRequestedSchema =
        !!requestedSchemaName &&
        (availableSchemas.some((s) => sameSchema(s, requestedSchemaName)) ||
          sameSchema(firstSchemaWithObjects, requestedSchemaName));

      const currentSelected = treeState.selectedSchemaByConnection.value[targetConnectionId];
      if (
        schemaName ||
        !currentSelected ||
        !availableSchemas.some((s) => sameSchema(s, currentSelected))
      ) {
        treeState.selectedSchemaByConnection.value[targetConnectionId] =
          (canUseRequestedSchema ? requestedSchemaName : '') ||
          firstSchemaWithObjects ||
          availableSchemas[0] ||
          fallbackSchema;
      }

      if (availableSchemas.length > 0) {
        const firstSchema = availableSchemas[0];
        if (firstSchema && !treeState.expandedSchemasByConnection.value[targetConnectionId]) {
          treeState.setSchemaExpanded(targetConnectionId, firstSchema, true);
        }
      }
    } catch (error: any) {
      console.error('Failed to load schema:', error.message);
      toastStore.addToast({
        severity: 'error',
        title: 'Schema Load Failed',
        message: error.message || 'Could not retrieve the database schema.'
      });
    } finally {
      cache.loadingByConnection.value[targetConnectionId] = false;
    }
  };

  const loadDbSchema = async (connectionId: string, dbName: string, force = false) => {
    if (
      cache.loadingDbByConnection.value[connectionId]?.[dbName] ||
      (!force && cache.perDbSchemas.value[connectionId]?.[dbName])
    )
      return;

    if (!cache.loadingDbByConnection.value[connectionId])
      cache.loadingDbByConnection.value[connectionId] = {};
    if (!cache.errorDbByConnection.value[connectionId])
      cache.errorDbByConnection.value[connectionId] = {};

    cache.loadingDbByConnection.value[connectionId][dbName] = true;
    delete cache.errorDbByConnection.value[connectionId][dbName];

    const connection = connectionsStore.connections.find((c: any) => c.id === connectionId);
    const defaultObjectSchema = resolveFallbackSchema(connection?.type, connection?.username);

    try {
      const payload = await BridgeService.request(
        'dbBridge.getDbSchema',
        'dbBridge.getDbSchemaResult',
        { connectionId, targetDatabase: dbName }
      );

      if (!payload) {
        throw new Error('Empty response from database bridge');
      }

      const backendSchemas = payload.schemas || [];
      const dbSchema: SchemaInfo = {
        tables: (payload.tables || []).map((t: any) => ({
          name: t.name,
          schema: t.schema || defaultObjectSchema
        })),
        views: (payload.views || []).map((v: any) => ({
          name: v.name,
          schema: v.schema || defaultObjectSchema
        })),
        functions: (payload.functions || []).map((f: any) => ({
          name: f.name,
          schema: f.schema || defaultObjectSchema,
          returnType: f.type || 'unknown'
        })),
        schemas: backendSchemas.map((s: any) => s.name || s)
      };

      if (!cache.perDbSchemas.value[connectionId]) cache.perDbSchemas.value[connectionId] = {};
      cache.perDbSchemas.value[connectionId][dbName] = dbSchema;
    } catch (error: any) {
      if (!cache.errorDbByConnection.value[connectionId])
        cache.errorDbByConnection.value[connectionId] = {};
      cache.errorDbByConnection.value[connectionId][dbName] = error.message || 'Unknown error';
      console.error(`[schema] Failed to load schema for ${dbName}:`, error.message);
    } finally {
      if (cache.loadingDbByConnection.value[connectionId]) {
        cache.loadingDbByConnection.value[connectionId][dbName] = false;
      }
    }
  };

  const clearDbSchema = (connectionId: string, dbName: string) => {
    if (cache.perDbSchemas.value[connectionId]) {
      delete cache.perDbSchemas.value[connectionId][dbName];
    }
    if (cache.errorDbByConnection.value[connectionId]) {
      delete cache.errorDbByConnection.value[connectionId][dbName];
    }
    if (cache.loadingDbByConnection.value[connectionId]) {
      delete cache.loadingDbByConnection.value[connectionId][dbName];
    }
    if (treeState.expandedDbsByConnection.value[connectionId]) {
      delete treeState.expandedDbsByConnection.value[connectionId][dbName];
    }
  };

  const refreshDbSchema = async (connectionId: string, dbName: string) => {
    clearDbSchema(connectionId, dbName);
    await loadDbSchema(connectionId, dbName);
  };

  const loadTableIndexes = async (connectionId: string, tableName: string) => {
    try {
      const payload = await BridgeService.request(
        'dbBridge.getTableIndexes',
        'dbBridge.getTableIndexesResult',
        {
          connectionId,
          tableName
        }
      );
      if (payload && payload.indexes) {
        if (!cache.tableIndexes.value[connectionId]) {
          cache.tableIndexes.value[connectionId] = {};
        }
        cache.tableIndexes.value[connectionId][tableName] = payload.indexes;
      }
    } catch (error: any) {
      console.error(`[schema] Failed to load indexes for ${tableName}:`, error.message);
      toastStore.addToast({
        severity: 'error',
        title: 'Indexes Load Failed',
        message: error.message || 'Could not retrieve table indexes.'
      });
    }
  };

  return {
    sameSchema,
    resolveFallbackSchema,
    loadSchema,
    loadDbSchema,
    clearDbSchema,
    refreshDbSchema,
    loadTableIndexes
  };
}
