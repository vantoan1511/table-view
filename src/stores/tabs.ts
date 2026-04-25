import type { Tab } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useTabsStore = defineStore('tabs', () => {
  // Only table tabs in the main tab strip
  const tabs = ref<Tab[]>([])
  const activeTabId = ref<string>('')

  const activeTab = computed(() =>
    tabs.value.find((t) => t.id === activeTabId.value) ?? null,
  )

  function setActiveTab(id: string) {
    activeTabId.value = id
  }

  function openTable(tableName: string) {
    const existing = tabs.value.find(
      (t) => t.type === 'table' && t.tableName === tableName,
    )
    if (existing) {
      activeTabId.value = existing.id
      return
    }
    const tab: Tab = {
      id: `tab-${tableName}-${Date.now()}`,
      type: 'table',
      title: tableName,
      tableName,
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }

  function openSqlEditor() {
    const tab: Tab = {
      id: `tab-sql-${Date.now()}`,
      type: 'sql',
      title: 'Query Console',
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }

  function closeTab(id: string) {
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
    setActiveTab,
    openTable,
    openSqlEditor,
    closeTab,
  }
})
