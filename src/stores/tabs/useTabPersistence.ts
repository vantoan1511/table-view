// src/stores/tabs/useTabPersistence.ts
import { NativeService } from '@/services/native';
import type { Tab } from '@/types';
import { type Ref } from 'vue';

export function useTabPersistence(tabs: Ref<Tab[]>, activeTabId: Ref<string>) {
  const saveTabsToStorage = async () => {
    try {
      const sqlTabs = tabs.value.filter((t) => t.type === 'sql');
      await NativeService.storage.set('app_tabs', sqlTabs);

      // Only persist activeTabId when it belongs to a sql tab
      const activeIsSql = sqlTabs.some((t) => t.id === activeTabId.value);
      await NativeService.storage.set('app_activeTabId', activeIsSql ? activeTabId.value : '');

      console.log(`[TabsStore] Saved ${sqlTabs.length} SQL tabs to storage`);
    } catch (err) {
      console.error('Failed to save tabs to storage:', err);
    }
  };

  const loadTabsFromStorage = async () => {
    try {
      const sqlOnly = await NativeService.storage.get<Tab[]>('app_tabs');
      if (sqlOnly) {
        tabs.value = sqlOnly;
        console.log(`[TabsStore] Loaded ${sqlOnly.length} SQL tabs from storage`);
      }
      const activeId = await NativeService.storage.get<string>('app_activeTabId');
      if (activeId && tabs.value.some((t) => t.id === activeId)) {
        activeTabId.value = activeId;
      }
    } catch (err) {
      console.log('[TabsStore] No persisted tabs found', err);
    }
  };

  return {
    saveTabsToStorage,
    loadTabsFromStorage
  };
}
