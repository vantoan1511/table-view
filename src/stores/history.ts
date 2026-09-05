import type { HistoryEntry } from '@/types';
import { storage } from '@/services/nativeService';
import { defineStore } from 'pinia';
import { ref } from 'vue';

import { useConnectionsStore } from './connections';

const MAX_HISTORY_LIMIT = 500;

export const useHistoryStore = defineStore('history', () => {
  const connectionsStore = useConnectionsStore();
  const history = ref<HistoryEntry[]>([]);

  const loadHistory = async () => {
    if (window.NL_PORT) {
      try {
        const data = await storage.getData('query_history');
        const loaded = JSON.parse(data) as HistoryEntry[];
        history.value = loaded;
      } catch (_err) {
        history.value = [];
      }
    }
  };

  const saveHistory = async () => {
    if (window.NL_PORT) {
      try {
        await storage.setData('query_history', JSON.stringify(history.value));
      } catch (err) {
        console.error('Failed to save query history:', err);
      }
    }
  };

  const addSqlEntry = async (payload: {
    query: string;
    connectionId?: string;
    dbName?: string;
    success: boolean;
    executionTime?: number;
    rowCount?: number;
    error?: string;
  }) => {
    const conn = connectionsStore.connections.find((c) => c.id === payload.connectionId);

    // Truncate extremely large SQL queries to prevent bloating the NeutralinoJS filesystem storage
    const MAX_QUERY_LENGTH = 10000;
    const sanitizedQuery =
      payload.query.length > MAX_QUERY_LENGTH
        ? payload.query.slice(0, MAX_QUERY_LENGTH) + '\n... (truncated for history)'
        : payload.query;

    const entry: HistoryEntry = {
      id: `history-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      type: 'sql',
      timestamp: new Date().toISOString(),
      connectionId: payload.connectionId,
      connectionName: conn?.name || 'Unknown',
      connectionColor: conn?.color || 'gray',
      dbName: payload.dbName,
      query: sanitizedQuery,
      success: payload.success,
      executionTime: payload.executionTime,
      rowCount: payload.rowCount,
      error: payload.error
    };

    history.value.unshift(entry);

    if (history.value.length > MAX_HISTORY_LIMIT) {
      history.value = history.value.slice(0, MAX_HISTORY_LIMIT);
    }

    await saveHistory();
  };

  const addTableEntry = async (payload: {
    tableName: string;
    schema?: string;
    connectionId?: string;
    dbName?: string;
  }) => {
    const conn = connectionsStore.connections.find((c) => c.id === payload.connectionId);
    const entry: HistoryEntry = {
      id: `history-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      type: 'table',
      timestamp: new Date().toISOString(),
      connectionId: payload.connectionId,
      connectionName: conn?.name || 'Unknown',
      connectionColor: conn?.color || 'gray',
      dbName: payload.dbName,
      schema: payload.schema,
      tableName: payload.tableName
    };

    // Avoid duplicate adjacent identical table views within 5 seconds to prevent spam
    const lastEntry = history.value[0];
    if (
      lastEntry &&
      lastEntry.type === 'table' &&
      lastEntry.tableName === payload.tableName &&
      lastEntry.schema === payload.schema &&
      lastEntry.connectionId === payload.connectionId &&
      lastEntry.dbName === payload.dbName &&
      Date.now() - new Date(lastEntry.timestamp).getTime() < 5000
    ) {
      return;
    }

    history.value.unshift(entry);

    if (history.value.length > MAX_HISTORY_LIMIT) {
      history.value = history.value.slice(0, MAX_HISTORY_LIMIT);
    }

    await saveHistory();
  };

  const clearHistory = async () => {
    history.value = [];
    await saveHistory();
  };

  return {
    history,
    loadHistory,
    saveHistory,
    addSqlEntry,
    addTableEntry,
    clearHistory
  };
});
