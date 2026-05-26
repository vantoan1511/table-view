import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { setActivePinia, createPinia } from 'pinia';
import { useTableData } from './useTableData';
import { useTabsStore } from '../tabs';
import { BridgeService } from '@/services/bridge';
import { TabType } from '@/types';

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
    setActivePinia(createPinia());
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

    it('saves connectionId and dbName and reuses them on subsequent load/sort', async () => {
      const tableData = useTableData(connectionsStore);
      vi.mocked(BridgeService.request).mockResolvedValue({ rows: [], fields: [], totalCount: 0 });

      // Load table with explicit connectionId and dbName
      await tableData.loadTable('users', isLoading, 'conn-2', undefined, 'custom_db');

      expect(tableData.activeConnectionId.value).toBe('conn-2');
      expect(tableData.activeDbName.value).toBe('custom_db');
      expect(BridgeService.request).toHaveBeenLastCalledWith(
        'dbBridge.fetchTableData',
        'dbBridge.fetchTableDataResult',
        expect.objectContaining({
          connectionId: 'conn-2',
          targetDatabase: 'custom_db'
        })
      );

      // Now toggle sort — this calls loadTable internally without passing connectionId or dbName
      await tableData.toggleSort('id', isLoading);

      // It should reuse the saved values
      expect(BridgeService.request).toHaveBeenLastCalledWith(
        'dbBridge.fetchTableData',
        'dbBridge.fetchTableDataResult',
        expect.objectContaining({
          connectionId: 'conn-2',
          targetDatabase: 'custom_db'
        })
      );
    });

    it('persists and restores sort/filter/page state per tab', async () => {
      const tableData = useTableData(connectionsStore);
      const tabsStore = useTabsStore();
      vi.mocked(BridgeService.request).mockResolvedValue({ rows: [], fields: [], totalCount: 0 });

      // Create two mocked tab objects
      const usersTab = {
        id: 'tab-users',
        type: TabType.TABLE,
        title: 'users',
        tableName: 'users',
        connectionId: 'conn-1',
        schema: undefined,
        dbName: undefined,
        sortColumn: undefined,
        sortDirection: undefined,
        currentPage: undefined,
        filterText: undefined
      };

      const productsTab = {
        id: 'tab-products',
        type: TabType.TABLE,
        title: 'products',
        tableName: 'products',
        connectionId: 'conn-1',
        schema: undefined,
        dbName: undefined,
        sortColumn: 'price',
        sortDirection: 'desc' as const,
        currentPage: 2,
        filterText: 'instock'
      };

      // Populate tabs store
      tabsStore.tabs = [usersTab, productsTab];

      // 1. Load users table (with active tab set to usersTab)
      tabsStore.activeTabId = usersTab.id;
      await tableData.loadTable('users', isLoading, 'conn-1');

      // Update state for usersTab
      tableData.sortColumn.value = 'first_name';
      tableData.sortDirection.value = 'desc';
      tableData.currentPage.value = 3;
      tableData.filterText.value = 'active';

      // Load table again to trigger persistence write back to usersTab
      await tableData.loadTable('users', isLoading, 'conn-1');
      expect(usersTab.sortColumn).toBe('first_name');
      expect(usersTab.sortDirection).toBe('desc');
      expect(usersTab.currentPage).toBe(3);
      expect(usersTab.filterText).toBe('active');

      // 2. Switch active tab to productsTab and load products table
      tabsStore.activeTabId = productsTab.id;
      await tableData.loadTable('products', isLoading, 'conn-1');

      // State should be restored from productsTab
      expect(tableData.sortColumn.value).toBe('price');
      expect(tableData.sortDirection.value).toBe('desc');
      expect(tableData.currentPage.value).toBe(2);
      expect(tableData.filterText.value).toBe('instock');

      // 3. Switch back to usersTab
      tabsStore.activeTabId = usersTab.id;
      await tableData.loadTable('users', isLoading, 'conn-1');

      // State should be restored from usersTab
      expect(tableData.sortColumn.value).toBe('first_name');
      expect(tableData.sortDirection.value).toBe('desc');
      expect(tableData.currentPage.value).toBe(3);
      expect(tableData.filterText.value).toBe('active');
    });

    it('persists and restores filter state without sort column when switching tabs', async () => {
      const tableData = useTableData(connectionsStore);
      const tabsStore = useTabsStore();
      vi.mocked(BridgeService.request).mockResolvedValue({ rows: [], fields: [], totalCount: 0 });

      const usersTab = {
        id: 'tab-users',
        type: TabType.TABLE,
        title: 'users',
        tableName: 'users',
        connectionId: 'conn-1',
        schema: undefined,
        dbName: undefined,
        sortColumn: undefined,
        sortDirection: undefined,
        currentPage: undefined,
        filterText: undefined
      };

      const productsTab = {
        id: 'tab-products',
        type: TabType.TABLE,
        title: 'products',
        tableName: 'products',
        connectionId: 'conn-1',
        schema: undefined,
        dbName: undefined,
        sortColumn: undefined,
        sortDirection: undefined,
        currentPage: undefined,
        filterText: undefined
      };

      tabsStore.tabs = [usersTab, productsTab];

      // 1. Switch to usersTab and load
      tabsStore.activeTabId = usersTab.id;
      await tableData.loadTable('users', isLoading, 'conn-1');

      // User types a filter condition but DOES NOT load table/press enter
      tableData.filterText.value = 'age > 18';

      // 2. Switch to productsTab (this should save the state of usersTab)
      tabsStore.activeTabId = productsTab.id;
      await tableData.loadTable('products', isLoading, 'conn-1');

      // Check usersTab state got saved automatically on switch
      expect(usersTab.filterText).toBe('age > 18');

      // 3. Switch back to usersTab
      tabsStore.activeTabId = usersTab.id;
      await tableData.loadTable('users', isLoading, 'conn-1');

      // State should be restored, even though sortColumn is undefined
      expect(tableData.filterText.value).toBe('age > 18');
      expect(tableData.sortColumn.value).toBeUndefined();
    });
  });
});
