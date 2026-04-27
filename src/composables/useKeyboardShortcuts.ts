import { onMounted, onUnmounted } from 'vue'
import { useTabsStore } from '@/stores/tabs'
import { useGridStore } from '@/stores/grid'
import { useConnectionsStore } from '@/stores/connections'

export function useKeyboardShortcuts() {
  const tabsStore = useTabsStore()
  const gridStore = useGridStore()
  const connectionsStore = useConnectionsStore()

  const handleKeydown = (e: KeyboardEvent) => {
    const isMod = e.ctrlKey || e.metaKey
    const target = e.target as HTMLElement
    const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable

    // Global shortcuts that should work even in inputs (if any)
    // ...

    // Shortcuts that should NOT work when typing
    if (isInput) return

    // Ctrl+W: Close active tab
    if (isMod && e.key === 'w') {
      e.preventDefault()
      if (tabsStore.activeTabId) {
        tabsStore.closeTab(tabsStore.activeTabId)
      }
    }

    // Ctrl+K: Focus search
    if (isMod && e.key === 'k') {
      e.preventDefault()
      const searchInput = document.querySelector('input[placeholder*="Search"]') as HTMLInputElement
      if (searchInput) {
        searchInput.focus()
      }
    }

    // Ctrl+R: Refresh data
    if (isMod && e.key === 'r') {
      e.preventDefault()
      if (tabsStore.activeTab?.type === 'table' && tabsStore.activeTab.tableName) {
        gridStore.loadTable(tabsStore.activeTab.tableName)
      }
    }

    // Ctrl+N: New Connection
    if (isMod && e.key === 'n') {
      e.preventDefault()
      connectionsStore.toggleConnectionModal(true)
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
  })
}
