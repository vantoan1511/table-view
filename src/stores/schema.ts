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

  // Keep a legacy ref for one-way compatibility; prefer computed above
  const loadedAllSchemas = ref(false)
  const activeSchemaRequestId = ref('')

  // ─── Helpers ─────────────────────────────────────────────────────────────────
  function emptySchema(): SchemaInfo {
    return { tables: [], views: [], functions: [], schemas: [] }
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

  const hasSchemaLoaded = (connectionId: string) => {
    return !!schemasByConnection.value[connectionId]
  }

  // ─── Load schema for a specific connection ───────────────────────────────────
  const loadSchema = async (
    allSchemas?: boolean,
    connectionId?: string,
    schemaName?: string,
  ) => {
    if (!window.NL_PORT) return

    const reqId = Date.now().toString()
    const targetConnectionId = connectionId || connectionsStore.activeConnectionId
    if (!targetConnectionId) return

    const connection = connectionsStore.connections.find((c) => c.id === targetConnectionId)
    const targetAllSchemas = allSchemas ?? connection?.displayAllSchemas ?? loadedAllSchemas.value
    const fallbackSchema = resolveFallbackSchema(connection?.type, connection?.username)
    const requestedSchemaName = schemaName || selectedSchemaByConnection.value[targetConnectionId] || ''
    const targetSchemaName = requestedSchemaName || (targetAllSchemas ? '' : fallbackSchema)

    // Update legacy ref for compat
    loadedAllSchemas.value = targetAllSchemas
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
          schemas: backendSchemas.length > 0 ? backendSchemas.map((s: any) => s.name || s) : [],
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
        allSchemas: targetAllSchemas,
        schemaName: targetSchemaName,
      },
    )
  }

  return {
    // New per-connection API
    schemasByConnection,
    selectedSchemaByConnection,
    expandedSchemasByConnection,
    loadingByConnection,
    getFilteredTables,
    getFilteredViews,
    getFilteredFunctions,
    setSchemaExpanded,
    isSchemaExpanded,
    isConnectionLoading,
    hasSchemaLoaded,
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
    loadedAllSchemas,
  }
})
