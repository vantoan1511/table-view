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
  const schema = ref<SchemaInfo>({ tables: [], views: [], functions: [], schemas: ['public'] })
  const selectedSchema = ref('public')
  const filterQuery = ref('')

  const filteredTables = computed(() => {
    const q = filterQuery.value.toLowerCase()
    return schema.value.tables.filter(
      (t) => t.schema === selectedSchema.value && t.name.toLowerCase().includes(q),
    )
  })

  const filteredViews = computed(() => {
    const q = filterQuery.value.toLowerCase()
    return schema.value.views.filter(
      (v) => v.schema === selectedSchema.value && v.name.toLowerCase().includes(q),
    )
  })

  const filteredFunctions = computed(() => {
    const q = filterQuery.value.toLowerCase()
    return schema.value.functions.filter(
      (f) => f.schema === selectedSchema.value && f.name.toLowerCase().includes(q),
    )
  })

  const setFilter = (query: string) => {
    filterQuery.value = query
  }

  const setSelectedSchema = (s: string) => {
    selectedSchema.value = s
  }

  const loadSchema = async (allSchemas: boolean = false) => {
    if (window.NL_PORT) {
      const reqId = Date.now().toString()
      
      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId === reqId) {
          if (payload.success) {
            const backendSchemas = payload.schema.schemas || []
            schema.value = {
              tables: (payload.schema.tables || []).map((t: any) => ({ name: t.name, schema: t.schema || 'public' })),
              views: (payload.schema.views || []).map((v: any) => ({ name: v.name, schema: v.schema || 'public' })),
              functions: (payload.schema.functions || []).map((f: any) => ({ name: f.name, schema: f.schema || 'public', returnType: f.type || 'unknown' })),
              schemas: backendSchemas.length > 0 ? backendSchemas.map((s: any) => s.name || s) : ['public']
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
        connectionId: connectionsStore.activeConnectionId,
        allSchemas 
      })
    }
  }

  return {
    schema,
    selectedSchema,
    filterQuery,
    filteredTables,
    filteredViews,
    filteredFunctions,
    setFilter,
    setSelectedSchema,
    loadSchema
  }
})
