import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { useTableData } from './useTableData';
import { BridgeService } from '@/services/bridge';

// Mock BridgeService
vi.mock('@/services/bridge', () => ({
  BridgeService: {
    request: vi.fn()
  }
}));

describe('useTableData', () => {
  let connectionsStore: any;
  let isLoading: any;

  beforeEach(() => {
    vi.clearAllMocks();
    window.NL_PORT = 1234; // Simulate Neutralino environment

    connectionsStore = {
      activeConnectionId: 'conn-1',
      connections: [
        { id: 'conn-1', name: 'Postgres', type: 'postgres' },
        { id: 'conn-2', name: 'Oracle', type: 'oracle' }
      ]
    };
    isLoading = ref(false);
  });

  it('initializes with default values', () => {
    const tableData = useTableData(connectionsStore);
    expect(tableData.columns.value).toEqual([]);
    expect(tableData.rows.value).toEqual([]);
    expect(tableData.totalRows.value).toBe(0);
    expect(tableData.currentPage.value).toBe(1);
    expect(tableData.rowsPerPage.value).toBe(100);
  });

  describe('resolveBackendTableName', () => {
    it('returns table name as is for non-schema-qualified connections', () => {
      const tableData = useTableData(connectionsStore);
      const result = tableData.resolveBackendTableName('users', 'conn-1');
      expect(result).toBe('users');
    });

    it('returns schema.table for postgres connections when schema is provided', () => {
      const tableData = useTableData(connectionsStore);
      const result = tableData.resolveBackendTableName('users', 'conn-1', 'sales');
      expect(result).toBe('sales.users');
    });

    it('returns schema.table for oracle connections when schema is provided', () => {
      const tableData = useTableData(connectionsStore);
      const result = tableData.resolveBackendTableName('users', 'conn-2', 'HR');
      expect(result).toBe('HR.users');
    });

    it('uses activeTableSchema if schemaName is not provided for oracle', () => {
      const tableData = useTableData(connectionsStore);
      tableData.activeTableSchema.value = 'SALES';
      const result = tableData.resolveBackendTableName('orders', 'conn-2');
      expect(result).toBe('SALES.orders');
    });
  });

  describe('toggleSort', () => {
    it('cycles through sort states: none -> asc -> desc -> none', async () => {
      const tableData = useTableData(connectionsStore);
      tableData.activeTableName.value = 'users';

      vi.mocked(BridgeService.request).mockResolvedValue({ rows: [], fields: [], totalCount: 0 });

      // None -> Asc
      await tableData.toggleSort('id', isLoading);
      expect(tableData.sortColumn.value).toBe('id');
      expect(tableData.sortDirection.value).toBe('asc');
      expect(BridgeService.request).toHaveBeenCalled();

      // Asc -> Desc
      await tableData.toggleSort('id', isLoading);
      expect(tableData.sortColumn.value).toBe('id');
      expect(tableData.sortDirection.value).toBe('desc');

      // Desc -> None
      await tableData.toggleSort('id', isLoading);
      expect(tableData.sortColumn.value).toBeUndefined();
      expect(tableData.sortDirection.value).toBeUndefined();
    });

    it('switches to new column with asc direction', async () => {
      const tableData = useTableData(connectionsStore);
      tableData.activeTableName.value = 'users';
      vi.mocked(BridgeService.request).mockResolvedValue({ rows: [], fields: [], totalCount: 0 });

      await tableData.toggleSort('id', isLoading);
      await tableData.toggleSort('name', isLoading);

      expect(tableData.sortColumn.value).toBe('name');
      expect(tableData.sortDirection.value).toBe('asc');
    });
  });

  describe('loadTable', () => {
    it('calls BridgeService.request with correct parameters', async () => {
      const tableData = useTableData(connectionsStore);
      const mockPayload = {
        rows: [{ id: 1, name: 'Test' }],
        fields: [
          { name: 'id', dataTypeID: 23, isPrimaryKey: true },
          { name: 'name', dataTypeID: 1043 }
        ],
        totalCount: 1,
        executionTime: 10
      };

      vi.mocked(BridgeService.request).mockResolvedValue(mockPayload);

      await tableData.loadTable('users', isLoading);

      expect(BridgeService.request).toHaveBeenCalledWith(
        'dbBridge.fetchTableData',
        'dbBridge.fetchTableDataResult',
        expect.objectContaining({
          connectionId: 'conn-1',
          tableName: 'users',
          limit: 100,
          offset: 0
        })
      );

      expect(tableData.rows.value).toEqual(mockPayload.rows);
      expect(tableData.columns.value).toHaveLength(2);
      expect(tableData.columns.value[0]).toEqual({
        name: 'id',
        dataType: '23',
        isPrimaryKey: true
      });
      expect(tableData.totalRows.value).toBe(1);
      expect(tableData.executionTime.value).toBe(10);
    });

    it('sets loading state during request', async () => {
      const tableData = useTableData(connectionsStore);
      let loadingInProcess = false;

      vi.mocked(BridgeService.request).mockImplementation(async () => {
        loadingInProcess = isLoading.value;
        return { rows: [], fields: [], totalCount: 0 };
      });

      await tableData.loadTable('users', isLoading);

      expect(loadingInProcess).toBe(true);
      expect(isLoading.value).toBe(false);
    });
  });
});
