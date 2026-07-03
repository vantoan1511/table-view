import { useGridStore } from '@/stores/grid';
import { useTabsStore } from '@/stores/tabs';
import type { Tab } from '@/types';
import { isDestructiveQuery } from '@/utils/sqlGuard';
import { EditorView } from 'codemirror';
import { ref, computed } from 'vue';

export function useSqlEditor(props: { tab: Tab }) {
  const gridStore = useGridStore();
  const tabsStore = useTabsStore();
  const activeResultTab = ref<'results' | 'messages'>('results');

  const showDestructiveConfirm = ref(false);
  const pendingQuery = ref('');
  const hasActiveTransaction = ref(false);

  const autoCommit = computed({
    get: () => props.tab.autoCommit ?? true,
    set: (val) => tabsStore.updateTabAutoCommit(props.tab.id, val)
  });

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

  const showTransactionConfirm = ref(false);

  const runQuery = async (query: string) => {
    const isAuto = autoCommit.value;
    const success = await gridStore.runQuery(
      query,
      gridStore.sqlLimit,
      props.tab.connectionId,
      props.tab.dbName,
      isAuto
    );
    activeResultTab.value = 'results';
    if (success && !isAuto) {
      hasActiveTransaction.value = true;
      showTransactionConfirm.value = true;
    }
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

    if (isDestructiveQuery(query)) {
      pendingQuery.value = query;
      showDestructiveConfirm.value = true;
      return;
    }

    runQuery(query);
  };

  const confirmDestructive = () => {
    if (pendingQuery.value) {
      runQuery(pendingQuery.value);
      pendingQuery.value = '';
    }
    showDestructiveConfirm.value = false;
  };

  const cancelDestructive = () => {
    pendingQuery.value = '';
    showDestructiveConfirm.value = false;
  };

  const commitTx = async () => {
    try {
      await gridStore.commitTransaction(props.tab.connectionId, props.tab.dbName);
    } finally {
      hasActiveTransaction.value = false;
      showTransactionConfirm.value = false;
    }
  };

  const rollbackTx = async () => {
    try {
      await gridStore.rollbackTransaction(props.tab.connectionId, props.tab.dbName);
    } finally {
      hasActiveTransaction.value = false;
      showTransactionConfirm.value = false;
    }
  };

  const saveQuery = () => {
    tabsStore.saveSqlTab(props.tab.id);
  };

  const exportQuery = () => {
    tabsStore.exportSqlTab(props.tab.id);
  };

  return {
    activeResultTab,
    autoCommit,
    showDestructiveConfirm,
    showTransactionConfirm,
    hasActiveTransaction,
    executeRun,
    confirmDestructive,
    cancelDestructive,
    commitTx,
    rollbackTx,
    saveQuery,
    exportQuery
  };
}
