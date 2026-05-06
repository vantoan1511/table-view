import type { Tab } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

import * as Neutralino from '@neutralinojs/lib'
import { useConnectionsStore } from './connections'
import { useSchemaStore } from './schema'

export const useTabsStore = defineStore('tabs', () => {
  const connectionsStore = useConnectionsStore()
  const schemaStore = useSchemaStore()
  // Only table tabs in the main tab strip
  const tabs = ref<Tab[]>([])
  const activeTabId = ref<string>('')
  const draggingTabId = ref<string | null>(null)
  const isAppClosing = ref(false)
  const showTabSelector = ref(false)
  const selectorConnectionId = ref<string | null>(null)

  // Persistence logic
  const saveTabsToStorage = async () => {
    if (!window.NL_PORT) return
    try {
      const data = JSON.stringify(tabs.value)
      await Neutralino.storage.setData('app_tabs', data)
      await Neutralino.storage.setData('app_activeTabId', activeTabId.value)
      console.log(`[TabsStore] Saved ${tabs.value.length} tabs to storage`)
      console.log(`[TabsStore] Active tab: ${activeTabId.value}`)
    } catch (err) {
      console.error('Failed to save tabs to storage:', err)
    }
  }

  const loadTabsFromStorage = async () => {
    if (!window.NL_PORT) return
    try {
      const data = await Neutralino.storage.getData('app_tabs')
      if (data) {
        const parsed = JSON.parse(data)
        tabs.value = parsed
        console.log(`[TabsStore] Loaded ${parsed.length} tabs from storage`)
      }
      const activeId = await Neutralino.storage.getData('app_activeTabId')
      if (activeId) {
        activeTabId.value = activeId
      }
    } catch (err) {
      console.log('[TabsStore] No persisted tabs found')
    }
  }

  // Watch for changes and save (debounced via setTimeout to avoid too many writes)
  let saveTimeout: any = null
  watch(tabs, () => {
    if (saveTimeout) clearTimeout(saveTimeout)
    saveTimeout = setTimeout(saveTabsToStorage, 500)
  }, { deep: true })
  
  watch(activeTabId, () => saveTabsToStorage())

  const activeTab = computed(() =>
    tabs.value.find((t) => t.id === activeTabId.value) ?? null,
  )

  const mainTabs = computed(() => tabs.value.filter(t => !t.minimized && !t.closed))
  const minimizedTabs = computed(() => tabs.value.filter(t => t.minimized && !t.closed))

  const setActiveTab = (id: string) => {
    activeTabId.value = id
  }

  const minimizeTab = (id: string) => {
    const tab = tabs.value.find(t => t.id === id)
    if (tab && !tab.minimized) {
      tab.minimized = true
      // If the minimized tab was the active one, fallback to the last main tab
      if (activeTabId.value === id) {
        const next = mainTabs.value[mainTabs.value.length - 1]
        activeTabId.value = next?.id ?? ''
      }
    }
  }

  const restoreTab = (id: string) => {
    const tab = tabs.value.find(t => t.id === id)
    if (tab && tab.minimized) {
      tab.minimized = false
      activeTabId.value = tab.id
    }
  }

  const reorderTab = (fromId: string, toId: string) => {
    const fromIdx = tabs.value.findIndex(t => t.id === fromId)
    const toIdx = tabs.value.findIndex(t => t.id === toId)
    if (fromIdx !== -1 && toIdx !== -1 && fromIdx !== toIdx) {
      const removed = tabs.value.splice(fromIdx, 1)
      const movedTab = removed[0]
      if (movedTab) {
        tabs.value.splice(toIdx, 0, movedTab)
      }
    }
  }

  const renameTab = (id: string, newTitle: string) => {
    const tab = tabs.value.find(t => t.id === id)
    if (tab) {
      tab.title = newTitle
    }
  }

  const openTable = (tableName: string, schemaName?: string, connectionId?: string) => {
    const targetSchema = schemaName || schemaStore.selectedSchema
    const targetConnectionId = connectionId ?? connectionsStore.activeConnectionId ?? undefined
    const existing = tabs.value.find(
      (t) => t.type === 'table' && t.tableName === tableName && t.schema === targetSchema && t.connectionId === targetConnectionId,
    )
    if (existing) {
      activeTabId.value = existing.id
      return
    }
    const tab: Tab = {
      id: `tab-${targetSchema}-${tableName}-${Date.now()}`,
      type: 'table',
      title: targetSchema ? `${targetSchema}.${tableName}` : tableName,
      tableName,
      connectionId: targetConnectionId,
      schema: targetSchema,
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }

  const getNextEditorNumber = (connectionName: string) => {
    const prefix = `${connectionName}-`
    const existingNumbers = tabs.value
      .filter(t => t.title.startsWith(prefix))
      .map(t => {
        const numPart = t.title.substring(prefix.length)
        const n = parseInt(numPart)
        return isNaN(n) ? 0 : n
      })
    return existingNumbers.length > 0 ? Math.max(...existingNumbers) + 1 : 1
  }

  const openSqlEditor = (connectionId?: string, query: string = '', isDraft: boolean = true, forceNew: boolean = false) => {
    const connId = connectionId || connectionsStore.activeConnectionId || undefined
    
    // If not forcing new, try to find an existing editor for this connection
    if (!forceNew && connId) {
      const existingTabs = tabs.value.filter(t => t.type === 'sql' && t.connectionId === connId)
      if (existingTabs.length > 1) {
        selectorConnectionId.value = connId
        showTabSelector.value = true
        return
      } else if (existingTabs.length === 1) {
        const tab = existingTabs[0]
        if (tab) {
          tab.closed = false
          tab.minimized = false
          activeTabId.value = tab.id
          if (query) tab.query = query
        }
        return
      }
    }

    const connection = connectionsStore.connections.find(c => c.id === connId)
    const connName = connection?.name || 'query'
    const nextNum = getNextEditorNumber(connName)

    const tab: Tab = {
      id: `tab-sql-${Date.now()}`,
      type: 'sql',
      title: `${connName}-${nextNum}`,
      connectionId: connId,
      schema: schemaStore.selectedSchema,
      query,
      isDraft,
      isDirty: false,
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }

  const updateTabQuery = (id: string, query: string) => {
    const tab = tabs.value.find(t => t.id === id)
    if (tab && tab.type === 'sql') {
      if (tab.query !== query) {
        console.log(`[TabsStore] Updating query for tab ${id}, length: ${query.length}`)
        tab.query = query
        tab.isDirty = true
      }
    }
  }

  const saveSqlTab = (id: string) => {
    const tab = tabs.value.find(t => t.id === id)
    if (tab && tab.type === 'sql') {
      tab.isDirty = false
    }
  }

  const exportSqlTab = async (id: string) => {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab || tab.type !== 'sql' || !window.NL_PORT) return

    const res = await Neutralino.os.showSaveDialog('Export SQL Query', {
      filters: [{ name: 'SQL Files', extensions: ['sql'] }]
    })
    if (!res) return

    try {
      await Neutralino.filesystem.writeFile(res, tab.query || '')
      tab.filePath = res
      tab.isDirty = false
      // Optional: rename tab to file name
      const filename = res.split(/[\\/]/).pop() || tab.title
      tab.title = filename.replace(/\.sql$/i, '')
    } catch (err) {
      console.error('Failed to export SQL file:', err)
    }
  }

  const closeTab = (id: string) => {
    const tab = tabs.value.find(t => t.id === id)
    if (tab) {
      tab.closed = true
      tab.minimized = false
      if (activeTabId.value === id) {
        const next = mainTabs.value[mainTabs.value.length - 1]
        activeTabId.value = next?.id ?? ''
      }
    }
  }

  const deleteTab = (id: string) => {
    const idx = tabs.value.findIndex((t) => t.id === id)
    if (idx !== -1) {
      tabs.value.splice(idx, 1)
      if (activeTabId.value === id) {
        const next = mainTabs.value[mainTabs.value.length - 1]
        activeTabId.value = next?.id ?? ''
      }
    }
  }

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
    openTable,
    openSqlEditor,
    updateTabQuery,
    saveSqlTab,
    exportSqlTab,
    renameTab,
    loadTabsFromStorage,
    closeTab,
    deleteTab,
    minimizeTab,
    restoreTab,
    showTabSelector,
    selectorConnectionId
  }
})
