import { usePreferencesStore } from '../stores/preferences';

import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { NativeService } from '@/services/native';

vi.mock('@/services/native', () => ({
  NativeService: {
    storage: {
      get: vi.fn(),
      set: vi.fn()
    }
  }
}));

describe('Preferences Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    document.documentElement.className = '';
    // Mock window.matchMedia for JSDOM
    window.matchMedia = vi.fn().mockImplementation((query) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn()
    }));
  });

  it('initializes with default settings', () => {
    const store = usePreferencesStore();
    expect(store.isOpen).toBe(false);
    expect(store.settings.theme).toBe('dark');
    expect(store.settings.language).toBe('en');
    expect(store.settings.autoUpdate).toBe(true);
    expect(store.settings.startMinimized).toBe(false);
    expect(store.settings.maxRows).toBe(1000);
    expect(store.settings.autoSaveHistory).toBe(true);
    expect(store.settings.playCompletionSound).toBe(false);
    expect(store.settings.telemetry).toBe(false);
  });

  it('toggles visibility state', () => {
    const store = usePreferencesStore();
    store.toggle();
    expect(store.isOpen).toBe(true);
    store.toggle(false);
    expect(store.isOpen).toBe(false);
    store.toggle(true);
    expect(store.isOpen).toBe(true);
    store.toggle();
    expect(store.isOpen).toBe(false);
  });

  it('opens and closes modal explicitly', () => {
    const store = usePreferencesStore();
    store.open();
    expect(store.isOpen).toBe(true);
    store.close();
    expect(store.isOpen).toBe(false);
  });

  it('loads preferences on init', async () => {
    const mockPreferences = {
      theme: 'light',
      language: 'vi',
      autoUpdate: false,
      startMinimized: true,
      maxRows: 500,
      autoSaveHistory: false,
      playCompletionSound: true,
      telemetry: true
    };
    vi.mocked(NativeService.storage.get).mockResolvedValueOnce(mockPreferences);

    const store = usePreferencesStore();
    await store.init();

    expect(NativeService.storage.get).toHaveBeenCalledWith('preferences');
    expect(store.settings.theme).toBe('light');
    expect(store.settings.language).toBe('vi');
    expect(store.settings.maxRows).toBe(500);
    expect(document.documentElement.classList.contains('dark')).toBe(false);
  });

  it('applies dark theme class when theme is dark', async () => {
    vi.mocked(NativeService.storage.get).mockResolvedValueOnce({
      theme: 'dark'
    });

    const store = usePreferencesStore();
    await store.init();

    expect(document.documentElement.classList.contains('dark')).toBe(true);
  });

  it('saves preferences and updates settings', async () => {
    const store = usePreferencesStore();
    await store.save({
      theme: 'light',
      language: 'vi',
      maxRows: 5000
    });

    expect(NativeService.storage.set).toHaveBeenCalledWith(
      'preferences',
      expect.objectContaining({
        theme: 'light',
        language: 'vi',
        maxRows: 5000
      })
    );
    expect(store.settings.theme).toBe('light');
    expect(store.settings.language).toBe('vi');
    expect(store.settings.maxRows).toBe(5000);
    expect(document.documentElement.classList.contains('dark')).toBe(false);
  });

  it('handles system preference media query changes', async () => {
    let mediaChangeCallback: ((e: any) => void) | null = null;
    window.matchMedia = vi.fn().mockImplementation((query) => ({
      matches: true,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn().mockImplementation((event, callback) => {
        mediaChangeCallback = callback;
      }),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn()
    }));

    const store = usePreferencesStore();
    await store.save({ theme: 'system' });

    expect(document.documentElement.classList.contains('dark')).toBe(true);
    expect(window.matchMedia).toHaveBeenCalledWith('(prefers-color-scheme: dark)');

    if (mediaChangeCallback) {
      // @ts-ignore
      mediaChangeCallback({ matches: false });
    }
    expect(document.documentElement.classList.contains('dark')).toBe(false);
  });
});
