import { setActivePinia, createPinia } from 'pinia';
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { useUpdaterStore } from '../updater';
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
});
