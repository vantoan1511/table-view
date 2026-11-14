import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import App from '@/App.vue';

vi.mock('vue', async (importOriginal) => {
  const actual = await importOriginal<typeof import('vue')>();
  return {
    ...actual,
    defineAsyncComponent: () => ({ template: '<div />' })
  };
});

vi.mock('@/services/nativeService', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/services/nativeService')>();
  return {
    ...actual,
    isAvailable: vi.fn(() => false),
    app: { ...actual.app, exit: vi.fn() },
    dbBridge: { ...actual.dbBridge, dispatch: vi.fn() },
    events: { ...actual.events, on: vi.fn() },
    os: { ...actual.os, open: vi.fn() }
  };
});

vi.mock('@/composables/useKeyboardShortcuts', () => ({
  useKeyboardShortcuts: vi.fn()
}));

vi.mock('@/composables/useTabSync', () => ({
  useTabSync: vi.fn()
}));

describe('App.vue MigrationBanner integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('renders MigrationBanner below TitleBar in the main workspace column', async () => {
    const wrapper = mount(App, {
      global: {
        stubs: {
          Sidebar: true,
          ResizeHandle: true,
          TitleBar: { template: '<div data-testid="title-bar-stub" />' },
          MigrationBanner: { template: '<div data-testid="migration-banner" />' },
          WorkspaceContainer: {
            template: '<div data-testid="workspace-container-stub"><slot name="main" /></div>'
          },
          TabContent: true,
          MinimizedDock: true,
          StatusBar: true,
          NewConnectionModal: true,
          ExportConnectionsModal: true,
          ImportConnectionsModal: true,
          GlobalErrorDialog: true,
          ToastContainer: true,
          UpdaterDialog: true,
          AboutDialog: true,
          PreferencesModal: true,
          TabSelectorDialog: true
        }
      }
    });

    const titleBar = wrapper.find('[data-testid="title-bar-stub"]');
    const banner = wrapper.find('[data-testid="migration-banner"]');
    const workspace = wrapper.find('[data-testid="workspace-container-stub"]');

    expect(titleBar.exists()).toBe(true);
    expect(banner.exists()).toBe(true);
    expect(workspace.exists()).toBe(true);

    // Verify DOM order inside the main content column: TitleBar -> MigrationBanner -> WorkspaceContainer
    const columnChildren = Array.from(titleBar.element.parentElement?.children ?? []);
    const titleIndex = columnChildren.indexOf(titleBar.element);
    const bannerIndex = columnChildren.indexOf(banner.element);
    const workspaceIndex = columnChildren.indexOf(workspace.element);

    expect(titleIndex).toBeGreaterThanOrEqual(0);
    expect(bannerIndex).toBe(titleIndex + 1);
    expect(workspaceIndex).toBe(bannerIndex + 1);

    await flushPromises();
    wrapper.unmount();
  });
});
