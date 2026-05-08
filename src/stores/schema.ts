import * as Neutralino from '@neutralinojs/lib'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { SchemaInfo } from '@/types'
import { useConnectionsStore } from './connections'

export const useSchemaStore = defineStore('schema', () => {
  const connectionsStore = useConnectionsStore()

  // Per-connection schema cache: connectionId -> SchemaInfo
  const schemasByConnection = ref<Record<string, SchemaInfo>>({})

  // Per-connection loading state
  const loadingByConnection = ref<Record<string, boolean>>({})

  // Per-connection, per-database schema cache for allDatabases mode
  // Shape: { [connectionId]: { [dbName]: SchemaInfo } }
  const perDbSchemas = ref<Record<string, Record<string, SchemaInfo>>>({})

  // Loading state per (connectionId, dbName)
  const loadingDbByConnection = ref<Record<string, Record<string, boolean>>>({})

  // Error state per (connectionId, dbName)
  const errorDbByConnection = ref<Record<string, Record<string, string>>>({})

  // Per-connection expanded databases in the tree
  const expandedDbsByConnection = ref<Record<string, Record<string, boolean>>>({})

  // Per-connection expanded schemas (which schema nodes are open in the tree)
  const expandedSchemasByConnection = ref<Record<string, Record<string, boolean>>>({})

  // Per-connection selected schema (for SQL editor context, etc.)
  const selectedSchemaByConnection = ref<Record<string, string>>({})

  // Global search filter (applies across all visible tree nodes)
  const filterQuery = ref('')

  // ─── Active-connection shims (for backward compat with SqlEditor, etc.) ──────
  // These derive from the *active* connection's state so existing callers keep working
  const activeConnectionId = computed(() => connectionsStore.activeConnectionId)

  const schema = computed(() => {
    const id = activeConnectionId.value
    return id ? (schemasByConnection.value[id] ?? emptySchema()) : emptySchema()
  })

  const selectedSchema = computed(() => {
    const id = activeConnectionId.value
    return id ? (selectedSchemaByConnection.value[id] ?? '') : ''
  })

  const loadedAllDatabases = ref(false)
  const activeSchemaRequestId = ref('')

  // ─── Helpers ─────────────────────────────────────────────────────────────────
  function emptySchema(): SchemaInfo {
    return { tables: [], views: [], functions: [], schemas: [], databases: undefined }
  }

  const sameSchema = (a: string, b: string) =>
    a.localeCompare(b, undefined, { sensitivity: 'accent' }) === 0

  const resolveFallbackSchema = (connectionType?: string, username?: string) => {
    if (connectionType === 'oracle') return username?.toUpperCase() || ''
    if (connectionType === 'postgresql' || connectionType === 'postgres') return 'public'
    return ''
  }

  // ─── Per-connection filtered lists ───────────────────────────────────────────
  const getFilteredTables = (connectionId: string, schemaName?: string) => {
    const s = schemasByConnection.value[connectionId]
    if (!s) return []
    const targetSchema = schemaName ?? selectedSchemaByConnection.value[connectionId] ?? ''
    const q = filterQuery.value.toLowerCase()
    return s.tables.filter(
      (t) => sameSchema(t.schema, targetSchema) && t.name.toLowerCase().includes(q),
    )
  }

  const getFilteredViews = (connectionId: string, schemaName?: string) => {
    const s = schemasByConnection.value[connectionId]
    if (!s) return []
    const targetSchema = schemaName ?? selectedSchemaByConnection.value[connectionId] ?? ''
    const q = filterQuery.value.toLowerCase()
    return s.views.filter(
      (v) => sameSchema(v.schema, targetSchema) && v.name.toLowerCase().includes(q),
    )
  }

  const getFilteredFunctions = (connectionId: string, schemaName?: string) => {
    const s = schemasByConnection.value[connectionId]
    if (!s) return []
    const targetSchema = schemaName ?? selectedSchemaByConnection.value[connectionId] ?? ''
    const q = filterQuery.value.toLowerCase()
    return s.functions.filter(
      (f) => sameSchema(f.schema, targetSchema) && f.name.toLowerCase().includes(q),
    )
  }

  // Shim for legacy computed consumers (e.g. old SchemaTree, SqlEditor)
  const filteredTables = computed(() => {
    const id = activeConnectionId.value
    return id ? getFilteredTables(id) : []
  })

  const filteredViews = computed(() => {
    const id = activeConnectionId.value
    return id ? getFilteredViews(id) : []
  })

  const filteredFunctions = computed(() => {
    const id = activeConnectionId.value
    return id ? getFilteredFunctions(id) : []
  })

  // ─── Mutations ────────────────────────────────────────────────────────────────
  const setFilter = (query: string) => {
    filterQuery.value = query
  }

  const setSelectedSchema = (s: string, connectionId?: string) => {
    const id = connectionId ?? activeConnectionId.value
    if (!id) return
    selectedSchemaByConnection.value[id] = s
  }

  const setSchemaExpanded = (connectionId: string, schemaName: string, expanded: boolean) => {
    if (!expandedSchemasByConnection.value[connectionId]) {
      expandedSchemasByConnection.value[connectionId] = {}
    }
    expandedSchemasByConnection.value[connectionId][schemaName] = expanded
  }

  const isSchemaExpanded = (connectionId: string, schemaName: string) => {
    return expandedSchemasByConnection.value[connectionId]?.[schemaName] ?? false
  }

  const isConnectionLoading = (connectionId: string) => {
    return loadingByConnection.value[connectionId] ?? false
  }

  const isDbLoading = (connectionId: string, dbName: string) => {
    return loadingDbByConnection.value[connectionId]?.[dbName] ?? false
  }

  const isDbExpanded = (connectionId: string, dbName: string) => {
    return expandedDbsByConnection.value[connectionId]?.[dbName] ?? false
  }

  const setDbExpanded = (connectionId: string, dbName: string, expanded: boolean) => {
    if (!expandedDbsByConnection.value[connectionId]) {
      expandedDbsByConnection.value[connectionId] = {}
    }
    expandedDbsByConnection.value[connectionId][dbName] = expanded
  }

  const getDbSchema = (connectionId: string, dbName: string): SchemaInfo | null => {
    return perDbSchemas.value[connectionId]?.[dbName] ?? null
  }

  const getDbError = (connectionId: string, dbName: string): string | null => {
    return errorDbByConnection.value[connectionId]?.[dbName] ?? null
  }

  const hasSchemaLoaded = (connectionId: string) => {
    return !!schemasByConnection.value[connectionId]
  }

  // ─── Load schema for a specific connection ───────────────────────────────────
  const loadSchema = async (
    allDatabases?: boolean,
    connectionId?: string,
    schemaName?: string,
  ) => {
    if (!window.NL_PORT) return

    const reqId = Date.now().toString()
    const targetConnectionId = connectionId || connectionsStore.activeConnectionId
    if (!targetConnectionId) return

    const connection = connectionsStore.connections.find((c) => c.id === targetConnectionId)
    const targetAllDatabases = allDatabases ?? connection?.displayAllDatabases ?? loadedAllDatabases.value
    const fallbackSchema = resolveFallbackSchema(connection?.type, connection?.username)
    const requestedSchemaName = schemaName || selectedSchemaByConnection.value[targetConnectionId] || ''
    const targetSchemaName = requestedSchemaName || (targetAllDatabases ? '' : fallbackSchema)

    // Update legacy ref for compat
    loadedAllDatabases.value = targetAllDatabases
    activeSchemaRequestId.value = reqId

    // Mark loading
    loadingByConnection.value[targetConnectionId] = true

    const onResult = (evt: any) => {
      const payload = evt.detail
      if (payload.reqId !== reqId) return

      if (activeSchemaRequestId.value !== reqId) {
        Neutralino.events.off('dbBridge.getSchemaResult', onResult)
        loadingByConnection.value[targetConnectionId] = false
        return
      }

      if (payload.success) {
        const backendSchemas = payload.schema.schemas || []
        // When allDatabases=true the backend returns a `databases` list in addition.
        // These are kept separate: `schemas` = schemas of the configured database,
        // `databases` = all server databases (shown as a read-only tier in the tree).
        const backendDatabases: any[] = payload.schema.databases || []
        const defaultObjectSchema = resolveFallbackSchema(connection?.type, connection?.username)
        const nextSchema: SchemaInfo = {
          tables: (payload.schema.tables || []).map((t: any) => ({
            name: t.name,
            schema: t.schema || defaultObjectSchema,
          })),
          views: (payload.schema.views || []).map((v: any) => ({
            name: v.name,
            schema: v.schema || defaultObjectSchema,
          })),
          functions: (payload.schema.functions || []).map((f: any) => ({
            name: f.name,
            schema: f.schema || defaultObjectSchema,
            returnType: f.type || 'unknown',
          })),
          // Schemas of the configured (connected) database
          schemas: backendSchemas.length > 0 ? backendSchemas.map((s: any) => s.name || s) : [],
          // All database names on the server — only set when allDatabases mode is active
          databases: backendDatabases.length > 0
            ? backendDatabases.map((d: any) => d.name || d)
            : undefined,
        }

        // Store in per-connection map
        schemasByConnection.value[targetConnectionId] = nextSchema

        // Resolve the active schema for this connection
        const availableSchemas = nextSchema.schemas
        const firstSchemaWithObjects =
          nextSchema.tables[0]?.schema ||
          nextSchema.views[0]?.schema ||
          nextSchema.functions[0]?.schema ||
          ''

        const canUseRequestedSchema =
          !!requestedSchemaName &&
          (availableSchemas.some((s) => sameSchema(s, requestedSchemaName)) ||
            sameSchema(firstSchemaWithObjects, requestedSchemaName))

        const currentSelected = selectedSchemaByConnection.value[targetConnectionId]
        if (
          schemaName ||
          !currentSelected ||
          !availableSchemas.some((s) => sameSchema(s, currentSelected))
        ) {
          selectedSchemaByConnection.value[targetConnectionId] =
            (canUseRequestedSchema ? requestedSchemaName : '') ||
            firstSchemaWithObjects ||
            availableSchemas[0] ||
            fallbackSchema
        }

        // Auto-expand the first schema in tree
        if (availableSchemas.length > 0) {
          const firstSchema = availableSchemas[0]
          if (firstSchema && !expandedSchemasByConnection.value[targetConnectionId]) {
            setSchemaExpanded(targetConnectionId, firstSchema, true)
          }
        }
      } else {
        console.error('Failed to load schema:', payload.error)
      }

      loadingByConnection.value[targetConnectionId] = false
      Neutralino.events.off('dbBridge.getSchemaResult', onResult)
    }

    Neutralino.events.on('dbBridge.getSchemaResult', onResult)
    Neutralino.extensions.dispatch(
      'com.github.vantoan1511.table-view.db-bridge',
      'dbBridge.getSchema',
      {
        reqId,
        connectionId: targetConnectionId,
        allDatabases: targetAllDatabases,
        schemaName: targetSchemaName,
      },
    )
  }

  // ─── Load schema for a specific database (allDatabases mode) ────────────────
  const loadDbSchema = async (connectionId: string, dbName: string) => {
    if (!window.NL_PORT) return
    // Already loaded or loading — skip
    if (
      loadingDbByConnection.value[connectionId]?.[dbName] ||
      perDbSchemas.value[connectionId]?.[dbName]
    ) return

    if (!loadingDbByConnection.value[connectionId])
      loadingDbByConnection.value[connectionId] = {}
    if (!errorDbByConnection.value[connectionId])
      errorDbByConnection.value[connectionId] = {}

    loadingDbByConnection.value[connectionId][dbName] = true
    delete errorDbByConnection.value[connectionId][dbName]

    const reqId = `${connectionId}::${dbName}::${Date.now()}`
    const connection = connectionsStore.connections.find((c) => c.id === connectionId)
    const defaultObjectSchema = resolveFallbackSchema(connection?.type, connection?.username)

    const onResult = (evt: any) => {
      const payload = evt.detail
      if (payload.reqId !== reqId) return
      Neutralino.events.off('dbBridge.getDbSchemaResult', onResult)

      if (!loadingDbByConnection.value[connectionId])
        loadingDbByConnection.value[connectionId] = {}
      loadingDbByConnection.value[connectionId][dbName] = false

      if (payload.success) {
        const backendSchemas = payload.schemas || []
        const dbSchema: SchemaInfo = {
          tables: (payload.tables || []).map((t: any) => ({
            name: t.name,
            schema: t.schema || defaultObjectSchema,
          })),
          views: (payload.views || []).map((v: any) => ({
            name: v.name,
            schema: v.schema || defaultObjectSchema,
          })),
          functions: (payload.functions || []).map((f: any) => ({
            name: f.name,
            schema: f.schema || defaultObjectSchema,
            returnType: f.type || 'unknown',
          })),
          schemas: backendSchemas.map((s: any) => s.name || s),
        }
        if (!perDbSchemas.value[connectionId]) perDbSchemas.value[connectionId] = {}
        perDbSchemas.value[connectionId][dbName] = dbSchema
      } else {
        if (!errorDbByConnection.value[connectionId])
          errorDbByConnection.value[connectionId] = {}
        errorDbByConnection.value[connectionId][dbName] = payload.error || 'Unknown error'
        console.error(`[schema] Failed to load schema for ${dbName}:`, payload.error)
      }
    }

    Neutralino.events.on('dbBridge.getDbSchemaResult', onResult)
    Neutralino.extensions.dispatch(
      'com.github.vantoan1511.table-view.db-bridge',
      'dbBridge.getDbSchema',
      { reqId, connectionId, targetDatabase: dbName },
    )
  }

  // Clear cached schema for a specific database (triggers reload on next expand)
  const clearDbSchema = (connectionId: string, dbName: string) => {
    if (perDbSchemas.value[connectionId]) {
      delete perDbSchemas.value[connectionId][dbName]
    }
    if (errorDbByConnection.value[connectionId]) {
      delete errorDbByConnection.value[connectionId][dbName]
    }
  }

  // Refresh (force-reload) a specific database's schema
  const refreshDbSchema = async (connectionId: string, dbName: string) => {
    clearDbSchema(connectionId, dbName)
    await loadDbSchema(connectionId, dbName)
  }

  return {
    // New per-connection API
    schemasByConnection,
    selectedSchemaByConnection,
    expandedSchemasByConnection,
    expandedDbsByConnection,
    loadingByConnection,
    perDbSchemas,
    loadingDbByConnection,
    errorDbByConnection,
    getFilteredTables,
    getFilteredViews,
    getFilteredFunctions,
    setSchemaExpanded,
    isSchemaExpanded,
    isConnectionLoading,
    isDbLoading,
    isDbExpanded,
    setDbExpanded,
    getDbSchema,
    getDbError,
    hasSchemaLoaded,
    // Per-DB actions
    loadDbSchema,
    clearDbSchema,
    refreshDbSchema,
    // Shared
    filterQuery,
    setFilter,
    setSelectedSchema,
    loadSchema,
    // Legacy shims for backward compatibility
    schema,
    selectedSchema,
    filteredTables,
    filteredViews,
    filteredFunctions,
    loadedAllDatabases,
  }
})
