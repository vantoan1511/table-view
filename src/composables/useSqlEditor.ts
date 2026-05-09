import { useGridStore } from '@/stores/grid';
import { useTabsStore } from '@/stores/tabs';
import type { Tab } from '@/types';
import { EditorView } from 'codemirror';
import { ref } from 'vue';

export function useSqlEditor(props: { tab: Tab }) {
  const gridStore = useGridStore();
  const tabsStore = useTabsStore();
  const activeResultTab = ref<'results' | 'messages'>('results');

  /**
   * Extract the SQL statement at the current cursor position.
   */
  const getQueryAtCursor = (view: EditorView): string => {
    const doc = view.state.doc.toString();
    const cursor = view.state.selection.main.head;

    const statements: { text: string; start: number; end: number }[] = [];
    let offset = 0;
    const parts = doc.split(';');

    for (let i = 0; i < parts.length; i++) {
      const raw = parts[i];
      if (raw === undefined) continue;

      const end = offset + raw.length + (i < parts.length - 1 ? 1 : 0);
      const trimmed = raw.trim();
      if (trimmed) {
        statements.push({ text: trimmed, start: offset, end });
      }
      offset = end;
    }

    if (statements.length === 0) return '';

    for (const stmt of statements) {
      if (cursor >= stmt.start && cursor <= stmt.end) {
        return stmt.text;
      }
    }

    return statements[statements.length - 1]?.text ?? '';
  };

  const executeRun = (view: EditorView | null) => {
    if (!view) return;

    const selection = view.state.sliceDoc(
      view.state.selection.main.from,
      view.state.selection.main.to
    );

    const queryAtCursor = getQueryAtCursor(view);
    const query = selection.trim() || queryAtCursor || view.state.doc.toString();
    if (!query) return;

    gridStore.runQuery(query, gridStore.sqlLimit, props.tab.connectionId, props.tab.dbName);
    activeResultTab.value = 'results';
  };

  const saveQuery = () => {
    tabsStore.saveSqlTab(props.tab.id);
  };

  const exportQuery = () => {
    tabsStore.exportSqlTab(props.tab.id);
  };

  return {
    activeResultTab,
    executeRun,
    saveQuery,
    exportQuery
  };
}
