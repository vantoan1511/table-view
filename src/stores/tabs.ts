import type { Tab } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { useConnectionsStore } from './connections'
import { useSchemaStore } from './schema'

export const useTabsStore = defineStore('tabs', () => {
  const connectionsStore = useConnectionsStore()
  const schemaStore = useSchemaStore()
  // Only table tabs in the main tab strip
  const tabs = ref<Tab[]>([])
  const activeTabId = ref<string>('')
  const draggingTabId = ref<string | null>(null)

  const activeTab = computed(() =>
    tabs.value.find((t) => t.id === activeTabId.value) ?? null,
  )

  const mainTabs = computed(() => tabs.value.filter(t => !t.minimized))
  const minimizedTabs = computed(() => tabs.value.filter(t => t.minimized))

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
      const [movedTab] = tabs.value.splice(fromIdx, 1)
      tabs.value.splice(toIdx, 0, movedTab)
    }
  }

  const openTable = (tableName: string, schemaName?: string) => {
    const targetSchema = schemaName || schemaStore.selectedSchema
    const existing = tabs.value.find(
      (t) => t.type === 'table' && t.tableName === tableName && t.schema === targetSchema,
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
      connectionId: connectionsStore.activeConnectionId ?? undefined,
      schema: targetSchema,
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }

  const openSqlEditor = () => {
    const tab: Tab = {
      id: `tab-sql-${Date.now()}`,
      type: 'sql',
      title: 'Query Console',
      connectionId: connectionsStore.activeConnectionId ?? undefined,
      schema: schemaStore.selectedSchema,
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }

  const closeTab = (id: string) => {

    const idx = tabs.value.findIndex((t) => t.id === id)
    if (idx === -1) return
    tabs.value.splice(idx, 1)
    if (activeTabId.value === id) {
      const next = tabs.value[Math.min(idx, tabs.value.length - 1)]
      activeTabId.value = next?.id ?? ''
    }
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    draggingTabId,
    mainTabs,
    minimizedTabs,
    setActiveTab,
    reorderTab,
    openTable,
    openSqlEditor,
    closeTab,
    minimizeTab,
    restoreTab,
  }
})
