import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { useSqlQuery } from './useSqlQuery';
import { BridgeService } from '@/services/bridge';

// Mock BridgeService
vi.mock('@/services/bridge', () => ({
  BridgeService: {
    request: vi.fn()
  }
}));

describe('useSqlQuery', () => {
  let connectionsStore: any;
  let isLoading: any;

  beforeEach(() => {
    vi.clearAllMocks();
    window.NL_PORT = 1234;
    connectionsStore = { activeConnectionId: 'conn-1' };
    isLoading = ref(false);
  });

  it('initializes with default values', () => {
    const sqlQuery = useSqlQuery(connectionsStore);
    expect(sqlQuery.sqlColumns.value).toEqual([]);
    expect(sqlQuery.sqlRows.value).toEqual([]);
    expect(sqlQuery.sqlRowCount.value).toBe(0);
    expect(sqlQuery.sqlLimit.value).toBe(200);
    expect(sqlQuery.sqlMessages.value).toEqual([]);
  });

  describe('runQuery', () => {
    it('calls BridgeService.request with correct parameters', async () => {
      const sqlQuery = useSqlQuery(connectionsStore);
      const mockPayload = {
        rows: [{ id: 1 }],
        fields: [{ name: 'id', dataTypeID: 23 }],
        rowCount: 1,
        executionTime: 5
      };
      vi.mocked(BridgeService.request).mockResolvedValue(mockPayload);

      await sqlQuery.runQuery('SELECT * FROM users', isLoading);

      expect(BridgeService.request).toHaveBeenCalledWith(
        'dbBridge.executeQuery',
        'dbBridge.executeQueryResult',
        {
          connectionId: 'conn-1',
          sql: 'SELECT * FROM users',
          limit: 200
        }
      );

      expect(sqlQuery.sqlRows.value).toEqual(mockPayload.rows);
      expect(sqlQuery.sqlRowCount.value).toBe(1);
      expect(sqlQuery.sqlMessages.value).toHaveLength(1);
      expect(sqlQuery.sqlMessages.value[0]!.type).toBe('info');
    });

    it('handles errors correctly', async () => {
      const sqlQuery = useSqlQuery(connectionsStore);
      vi.mocked(BridgeService.request).mockRejectedValue(new Error('Syntax error'));

      await sqlQuery.runQuery('INVALID SQL', isLoading);

      expect(sqlQuery.sqlMessages.value).toHaveLength(1);
      expect(sqlQuery.sqlMessages.value[0]!.type).toBe('error');
      expect(sqlQuery.sqlMessages.value[0]!.text).toContain('Syntax error');
      expect(isLoading.value).toBe(false);
    });

    it('clears messages before running query', async () => {
      const sqlQuery = useSqlQuery(connectionsStore);
      sqlQuery.sqlMessages.value = [{ type: 'info', text: 'Old message', timestamp: '' }];

      vi.mocked(BridgeService.request).mockResolvedValue({ rows: [], fields: [], rowCount: 0 });

      await sqlQuery.runQuery('SELECT 1', isLoading);

      expect(sqlQuery.sqlMessages.value).toHaveLength(1);
      expect(sqlQuery.sqlMessages.value[0]!.text).not.toContain('Old message');
    });
  });
});
