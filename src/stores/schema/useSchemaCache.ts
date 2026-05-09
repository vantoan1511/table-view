import { ref } from 'vue'
import type { SchemaInfo } from '@/types'

export function useSchemaCache() {
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

  function emptySchema(): SchemaInfo {
    return { tables: [], views: [], functions: [], schemas: [], databases: undefined }
  }

  const hasSchemaLoaded = (connectionId: string) => {
    return !!schemasByConnection.value[connectionId]
  }

  const getDbSchema = (connectionId: string, dbName: string): SchemaInfo | null => {
    return perDbSchemas.value[connectionId]?.[dbName] ?? null
  }

  const getDbError = (connectionId: string, dbName: string): string | null => {
    return errorDbByConnection.value[connectionId]?.[dbName] ?? null
  }

  return {
    schemasByConnection,
    loadingByConnection,
    perDbSchemas,
    loadingDbByConnection,
    errorDbByConnection,
    emptySchema,
    hasSchemaLoaded,
    getDbSchema,
    getDbError
  }
}
