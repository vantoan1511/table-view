import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import PrimeVue from 'primevue/config';
import InputText from 'primevue/inputtext';
import GridToolbar from '../GridToolbar.vue';
import { useGridStore } from '@/stores/grid';
import { nextTick } from 'vue';

// Mock BridgeService
vi.mock('@/services/bridge', () => ({
  BridgeService: {
    request: vi.fn().mockResolvedValue(null)
  }
}));

// Mock nativeService
vi.mock('@/services/nativeService', () => ({
  os: {
    showSaveDialog: vi.fn(),
    writeFile: vi.fn()
  }
}));

describe('GridToolbar - Filter Autocomplete & Focus', () => {
  let pinia: any;

  beforeEach(() => {
    pinia = createPinia();
    setActivePinia(pinia);
    (window as any).NL_PORT = '8080';
  });

  const createWrapper = () => {
    return mount(GridToolbar, {
      global: {
        plugins: [pinia, PrimeVue],
        components: {
          InputText
        },
        directives: {
          tooltip: () => {}
        },
        stubs: {
          AlterTableDialog: true,
          Popover: true,
          Select: true
        }
      },
      attachTo: document.body
    });
  };

  it('correctly handles suggestion selection via click and focuses native input without TypeError', async () => {
    const gridStore = useGridStore();
    gridStore.activeTableName = 'users';
    gridStore.columns = [
      { name: 'id', dataType: 'int4' } as any,
      { name: 'username', dataType: 'varchar' } as any,
      { name: 'email', dataType: 'varchar' } as any
    ];

    const wrapper = createWrapper();
    await nextTick();

    const input = wrapper.find('input[type="text"]');
    expect(input.exists()).toBe(true);

    const inputEl = input.element as HTMLInputElement;
    const focusSpy = vi.spyOn(inputEl, 'focus');
    const setSelectionRangeSpy = vi.spyOn(inputEl, 'setSelectionRange');

    // Simulate typing a prefix to open suggestions
    await input.setValue('user');
    await input.trigger('input');
    await nextTick();

    // Check suggestions dropdown
    const suggestionButtons = wrapper.findAll('.z-50 button');
    expect(suggestionButtons.length).toBeGreaterThan(0);

    // Click on the suggestion
    await suggestionButtons[0].trigger('click');
    await nextTick();

    // Ensure focus and setSelectionRange were called on native input element
    expect(focusSpy).toHaveBeenCalled();
    expect(setSelectionRangeSpy).toHaveBeenCalled();

    // Ensure suggestion value replaced token
    expect(gridStore.filterText).toContain('username');

    wrapper.unmount();
  });

  it('handles suggestion selection via Enter key and restores focus', async () => {
    const gridStore = useGridStore();
    gridStore.activeTableName = 'users';
    gridStore.columns = [
      { name: 'id', dataType: 'int4' } as any,
      { name: 'created_at', dataType: 'timestamp' } as any
    ];

    const wrapper = createWrapper();
    await nextTick();

    const input = wrapper.find('input[type="text"]');
    const inputEl = input.element as HTMLInputElement;
    const focusSpy = vi.spyOn(inputEl, 'focus');

    // Simulate typing
    await input.setValue('create');
    await input.trigger('input');
    await nextTick();

    // Press Enter to select first suggestion
    await input.trigger('keydown', { key: 'Enter' });
    await nextTick();

    expect(focusSpy).toHaveBeenCalled();
    expect(gridStore.filterText).toContain('created_at');

    wrapper.unmount();
  });

  it('properly calculates cursorOffset from native input element', async () => {
    const gridStore = useGridStore();
    gridStore.activeTableName = 'users';
    gridStore.columns = [
      { name: 'first_name', dataType: 'varchar' } as any,
      { name: 'last_name', dataType: 'varchar' } as any
    ];

    const wrapper = createWrapper();
    await nextTick();

    const input = wrapper.find('input[type="text"]');
    const inputEl = input.element as HTMLInputElement;

    // Simulate input with cursor position
    inputEl.value = 'first';
    inputEl.setSelectionRange(5, 5);
    await input.trigger('input');
    await nextTick();

    const suggestionButtons = wrapper.findAll('.z-50 button');
    expect(suggestionButtons.length).toBeGreaterThan(0);
    expect(suggestionButtons[0].text()).toContain('first_name');

    wrapper.unmount();
  });

  it('clears filter text when clear button is clicked', async () => {
    const gridStore = useGridStore();
    gridStore.activeTableName = 'users';
    gridStore.filterText = 'id > 5';
    const loadTableSpy = vi.spyOn(gridStore, 'loadTable').mockImplementation(() => Promise.resolve());

    const wrapper = createWrapper();
    await nextTick();

    // Find clear button (with X icon)
    const clearBtn = wrapper.find('button.absolute');
    expect(clearBtn.exists()).toBe(true);

    await clearBtn.trigger('click');
    expect(gridStore.filterText).toBe('');
    expect(loadTableSpy).toHaveBeenCalledWith('users');

    wrapper.unmount();
  });
});
