import { setActivePinia, createPinia } from 'pinia';
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { useUpdaterStore } from '../updater';
import { useToastStore } from '../toast';
import * as Neutralino from '@neutralinojs/lib';

// Mock Neutralino
vi.mock('@neutralinojs/lib', () => ({
  storage: {
    getData: vi.fn(),
    setData: vi.fn()
  },
  filesystem: {
    remove: vi.fn(),
    writeFile: vi.fn()
  },
  updater: {
    checkForUpdates: vi.fn()
  },
  os: {
    execCommand: vi.fn()
  },
  app: {
    exit: vi.fn()
  }
}));

describe('Updater Store', () => {
  let originalFetch: typeof fetch;

  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();

    // Setup window environments
    // @ts-ignore
    window.NL_PORT = 1234;
    // @ts-ignore
    window.NL_APPVERSION = '0.2.10';
    // @ts-ignore
    window.NL_OS = 'Windows';

    originalFetch = global.fetch;
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  it('checks for updates and fetches release notes from GitHub API on success', async () => {
    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.11',
      resourcesURL: 'https://example.com/resources.neu',
      data: {
        extensionUrl: 'https://example.com/db-bridge.exe',
        releaseNotes: 'Generic notes'
      }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);

    // Mock fetch for GitHub API
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        body: '# Features\n• Added cool feature\n\n# Bug Fixes\n• Fixed issue'
      })
    });
    global.fetch = mockFetch;

    const store = useUpdaterStore();
    await store.checkForUpdates();

    expect(Neutralino.updater.checkForUpdates).toHaveBeenCalled();
    expect(mockFetch).toHaveBeenCalledWith(
      'https://api.github.com/repos/vantoan1511/table-view/releases/tags/v0.2.11'
    );
    expect(store.updateAvailable).not.toBeNull();
    expect(store.updateAvailable?.data.releaseNotes).toBe(
      '# Features\n• Added cool feature\n\n# Bug Fixes\n• Fixed issue'
    );
  });

  it('falls back to manifest release notes if GitHub API returns non-ok response', async () => {
    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.11',
      resourcesURL: 'https://example.com/resources.neu',
      data: {
        extensionUrl: 'https://example.com/db-bridge.exe',
        releaseNotes: 'Generic notes'
      }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);

    // Mock fetch with error status
    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404
    });
    global.fetch = mockFetch;

    const store = useUpdaterStore();
    await store.checkForUpdates();

    expect(mockFetch).toHaveBeenCalled();
    expect(store.updateAvailable).not.toBeNull();
    expect(store.updateAvailable?.data.releaseNotes).toBe('Generic notes');
  });

  it('falls back to manifest release notes if GitHub API fetch throws an error', async () => {
    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.11',
      resourcesURL: 'https://example.com/resources.neu',
      data: {
        extensionUrl: 'https://example.com/db-bridge.exe',
        releaseNotes: 'Generic notes'
      }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);

    // Mock fetch to reject
    const mockFetch = vi.fn().mockRejectedValue(new Error('Network failure'));
    global.fetch = mockFetch;

    const store = useUpdaterStore();
    await store.checkForUpdates();

    expect(mockFetch).toHaveBeenCalled();
    expect(store.updateAvailable).not.toBeNull();
    expect(store.updateAvailable?.data.releaseNotes).toBe('Generic notes');
  });

  it('ignores older or same versions during update check', async () => {
    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.10',
      resourcesURL: 'https://example.com/resources.neu',
      data: {
        extensionUrl: 'https://example.com/db-bridge.exe'
      }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);

    const store = useUpdaterStore();
    await store.checkForUpdates();

    expect(store.updateAvailable).toBeNull();
  });

  it('installs updates and writes the swapper batch script with the dynamic/detected exe name', async () => {
    // Setup window environment
    // @ts-ignore
    window.NL_PID = 9999;

    // Mock Neutralino.os.execCommand to simulate PowerShell returning a path
    vi.mocked(Neutralino.os.execCommand).mockImplementation(async (cmd) => {
      if (cmd.includes('Get-Process')) {
        return {
          exitCode: 0,
          stdOut: 'C:\\Program Files\\Table View\\custom-table-view.exe\n',
          stdErr: ''
        };
      }
      return { exitCode: 0, stdOut: '', stdErr: '' };
    });

    const store = useUpdaterStore();

    // Setup updateAvailable manifest
    store.updateAvailable = {
      applicationId: 'table-view',
      version: '0.2.11',
      resourcesURL: 'https://example.com/resources.neu',
      data: {
        extensionUrl: 'https://example.com/db-bridge.exe'
      }
    };

    // Trigger update installation
    await store.installUpdates();

    // Verify download calls
    expect(Neutralino.os.execCommand).toHaveBeenCalledWith(
      expect.stringContaining("Split-Path -Path 'resources.neu.new'")
    );
    expect(Neutralino.os.execCommand).toHaveBeenCalledWith(
      expect.stringContaining("Invoke-WebRequest -Uri 'https://example.com/resources.neu'")
    );
    expect(Neutralino.os.execCommand).toHaveBeenCalledWith(
      expect.stringContaining("Split-Path -Path 'bin\\db-bridge.exe.new'")
    );
    expect(Neutralino.os.execCommand).toHaveBeenCalledWith(
      expect.stringContaining("Invoke-WebRequest -Uri 'https://example.com/db-bridge.exe'")
    );

    // Verify that the swapper batch script is written with the correct dynamic exe name
    expect(Neutralino.filesystem.writeFile).toHaveBeenCalledWith(
      'updater.bat',
      expect.stringContaining('custom-table-view.exe')
    );
  });

  it('triggers a toast notification in background checks (manual = false)', async () => {
    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.11',
      resourcesURL: 'https://example.com/resources.neu',
      data: {
        extensionUrl: 'https://example.com/db-bridge.exe'
      }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);
    global.fetch = vi.fn().mockResolvedValue({ ok: false });

    const toastStore = useToastStore();
    const store = useUpdaterStore();

    await store.checkForUpdates(false);

    expect(store.updateAvailable).not.toBeNull();
    expect(store.showUpdateDialog).toBe(false);
    expect(toastStore.toasts.length).toBe(1);
    expect(toastStore.toasts[0].title).toBe('Update Available');
    expect(toastStore.toasts[0].actions).toBeDefined();
    expect(toastStore.toasts[0].actions?.length).toBe(2);
  });

  it('directly opens dialog in manual checks (manual = true)', async () => {
    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.11',
      resourcesURL: 'https://example.com/resources.neu',
      data: {
        extensionUrl: 'https://example.com/db-bridge.exe'
      }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);
    global.fetch = vi.fn().mockResolvedValue({ ok: false });

    const toastStore = useToastStore();
    const store = useUpdaterStore();

    await store.checkForUpdates(true);

    expect(store.updateAvailable).not.toBeNull();
    expect(store.showUpdateDialog).toBe(true);
    expect(toastStore.toasts.length).toBe(0);
  });

  it('toast actions behave correctly (Dismiss closes, Show Details opens dialog)', async () => {
    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.11',
      resourcesURL: 'https://example.com/resources.neu',
      data: {
        extensionUrl: 'https://example.com/db-bridge.exe'
      }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);
    global.fetch = vi.fn().mockResolvedValue({ ok: false });

    const toastStore = useToastStore();
    const store = useUpdaterStore();

    // 1. Background check triggers toast
    await store.checkForUpdates(false);
    expect(toastStore.toasts.length).toBe(1);

    const toast = toastStore.toasts[0];
    const dismissAction = toast.actions?.find((a) => a.label === 'Dismiss');
    const showDetailsAction = toast.actions?.find((a) => a.label === 'Show Details');

    expect(dismissAction).toBeDefined();
    expect(showDetailsAction).toBeDefined();

    // 2. Click "Show Details" -> Opens dialog, removes toast
    showDetailsAction?.onClick();
    expect(store.showUpdateDialog).toBe(true);
    expect(toastStore.toasts.length).toBe(0);

    // Reset state
    store.showUpdateDialog = false;

    // 3. Trigger toast again and click "Dismiss"
    await store.checkForUpdates(false);
    expect(toastStore.toasts.length).toBe(1);

    toastStore.toasts[0].actions?.find((a) => a.label === 'Dismiss')?.onClick();
    expect(store.showUpdateDialog).toBe(false);
    expect(toastStore.toasts.length).toBe(0);
  });

  it('uses preview manifest URL when optInPreview preference is enabled', async () => {
    const { usePreferencesStore } = await import('../preferences');
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.optInPreview = true;

    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.11-preview.1',
      resourcesURL: 'https://example.com/resources.neu',
      data: { extensionUrl: 'https://example.com/db-bridge.exe' }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);
    global.fetch = vi.fn().mockResolvedValue({ ok: false });

    const store = useUpdaterStore();
    await store.checkForUpdates();

    expect(Neutralino.updater.checkForUpdates).toHaveBeenCalledWith(
      'https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest-preview.json'
    );
    expect(store.updateAvailable?.version).toBe('0.2.11-preview.1');
  });

  it('correctly compares prerelease version numbers', async () => {
    const { usePreferencesStore } = await import('../preferences');
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.optInPreview = true;

    // Current version is a preview
    // @ts-ignore
    window.NL_APPVERSION = '0.2.11-preview.1';

    const mockManifest = {
      applicationId: 'table-view',
      version: '0.2.11-preview.2',
      resourcesURL: 'https://example.com/resources.neu',
      data: { extensionUrl: 'https://example.com/db-bridge.exe' }
    };

    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(mockManifest);
    global.fetch = vi.fn().mockResolvedValue({ ok: false });

    const store = useUpdaterStore();
    await store.checkForUpdates();

    expect(store.updateAvailable).not.toBeNull();
    expect(store.updateAvailable?.version).toBe('0.2.11-preview.2');
  });

  it('correctly handles multi-digit and lexical pre-release identifiers', async () => {
    const { usePreferencesStore } = await import('../preferences');
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.optInPreview = true;

    // Test multi-digit preview upgrade: 0.2.11-preview.10 > 0.2.11-preview.2
    // @ts-ignore
    window.NL_APPVERSION = '0.2.11-preview.2';
    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue({
      applicationId: 'table-view',
      version: '0.2.11-preview.10',
      resourcesURL: 'https://example.com/resources.neu',
      data: { extensionUrl: 'https://example.com/db-bridge.exe' }
    });
    global.fetch = vi.fn().mockResolvedValue({ ok: false });

    const store = useUpdaterStore();
    await store.checkForUpdates();
    expect(store.updateAvailable?.version).toBe('0.2.11-preview.10');

    // Test lexical upgrade: 0.2.11-rc.1 > 0.2.11-preview.5
    // @ts-ignore
    window.NL_APPVERSION = '0.2.11-preview.5';
    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue({
      applicationId: 'table-view',
      version: '0.2.11-rc.1',
      resourcesURL: 'https://example.com/resources.neu',
      data: { extensionUrl: 'https://example.com/db-bridge.exe' }
    });
    await store.checkForUpdates();
    expect(store.updateAvailable?.version).toBe('0.2.11-rc.1');
  });

  it('recognizes preview build over stable when optInPreview is true, and ignores when false', async () => {
    const { usePreferencesStore } = await import('../preferences');
    const preferencesStore = usePreferencesStore();

    // @ts-ignore
    window.NL_APPVERSION = '0.2.11';
    const previewManifest = {
      applicationId: 'table-view',
      version: '0.2.11-preview.1',
      resourcesURL: 'https://example.com/resources.neu',
      data: { extensionUrl: 'https://example.com/db-bridge.exe' }
    };
    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue(previewManifest);
    global.fetch = vi.fn().mockResolvedValue({ ok: false });

    const store = useUpdaterStore();

    // 1. Opt-in preview is true -> update is recognized
    preferencesStore.settings.optInPreview = true;
    await store.checkForUpdates();
    expect(store.updateAvailable?.version).toBe('0.2.11-preview.1');

    // Reset update state
    store.updateAvailable = null;

    // 2. Opt-in preview is false -> preview update is ignored
    preferencesStore.settings.optInPreview = false;
    await store.checkForUpdates();
    expect(store.updateAvailable).toBeNull();
  });

  it('shows no updates toast on manual update check when app is up to date', async () => {
    // @ts-ignore
    window.NL_APPVERSION = '0.2.11';
    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue({
      applicationId: 'table-view',
      version: '0.2.11',
      resourcesURL: 'https://example.com/resources.neu',
      data: { extensionUrl: 'https://example.com/db-bridge.exe' }
    });

    const toastStore = useToastStore();
    const store = useUpdaterStore();

    await store.checkForUpdates(true);

    expect(store.updateAvailable).toBeNull();
    expect(toastStore.toasts.length).toBe(1);
    expect(toastStore.toasts[0].title).toBe('No Updates Available');
    expect(toastStore.toasts[0].message).toContain('0.2.11');
  });

  it('persists and uses installedVersion from storage if newer than NL_APPVERSION', async () => {
    // Simulated base binary version
    // @ts-ignore
    window.NL_APPVERSION = '0.2.11';

    // Mock storage returning installed preview version
    vi.mocked(Neutralino.storage.getData).mockResolvedValue(
      JSON.stringify({
        installedVersion: '0.2.11-preview.1'
      })
    );

    const store = useUpdaterStore();
    await store.init();

    expect(store.installedVersion).toBe('0.2.11-preview.1');
    expect(store.getCurrentAppVersion()).toBe('0.2.11-preview.1');

    // When checking updates for preview.2
    vi.mocked(Neutralino.updater.checkForUpdates).mockResolvedValue({
      applicationId: 'table-view',
      version: '0.2.11-preview.2',
      resourcesURL: 'https://example.com/resources.neu',
      data: { extensionUrl: 'https://example.com/db-bridge.exe' }
    });
    const { usePreferencesStore } = await import('../preferences');
    usePreferencesStore().settings.optInPreview = true;

    await store.checkForUpdates();
    expect(store.updateAvailable?.version).toBe('0.2.11-preview.2');
  });
});
