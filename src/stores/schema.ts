import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SchemaInfo, SchemaTable, SchemaView, SchemaFunction } from '@/types'

// ─── Mock Data ──────────────────────────────────────────────────────────────

const mockSchema: SchemaInfo = {
  tables: [
    { name: 'users', schema: 'public' },
    { name: 'orders', schema: 'public' },
    { name: 'order_items', schema: 'public' },
    { name: 'products', schema: 'public' },
    { name: 'categories', schema: 'public' },
    { name: 'customers', schema: 'public' },
    { name: 'invoices', schema: 'public' },
    { name: 'payments', schema: 'public' },
    { name: 'roles', schema: 'public' },
    { name: 'permissions', schema: 'public' },
    { name: 'audit_logs', schema: 'public' },
    { name: 'settings', schema: 'public' },
  ],
  views: [
    { name: 'active_users', schema: 'public' },
    { name: 'order_summary', schema: 'public' },
    { name: 'revenue_report', schema: 'public' },
    { name: 'user_roles_view', schema: 'public' },
  ],
  functions: [
    { name: 'get_user_orders', schema: 'public', returnType: 'SETOF record' },
    { name: 'calculate_total', schema: 'public', returnType: 'numeric' },
    { name: 'update_timestamp', schema: 'public', returnType: 'trigger' },
    { name: 'validate_email', schema: 'public', returnType: 'boolean' },
    { name: 'generate_invoice', schema: 'public', returnType: 'uuid' },
    { name: 'archive_orders', schema: 'public', returnType: 'integer' },
  ],
  schemas: ['public', 'auth'],
}

import * as Neutralino from '@neutralinojs/lib'
import { useConnectionsStore } from './connections'

export const useSchemaStore = defineStore('schema', () => {
  const connectionsStore = useConnectionsStore()
  const schema = ref<SchemaInfo>({ tables: [], views: [], functions: [], schemas: [] })
  const selectedSchema = ref('')
  const filterQuery = ref('')
  const loadedAllSchemas = ref(false)
  const activeSchemaRequestId = ref('')

  const sameSchema = (a: string, b: string) => a.localeCompare(b, undefined, { sensitivity: 'accent' }) === 0

  const resolveFallbackSchema = (connectionType?: string, username?: string) => {
    if (connectionType === 'oracle') {
      return username?.toUpperCase() || ''
    }
    if (connectionType === 'postgresql' || connectionType === 'postgres') {
      return 'public'
    }
    return ''
  }

  const filteredTables = computed(() => {
    const q = filterQuery.value.toLowerCase()
    return schema.value.tables.filter(
      (t) => sameSchema(t.schema, selectedSchema.value) && t.name.toLowerCase().includes(q),
    )
  })

  const filteredViews = computed(() => {
    const q = filterQuery.value.toLowerCase()
    return schema.value.views.filter(
      (v) => sameSchema(v.schema, selectedSchema.value) && v.name.toLowerCase().includes(q),
    )
  })

  const filteredFunctions = computed(() => {
    const q = filterQuery.value.toLowerCase()
    return schema.value.functions.filter(
      (f) => sameSchema(f.schema, selectedSchema.value) && f.name.toLowerCase().includes(q),
    )
  })

  const setFilter = (query: string) => {
    filterQuery.value = query
  }

  const setSelectedSchema = (s: string) => {
    selectedSchema.value = s
  }

  const loadSchema = async (
    allSchemas?: boolean,
    connectionId?: string,
    schemaName?: string,
  ) => {
    if (window.NL_PORT) {
      const reqId = Date.now().toString()
      const targetConnectionId = connectionId || connectionsStore.activeConnectionId
      const connection = connectionsStore.connections.find((c) => c.id === targetConnectionId)
      const targetAllSchemas = allSchemas ?? connection?.displayAllSchemas ?? loadedAllSchemas.value
      const fallbackSchema = resolveFallbackSchema(connection?.type, connection?.username)
      const requestedSchemaName = schemaName || selectedSchema.value
      const targetSchemaName = requestedSchemaName || (targetAllSchemas ? '' : fallbackSchema)
      loadedAllSchemas.value = targetAllSchemas
      activeSchemaRequestId.value = reqId
      
      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId === reqId) {
          if (activeSchemaRequestId.value !== reqId) {
            Neutralino.events.off('dbBridge.getSchemaResult', onResult)
            return
          }

          if (payload.success) {
            const backendSchemas = payload.schema.schemas || []
            const defaultObjectSchema = resolveFallbackSchema(connection?.type, connection?.username)
            const nextSchema: SchemaInfo = {
              tables: (payload.schema.tables || []).map((t: any) => ({ name: t.name, schema: t.schema || defaultObjectSchema })),
              views: (payload.schema.views || []).map((v: any) => ({ name: v.name, schema: v.schema || defaultObjectSchema })),
              functions: (payload.schema.functions || []).map((f: any) => ({ name: f.name, schema: f.schema || defaultObjectSchema, returnType: f.type || 'unknown' })),
              schemas: backendSchemas.length > 0 ? backendSchemas.map((s: any) => s.name || s) : []
            }

            schema.value = nextSchema

            const availableSchemas = nextSchema.schemas
            const firstSchemaWithObjects =
              nextSchema.tables[0]?.schema ||
              nextSchema.views[0]?.schema ||
              nextSchema.functions[0]?.schema ||
              ''
            const canUseRequestedSchema =
              !!requestedSchemaName &&
              (
                availableSchemas.some((s) => sameSchema(s, requestedSchemaName)) ||
                sameSchema(firstSchemaWithObjects, requestedSchemaName)
              )

            if (
              schemaName ||
              !selectedSchema.value ||
              !availableSchemas.some((s) => sameSchema(s, selectedSchema.value))
            ) {
              selectedSchema.value =
                (canUseRequestedSchema ? requestedSchemaName : '') ||
                firstSchemaWithObjects ||
                availableSchemas[0] ||
                fallbackSchema
            }
          } else {
            console.error("Failed to load schema:", payload.error)
          }
          Neutralino.events.off('dbBridge.getSchemaResult', onResult)
        }
      }
      
      Neutralino.events.on('dbBridge.getSchemaResult', onResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.getSchema', { 
        reqId,
        connectionId: targetConnectionId,
        allSchemas: targetAllSchemas,
        schemaName: targetSchemaName,
      })
    }
  }

  return {
    schema,
    selectedSchema,
    filterQuery,
    loadedAllSchemas,
    filteredTables,
    filteredViews,
    filteredFunctions,
    setFilter,
    setSelectedSchema,
    loadSchema
  }
})
