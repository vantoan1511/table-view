import { useHistoryStore } from '../history';
import { useConnectionsStore } from '../connections';
import * as Neutralino from '@neutralinojs/lib';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// Mock Neutralino.storage
vi.mock('@neutralinojs/lib', () => ({
  storage: {
    getData: vi.fn(),
    setData: vi.fn()
  }
}));

describe('History Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();

    // Setup window.NL_PORT for test suite context
    (window as any).NL_PORT = '8080';
  });

  it('initializes with empty history', () => {
    const store = useHistoryStore();
    expect(store.history).toEqual([]);
  });

  it('successfully loads history from Neutralino storage', async () => {
    const mockHistory = [
      {
        id: 'test-1',
        type: 'sql' as const,
        query: 'SELECT * FROM users',
        timestamp: new Date().toISOString(),
        success: true
      }
    ];

    vi.mocked(Neutralino.storage.getData).mockResolvedValue(JSON.stringify(mockHistory));

    const store = useHistoryStore();
    await store.loadHistory();

    expect(Neutralino.storage.getData).toHaveBeenCalledWith('query_history');
    expect(store.history).toEqual(mockHistory);
  });

  it('initializes empty history when Neutralino storage fails', async () => {
    vi.mocked(Neutralino.storage.getData).mockRejectedValue(new Error('Storage not found'));

    const store = useHistoryStore();
    await store.loadHistory();

    expect(store.history).toEqual([]);
  });

  it('successfully adds and persists an SQL entry', async () => {
    const store = useHistoryStore();
    const connectionsStore = useConnectionsStore();

    connectionsStore.connections = [
      {
        id: 'conn-1',
        name: 'PostgreSQL Local',
        type: 'postgresql' as any,
        host: 'localhost',
        port: 5432,
        database: 'postgres',
        username: 'postgres',
        password: '',
        color: 'indigo' as any,
        savePassword: true,
        displayAllDatabases: false,
        isConnected: true
      }
    ];

    await store.addSqlEntry({
      query: 'SELECT 1;',
      connectionId: 'conn-1',
      dbName: 'postgres',
      success: true,
      executionTime: 5,
      rowCount: 1
    });

    expect(store.history.length).toBe(1);
    expect(store.history[0].query).toBe('SELECT 1;');
    expect(store.history[0].type).toBe('sql');
    expect(store.history[0].connectionName).toBe('PostgreSQL Local');
    expect(store.history[0].connectionColor).toBe('indigo');
    expect(store.history[0].success).toBe(true);

    expect(Neutralino.storage.setData).toHaveBeenCalled();
  });

  it('successfully adds a Table load entry', async () => {
    const store = useHistoryStore();
    const connectionsStore = useConnectionsStore();

    connectionsStore.connections = [
      {
        id: 'conn-1',
        name: 'PostgreSQL Local',
        type: 'postgresql' as any,
        host: 'localhost',
        port: 5432,
        database: 'postgres',
        username: 'postgres',
        password: '',
        color: 'indigo' as any,
        savePassword: true,
        displayAllDatabases: false,
        isConnected: true
      }
    ];

    await store.addTableEntry({
      tableName: 'users',
      schema: 'public',
      connectionId: 'conn-1',
      dbName: 'postgres'
    });

    expect(store.history.length).toBe(1);
    expect(store.history[0].tableName).toBe('users');
    expect(store.history[0].schema).toBe('public');
    expect(store.history[0].type).toBe('table');

    expect(Neutralino.storage.setData).toHaveBeenCalled();
  });

  it('deduplicates adjacent identical table load entries within 5 seconds', async () => {
    const store = useHistoryStore();

    await store.addTableEntry({
      tableName: 'users',
      schema: 'public',
      connectionId: 'conn-1',
      dbName: 'postgres'
    });

    // Attempt immediately adding same table
    await store.addTableEntry({
      tableName: 'users',
      schema: 'public',
      connectionId: 'conn-1',
      dbName: 'postgres'
    });

    expect(store.history.length).toBe(1);
  });

  it('limits history elements to the maximum cap of 500', async () => {
    const store = useHistoryStore();

    // Populate history with 500 items
    for (let i = 0; i < 500; i++) {
      store.history.push({
        id: `id-${i}`,
        type: 'sql',
        query: `SELECT ${i};`,
        timestamp: new Date().toISOString()
      });
    }

    // Add another item
    await store.addSqlEntry({
      query: 'SELECT OVERFLOW;',
      connectionId: 'conn-1',
      success: true
    });

    expect(store.history.length).toBe(500);
    expect(store.history[0].query).toBe('SELECT OVERFLOW;');
  });

  it('clears history correctly', async () => {
    const store = useHistoryStore();
    store.history = [
      {
        id: '1',
        type: 'sql',
        query: 'SELECT 1;',
        timestamp: new Date().toISOString()
      }
    ];

    await store.clearHistory();

    expect(store.history).toEqual([]);
    expect(Neutralino.storage.setData).toHaveBeenCalledWith('query_history', '[]');
  });
});
