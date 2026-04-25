import type { Tab } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useTabsStore = defineStore('tabs', () => {
  // Only table tabs in the main tab strip
  const tabs = ref<Tab[]>([])
  const activeTabId = ref<string>('')
  const draggingTabId = ref<string | null>(null)

  // ─── Split View (Bottom Panel) ──────────────────────────────────────────────
  const bottomTabId = ref<string | null>(null)
  const splitRatio = ref(50) // percentage of height for the top panel

  const activeTab = computed(() =>
    tabs.value.find((t) => t.id === activeTabId.value) ?? null,
  )

  const bottomTab = computed(() =>
    bottomTabId.value ? tabs.value.find((t) => t.id === bottomTabId.value) ?? null : null,
  )

  const mainTabs = computed(() => tabs.value.filter(t => !t.minimized))
  const minimizedTabs = computed(() => tabs.value.filter(t => t.minimized))

  function setActiveTab(id: string) {
    activeTabId.value = id
  }

  function minimizeTab(id: string) {
    const tab = tabs.value.find(t => t.id === id)
    if (tab && !tab.minimized) {
      // If the tab is currently in the bottom panel, clear it
      if (bottomTabId.value === id) {
        bottomTabId.value = null
      }
      tab.minimized = true
      // If the minimized tab was the active one, fallback to the last main tab
      if (activeTabId.value === id) {
        const next = mainTabs.value[mainTabs.value.length - 1]
        activeTabId.value = next?.id ?? ''
      }
    }
  }

  function restoreTab(id: string) {
    const tab = tabs.value.find(t => t.id === id)
    if (tab && tab.minimized) {
      tab.minimized = false
      activeTabId.value = tab.id
    }
  }

  // ─── Split View Actions ─────────────────────────────────────────────────────
  function moveTabToBottom(id: string) {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return

    // If minimized, restore it first
    if (tab.minimized) {
      tab.minimized = false
    }

    // If the moved tab was the active tab, switch active to another main tab
    if (activeTabId.value === id) {
      const others = mainTabs.value.filter(t => t.id !== id)
      activeTabId.value = others[others.length - 1]?.id ?? ''
    }

    bottomTabId.value = id
  }

  function closeBottomPanel() {
    bottomTabId.value = null
  }

  function moveBottomToTop() {
    if (bottomTabId.value) {
      activeTabId.value = bottomTabId.value
      bottomTabId.value = null
    }
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
    // If closing the bottom panel tab, clear it
    if (bottomTabId.value === id) {
      bottomTabId.value = null
    }

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
    // Split view
    bottomTabId,
    bottomTab,
    splitRatio,
    setActiveTab,
    openTable,
    openSqlEditor,
    closeTab,
    minimizeTab,
    restoreTab,
    moveTabToBottom,
    closeBottomPanel,
    moveBottomToTop,
  }
})
