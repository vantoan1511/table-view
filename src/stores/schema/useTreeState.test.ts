import { describe, it, expect } from 'vitest'
import { useTreeState } from './useTreeState'

describe('useTreeState', () => {
  it('initializes with empty state', () => {
    const state = useTreeState()
    expect(state.expandedDbsByConnection.value).toEqual({})
    expect(state.expandedSchemasByConnection.value).toEqual({})
    expect(state.selectedSchemaByConnection.value).toEqual({})
    expect(state.filterQuery.value).toBe('')
    expect(state.expandedGroups.value).toEqual({})
  })

  it('setFilter updates filterQuery', () => {
    const state = useTreeState()
    state.setFilter('test')
    expect(state.filterQuery.value).toBe('test')
  })

  it('manages expanded schemas correctly', () => {
    const state = useTreeState()
    const connId = 'conn-1'
    const schemaName = 'public'
    
    expect(state.isSchemaExpanded(connId, schemaName)).toBe(false)
    
    state.setSchemaExpanded(connId, schemaName, true)
    expect(state.isSchemaExpanded(connId, schemaName)).toBe(true)
    
    state.setSchemaExpanded(connId, schemaName, false)
    expect(state.isSchemaExpanded(connId, schemaName)).toBe(false)
  })

  it('manages expanded databases correctly', () => {
    const state = useTreeState()
    const connId = 'conn-1'
    const dbName = 'postgres'
    
    expect(state.isDbExpanded(connId, dbName)).toBe(false)
    
    state.setDbExpanded(connId, dbName, true)
    expect(state.isDbExpanded(connId, dbName)).toBe(true)
    
    state.setDbExpanded(connId, dbName, false)
    expect(state.isDbExpanded(connId, dbName)).toBe(false)
  })
})
