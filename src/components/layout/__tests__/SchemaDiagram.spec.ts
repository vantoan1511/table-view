import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import SchemaDiagram from '../SchemaDiagram.vue';
import { useDiagramStore } from '@/stores/diagram';
import { BridgeService } from '@/services/bridge';
import type { Tab } from '@/types';

// Mock BridgeService
vi.mock('@/services/bridge', () => ({
  BridgeService: {
    request: vi.fn().mockResolvedValue(null)
  }
}));

// Mock ResizeObserver
global.ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as any;

describe('SchemaDiagram Component', () => {
  let pinia: any;

  const mockTab: Tab = {
    id: 'tab-1',
    title: 'public (Diagram)',
    type: 'diagram',
    connectionId: 'conn-1',
    schema: 'public',
    dbName: 'testdb'
  };

  beforeEach(() => {
    pinia = createPinia();
    setActivePinia(pinia);
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  it('renders loading state initially and mounts cleanly', () => {
    const store = useDiagramStore();
    const key = store.getCacheKey('conn-1', 'public', 'testdb');
    store.loading[key] = true;

    const wrapper = mount(SchemaDiagram, {
      props: { tab: mockTab },
      global: {
        plugins: [pinia],
        directives: {
          tooltip: () => {}
        },
        stubs: {
          Button: true,
          IconField: true,
          InputIcon: true,
          InputText: true
        }
      }
    });

    expect(wrapper.text()).toContain('Retrieving Schema Metadata');
    wrapper.unmount();
  });

  it('debounces search input updates by 200ms', async () => {
    const store = useDiagramStore();
    const key = store.getCacheKey('conn-1', 'public', 'testdb');
    store.diagrams[key] = {
      tables: [
        {
          name: 'users',
          columns: [{ name: 'id', dataType: 'int', isPrimaryKey: true, nullable: false }]
        },
        {
          name: 'orders',
          columns: [{ name: 'id', dataType: 'int', isPrimaryKey: true, nullable: false }]
        }
      ],
      relations: []
    };

    const wrapper = mount(SchemaDiagram, {
      props: { tab: mockTab },
      global: {
        plugins: [pinia],
        directives: {
          tooltip: () => {}
        },
        stubs: {
          Button: true,
          IconField: true,
          InputIcon: true,
          InputText: true
        }
      }
    });

    // Initially without search, both tables would be filtered
    // We can simulate updating the search input
    const vm = wrapper.vm as any;
    expect(vm.filteredTables.length).toBe(2);

    vm.searchQuery = 'user';
    await wrapper.vm.$nextTick();

    // Before debounce fires (e.g. 100ms)
    vi.advanceTimersByTime(100);
    expect(vm.debouncedSearchQuery).toBe('');
    expect(vm.filteredTables.length).toBe(2);

    // After debounce expires (>200ms total)
    vi.advanceTimersByTime(150);
    expect(vm.debouncedSearchQuery).toBe('user');
    expect(vm.filteredTables.length).toBe(1);
    expect(vm.filteredTables[0].name).toBe('users');

    wrapper.unmount();
  });

  it('culls off-screen tables via visibleTables computed', async () => {
    const store = useDiagramStore();
    const key = store.getCacheKey('conn-1', 'public', 'testdb');
    store.diagrams[key] = {
      tables: [
        {
          name: 'table_in_view',
          columns: [{ name: 'id', dataType: 'int', isPrimaryKey: true, nullable: false }]
        },
        {
          name: 'table_off_screen',
          columns: [{ name: 'id', dataType: 'int', isPrimaryKey: true, nullable: false }]
        }
      ],
      relations: []
    };

    const wrapper = mount(SchemaDiagram, {
      props: { tab: mockTab },
      global: {
        plugins: [pinia],
        directives: {
          tooltip: () => {}
        },
        stubs: {
          Button: true,
          IconField: true,
          InputIcon: true,
          InputText: true
        }
      }
    });

    const vm = wrapper.vm as any;
    // Set explicit positions
    vm.panX = 0;
    vm.panY = 0;
    vm.zoom = 1.0;
    vm.viewportWidth = 800;
    vm.viewportHeight = 600;

    vm.tablePositions = {
      table_in_view: { x: 100, y: 100 },
      table_off_screen: { x: 5000, y: 5000 }
    };

    expect(vm.visibleTables.map((t: any) => t.name)).toContain('table_in_view');
    expect(vm.visibleTables.map((t: any) => t.name)).not.toContain('table_off_screen');

    wrapper.unmount();
  });

  it('culls off-screen relation edges in buildConnectionPaths', async () => {
    const store = useDiagramStore();
    const key = store.getCacheKey('conn-1', 'public', 'testdb');
    store.diagrams[key] = {
      tables: [
        {
          name: 't1',
          columns: [{ name: 'id', dataType: 'int', isPrimaryKey: true, nullable: false }]
        },
        {
          name: 't2',
          columns: [{ name: 'id', dataType: 'int', isPrimaryKey: true, nullable: false }, { name: 't1_id', dataType: 'int', isPrimaryKey: false, nullable: false }]
        },
        {
          name: 't_far1',
          columns: [{ name: 'id', dataType: 'int', isPrimaryKey: true, nullable: false }]
        },
        {
          name: 't_far2',
          columns: [{ name: 'id', dataType: 'int', isPrimaryKey: true, nullable: false }, { name: 'far1_id', dataType: 'int', isPrimaryKey: false, nullable: false }]
        }
      ],
      relations: [
        {
          constraintName: 'rel_near',
          sourceTable: 't2',
          sourceColumn: 't1_id',
          targetTable: 't1',
          targetColumn: 'id'
        },
        {
          constraintName: 'rel_far',
          sourceTable: 't_far2',
          sourceColumn: 'far1_id',
          targetTable: 't_far1',
          targetColumn: 'id'
        }
      ]
    };

    const wrapper = mount(SchemaDiagram, {
      props: { tab: mockTab },
      global: {
        plugins: [pinia],
        directives: {
          tooltip: () => {}
        },
        stubs: {
          Button: true,
          IconField: true,
          InputIcon: true,
          InputText: true
        }
      }
    });

    const vm = wrapper.vm as any;
    vm.panX = 0;
    vm.panY = 0;
    vm.zoom = 1.0;
    vm.viewportWidth = 800;
    vm.viewportHeight = 600;

    vm.tablePositions = {
      t1: { x: 50, y: 50 },
      t2: { x: 350, y: 50 },
      t_far1: { x: 6000, y: 6000 },
      t_far2: { x: 6500, y: 6000 }
    };

    const paths = vm.buildConnectionPaths();
    expect(paths.length).toBe(1);
    expect(paths[0].id).toContain('rel_near');

    wrapper.unmount();
  });
});
