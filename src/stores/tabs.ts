import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Tab, TabType } from '@/types'

export const useTabsStore = defineStore('tabs', () => {
  // Only table tabs in the main tab strip
  const tabs = ref<Tab[]>([
    { id: 'tab-users', type: 'table', title: 'users', tableName: 'users' },
    { id: 'tab-orders', type: 'table', title: 'orders', tableName: 'orders' },
  ])
  const activeTabId = ref<string>('tab-users')

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
    closeTab,
  }
})
