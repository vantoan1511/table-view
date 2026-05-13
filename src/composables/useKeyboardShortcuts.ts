import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useLayoutStore } from '@/stores/layout';
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';
import { onMounted, onUnmounted } from 'vue';

export function useKeyboardShortcuts() {
  const tabsStore = useTabsStore();
  const gridStore = useGridStore();
  const connectionsStore = useConnectionsStore();
  const layoutStore = useLayoutStore();
  const schemaStore = useSchemaStore();

  const handleKeydown = (e: KeyboardEvent) => {
    const isMod = e.ctrlKey || e.metaKey;
    const isFKey = e.key.startsWith('F') && e.key.length > 1;

    // Whitelist for essential system shortcuts (Copy, Paste, etc.)
    const systemShortcuts = ['c', 'v', 'x', 'a', 'z', 'y'];
    if (isMod && systemShortcuts.includes(e.key.toLowerCase())) {
      return;
    }

    // ─── App Shortcuts ────────────────────────────────────────────────────────

    // Ctrl+W: Close active tab
    if (isMod && e.key.toLowerCase() === 'w') {
      e.preventDefault();
      if (tabsStore.activeTabId) {
        tabsStore.closeTab(tabsStore.activeTabId);
      }
      return;
    }

    // Ctrl+K: Focus search
    if (isMod && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      const searchInput = document.querySelector(
        'input[placeholder*="Search"]'
      ) as HTMLInputElement;
      if (searchInput) {
        searchInput.focus();
      }
      return;
    }

    // Ctrl+R: Refresh data
    if (isMod && e.key.toLowerCase() === 'r') {
      e.preventDefault();
      const tab = tabsStore.activeTab;
      if (tab?.type === 'table' && tab.tableName) {
        gridStore.loadTable(tab.tableName, tab.connectionId, tab.schema, tab.dbName);
      }
      return;
    }

    // Ctrl+N: New Connection
    if (isMod && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      connectionsStore.toggleConnectionModal(true);
      return;
    }

    // Ctrl+J: Toggle Console
    if (isMod && e.key.toLowerCase() === 'j') {
      e.preventDefault();
      layoutStore.togglePanel('console');
      return;
    }

    // Ctrl+I: Toggle Inspector
    if (isMod && e.key.toLowerCase() === 'i') {
      e.preventDefault();
      layoutStore.togglePanel('inspector');
      return;
    }

    // F5: Refresh
    if (e.key === 'F5') {
      e.preventDefault();
      const tab = tabsStore.activeTab;
      if (tab?.type === 'table' && tab.tableName) {
        gridStore.loadTable(tab.tableName, tab.connectionId, tab.schema, tab.dbName);
      } else if (connectionsStore.activeConnectionId) {
        // Fallback to refreshing the tree for active connection
        schemaStore.loadSchema(undefined, connectionsStore.activeConnectionId);
      }
      return;
    }

    // ─── Block All Other Defaults ─────────────────────────────────────────────

    // Disable all other browser shortcuts (e.g., Ctrl+P, Ctrl+S, F-keys)
    // but allow normal typing and navigation (Arrows, Enter, etc.)
    if (isMod || isFKey) {
      e.preventDefault();
    }
  };

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
}
