import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { useSchemaActions } from './useSchemaActions';
import { BridgeService } from '@/services/bridge';

vi.mock('@/services/bridge', () => ({
  BridgeService: {
    request: vi.fn()
  }
}));

describe('useSchemaActions', () => {
  let cache: any;
  let treeState: any;
  let connectionsStore: any;
  let loadedAllDatabases: any;

  beforeEach(() => {
    vi.clearAllMocks();
    cache = {
      loadingByConnection: ref({}),
      schemasByConnection: ref({}),
      loadingDbByConnection: ref({}),
      perDbSchemas: ref({}),
      errorDbByConnection: ref({})
    };
    treeState = {
      selectedSchemaByConnection: ref({}),
      expandedSchemasByConnection: ref({}),
      setSchemaExpanded: vi.fn((connId, name, val) => {
        if (!treeState.expandedSchemasByConnection.value[connId])
          treeState.expandedSchemasByConnection.value[connId] = {};
        treeState.expandedSchemasByConnection.value[connId][name] = val;
      })
    };
    connectionsStore = {
      activeConnectionId: 'conn-1',
      connections: [
        { id: 'conn-1', type: 'postgresql', username: 'postgres', displayAllDatabases: false }
      ]
    };
    loadedAllDatabases = ref(false);
  });

  it('sameSchema compares strings correctly (accent insensitive)', () => {
    const actions = useSchemaActions(cache, treeState, connectionsStore, loadedAllDatabases);
    expect(actions.sameSchema('public', 'PUBLIC')).toBe(true);
    expect(actions.sameSchema('Schema1', 'schema1')).toBe(true);
  });

  it('resolveFallbackSchema returns correct defaults', () => {
    const actions = useSchemaActions(cache, treeState, connectionsStore, loadedAllDatabases);
    expect(actions.resolveFallbackSchema('postgresql', 'user')).toBe('public');
    expect(actions.resolveFallbackSchema('oracle', 'MYUSER')).toBe('MYUSER');
    expect(actions.resolveFallbackSchema('mysql', 'root')).toBe('');
  });

  describe('loadSchema', () => {
    it('calls dbBridge.getSchema and updates cache', async () => {
      const actions = useSchemaActions(cache, treeState, connectionsStore, loadedAllDatabases);
      const mockResponse = {
        schema: {
          tables: [{ name: 'users', schema: 'public' }],
          views: [],
          functions: [],
          schemas: ['public', 'information_schema'],
          databases: ['postgres']
        }
      };
      vi.mocked(BridgeService.request).mockResolvedValue(mockResponse);

      await actions.loadSchema();

      expect(BridgeService.request).toHaveBeenCalledWith(
        'dbBridge.getSchema',
        'dbBridge.getSchemaResult',
        expect.objectContaining({ connectionId: 'conn-1' })
      );

      expect(cache.schemasByConnection.value['conn-1']).toBeDefined();
      expect(cache.schemasByConnection.value['conn-1'].tables).toHaveLength(1);
      expect(treeState.selectedSchemaByConnection.value['conn-1']).toBe('public');
    });
  });

  describe('loadDbSchema', () => {
    it('loads schema for a specific database', async () => {
      const actions = useSchemaActions(cache, treeState, connectionsStore, loadedAllDatabases);
      const mockResponse = {
        tables: [{ name: 'logs', schema: 'public' }],
        views: [],
        functions: [],
        schemas: ['public']
      };
      vi.mocked(BridgeService.request).mockResolvedValue(mockResponse);

      await actions.loadDbSchema('conn-1', 'logs_db');

      expect(BridgeService.request).toHaveBeenCalledWith(
        'dbBridge.getDbSchema',
        'dbBridge.getDbSchemaResult',
        { connectionId: 'conn-1', targetDatabase: 'logs_db' }
      );

      expect(cache.perDbSchemas.value['conn-1']['logs_db']).toBeDefined();
      expect(cache.perDbSchemas.value['conn-1']['logs_db'].tables[0].name).toBe('logs');
    });
  });
});
