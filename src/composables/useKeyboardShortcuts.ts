import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useLayoutStore } from '@/stores/layout';
import { usePreferencesStore } from '@/stores/preferences';
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';
import { onMounted, onUnmounted } from 'vue';

export function matchesShortcut(e: KeyboardEvent, keys: string[] | undefined): boolean {
  if (!keys || keys.length === 0) return false;

  const hasMod = keys.some((k) => ['Mod', 'Ctrl', 'Cmd'].includes(k));
  const hasShift = keys.includes('Shift');
  const hasAlt = keys.includes('Alt');

  const reqMod = e.ctrlKey || e.metaKey;
  if (hasMod !== reqMod) return false;
  if (hasShift !== e.shiftKey) return false;
  if (hasAlt !== e.altKey) return false;

  const mainKey = keys.find((k) => !['Mod', 'Ctrl', 'Cmd', 'Shift', 'Alt'].includes(k));
  if (!mainKey) return false;

  if (mainKey === '/' && (e.key === '/' || e.key === '?')) return true;

  return e.key.toLowerCase() === mainKey.toLowerCase();
}

export function useKeyboardShortcuts() {
  const tabsStore = useTabsStore();
  const gridStore = useGridStore();
  const connectionsStore = useConnectionsStore();
  const layoutStore = useLayoutStore();
  const schemaStore = useSchemaStore();
  const preferencesStore = usePreferencesStore();

  const handleKeydown = (e: KeyboardEvent) => {
    const isMod = e.ctrlKey || e.metaKey;
    const isFKey = e.key.startsWith('F') && e.key.length > 1;

    // Whitelist for essential system shortcuts (Copy, Paste, etc.)
    const systemShortcuts = ['c', 'v', 'x', 'a', 'z', 'y'];
    if (isMod && systemShortcuts.includes(e.key.toLowerCase())) {
      return;
    }

    const s = preferencesStore.settings.shortcuts;

    // ─── App Shortcuts ────────────────────────────────────────────────────────

    // Open Preferences
    if (matchesShortcut(e, s?.openPreferences)) {
      e.preventDefault();
      preferencesStore.open('general');
      return;
    }

    // Open Shortcuts Tab
    if (matchesShortcut(e, s?.openShortcuts)) {
      e.preventDefault();
      preferencesStore.open('shortcuts');
      return;
    }

    // Close active tab
    if (matchesShortcut(e, s?.closeTab)) {
      e.preventDefault();
      if (tabsStore.activeTabId) {
        tabsStore.closeTab(tabsStore.activeTabId);
      }
      return;
    }

    // Focus search
    if (matchesShortcut(e, s?.focusSearch)) {
      e.preventDefault();
      const searchInput = document.querySelector(
        'input[placeholder*="Search"]'
      ) as HTMLInputElement;
      if (searchInput) {
        searchInput.focus();
      }
      return;
    }

    // Refresh data
    if (matchesShortcut(e, s?.refreshData)) {
      e.preventDefault();
      const tab = tabsStore.activeTab;
      if (tab?.type === 'table' && tab.tableName) {
        gridStore.loadTable(tab.tableName, tab.connectionId, tab.schema, tab.dbName);
      }
      return;
    }

    // New Connection
    if (matchesShortcut(e, s?.newConnection)) {
      e.preventDefault();
      connectionsStore.toggleConnectionModal(true);
      return;
    }

    // Toggle Sidebar
    if (matchesShortcut(e, s?.toggleSidebar)) {
      e.preventDefault();
      layoutStore.toggleSidebar();
      return;
    }

    // Toggle Console
    if (matchesShortcut(e, s?.toggleConsole)) {
      e.preventDefault();
      layoutStore.togglePanel('console');
      return;
    }

    // Toggle Inspector
    if (matchesShortcut(e, s?.toggleInspector)) {
      e.preventDefault();
      layoutStore.togglePanel('inspector');
      return;
    }

    // Refresh Schema / Table (Alternative / F5)
    if (matchesShortcut(e, s?.refreshAlternative)) {
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
