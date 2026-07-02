import { useGridStore } from '../grid';
import { BridgeService } from '@/services/bridge';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/services/bridge', () => ({
  BridgeService: {
    request: vi.fn()
  }
}));

describe('Grid Store - Constraints', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    (window as any).NL_PORT = '8080';
  });

  it('getTableConstraints invokes the bridge service and returns constraints', async () => {
    const store = useGridStore();
    const mockResponse = {
      constraints: [
        {
          name: 'pk_users',
          constraintType: 'PRIMARY KEY',
          definition: 'PRIMARY KEY (id)'
        }
      ]
    };
    vi.mocked(BridgeService.request).mockResolvedValue(mockResponse);

    const constraints = await store.getTableConstraints('users');

    expect(BridgeService.request).toHaveBeenCalledWith(
      'dbBridge.getTableConstraints',
      'dbBridge.getTableConstraintsResult',
      expect.objectContaining({
        tableName: 'users'
      })
    );
    expect(constraints).toEqual(mockResponse.constraints);
  });
});
