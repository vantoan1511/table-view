import { ref } from 'vue'

export function useTreeState() {
  // Per-connection expanded databases in the tree
  const expandedDbsByConnection = ref<Record<string, Record<string, boolean>>>({})

  // Per-connection expanded schemas (which schema nodes are open in the tree)
  const expandedSchemasByConnection = ref<Record<string, Record<string, boolean>>>({})

  // Per-connection selected schema (for SQL editor context, etc.)
  const selectedSchemaByConnection = ref<Record<string, string>>({})

  // Global search filter (applies across all visible tree nodes)
  const filterQuery = ref('')

  // Per-connection, per-schema, per-group expansion state
  const expandedGroups = ref<Record<string, boolean>>({})

  const setFilter = (query: string) => {
    filterQuery.value = query
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

  const isDbExpanded = (connectionId: string, dbName: string) => {
    return expandedDbsByConnection.value[connectionId]?.[dbName] ?? false
  }

  const setDbExpanded = (connectionId: string, dbName: string, expanded: boolean) => {
    if (!expandedDbsByConnection.value[connectionId]) {
      expandedDbsByConnection.value[connectionId] = {}
    }
    expandedDbsByConnection.value[connectionId][dbName] = expanded
  }

  return {
    expandedDbsByConnection,
    expandedSchemasByConnection,
    selectedSchemaByConnection,
    filterQuery,
    expandedGroups,
    setFilter,
    setSchemaExpanded,
    isSchemaExpanded,
    isDbExpanded,
    setDbExpanded
  }
}
