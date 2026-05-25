import { setActivePinia, createPinia } from 'pinia';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useDiagramStore } from '../diagram';
import { BridgeService } from '@/services/bridge';

// Mock BridgeService
vi.mock('@/services/bridge', () => ({
  BridgeService: {
    request: vi.fn()
  }
}));

describe('Diagram Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('initializes with empty states', () => {
    const store = useDiagramStore();
    expect(store.diagrams).toEqual({});
    expect(store.loading).toEqual({});
    expect(store.errors).toEqual({});
  });

  it('creates correct cache keys', () => {
    const store = useDiagramStore();

    // Without dbName
    const key1 = store.getCacheKey('conn-1', 'public');
    expect(key1).toBe('conn-1-default-public');

    // With dbName
    const key2 = store.getCacheKey('conn-1', 'public', 'postgres');
    expect(key2).toBe('conn-1-postgres-public');
  });

  it('clears cached data correctly', () => {
    const store = useDiagramStore();
    const key = 'conn-1-default-public';

    store.diagrams[key] = { tables: [], relations: [] };
    store.loading[key] = true;
    store.errors[key] = 'Some error';

    store.clearCache('conn-1', 'public');

    expect(store.diagrams[key]).toBeUndefined();
    expect(store.loading[key]).toBeUndefined();
    expect(store.errors[key]).toBeUndefined();
  });

  it('successfully fetches schema details via BridgeService and updates cache', async () => {
    const store = useDiagramStore();
    const mockPayload = {
      tables: [
        {
          name: 'users',
          columns: [
            { name: 'id', dataType: 'integer', isPrimaryKey: true, nullable: false },
            { name: 'name', dataType: 'text', isPrimaryKey: false, nullable: true }
          ]
        }
      ],
      relations: [
        {
          constraintName: 'fk_users_profile',
          sourceTable: 'profiles',
          sourceColumn: 'user_id',
          targetTable: 'users',
          targetColumn: 'id'
        }
      ]
    };

    vi.mocked(BridgeService.request).mockResolvedValue(mockPayload);

    const result = await store.fetchSchemaDetails('conn-1', 'public');

    expect(BridgeService.request).toHaveBeenCalledWith(
      'dbBridge.getSchemaDetails',
      'dbBridge.getSchemaDetailsResult',
      {
        connectionId: 'conn-1',
        schemaName: 'public',
        targetDatabase: undefined
      }
    );

    expect(result).not.toBeNull();
    expect(result?.tables.length).toBe(1);
    expect(result?.tables[0].name).toBe('users');
    expect(result?.tables[0].columns[0].name).toBe('id');
    expect(result?.tables[0].columns[0].isPrimaryKey).toBe(true);
    expect(result?.relations.length).toBe(1);
    expect(result?.relations[0].constraintName).toBe('fk_users_profile');

    const key = 'conn-1-default-public';
    expect(store.diagrams[key]).toEqual(result);
    expect(store.loading[key]).toBe(false);
    expect(store.errors[key]).toBe(null);
  });

  it('handles error response from BridgeService', async () => {
    const store = useDiagramStore();

    vi.mocked(BridgeService.request).mockRejectedValue(new Error('Connection failure'));

    const result = await store.fetchSchemaDetails('conn-1', 'public');

    expect(result).toBeNull();

    const key = 'conn-1-default-public';
    expect(store.diagrams[key]).toBeUndefined();
    expect(store.loading[key]).toBe(false);
    expect(store.errors[key]).toBe('Connection failure');
  });
});
