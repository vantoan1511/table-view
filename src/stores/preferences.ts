import { defineStore } from 'pinia';
import { reactive, ref } from 'vue';

import { NativeService } from '@/services/native';
import { useUpdaterStore } from './updater';

export interface Preferences {
  theme: 'light' | 'dark' | 'system';
  language: 'en' | 'vi';
  autoUpdate: boolean;
  optInPreview?: boolean;
  startMinimized: boolean;
  maxRows: number;
  connectionTimeout?: number;
  autoSaveHistory: boolean;
  playCompletionSound: boolean;
  telemetry: boolean;
}

export const usePreferencesStore = defineStore('preferences', () => {
  const isOpen = ref(false);
  const activeTab = ref('general');

  const settings = reactive<Preferences>({
    theme: 'dark',
    language: 'en',
    autoUpdate: true,
    optInPreview: false,
    startMinimized: false,
    maxRows: 1000,
    connectionTimeout: 15,
    autoSaveHistory: true,
    playCompletionSound: false,
    telemetry: false
  });

  let mediaQueryListener: ((e: MediaQueryListEvent) => void) | null = null;

  const applyTheme = (theme: 'light' | 'dark' | 'system') => {
    if (mediaQueryListener) {
      window
        .matchMedia('(prefers-color-scheme: dark)')
        .removeEventListener('change', mediaQueryListener);
      mediaQueryListener = null;
    }

    let isDark = false;
    if (theme === 'dark') {
      isDark = true;
    } else if (theme === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      isDark = mediaQuery.matches;
      mediaQueryListener = (e: MediaQueryListEvent) => {
        if (settings.theme === 'system') {
          document.documentElement.classList.toggle('dark', e.matches);
        }
      };
      mediaQuery.addEventListener('change', mediaQueryListener);
    }
    document.documentElement.classList.toggle('dark', isDark);
  };

  const init = async () => {
    try {
      const data = await NativeService.storage.get<Preferences>('preferences');
      if (data) {
        Object.assign(settings, data);
      }
      applyTheme(settings.theme);
    } catch (error) {
      console.error('Failed to initialize preferences:', error);
    }
  };

  const save = async (newSettings?: Partial<Preferences>) => {
    const prevOptIn = settings.optInPreview;
    if (newSettings) {
      Object.assign(settings, newSettings);
    }
    applyTheme(settings.theme);
    await NativeService.storage.set('preferences', settings);

    if (prevOptIn !== settings.optInPreview) {
      const updaterStore = useUpdaterStore();
      updaterStore.checkForUpdates(false);
    }
  };

  const toggle = (val?: boolean) => {
    isOpen.value = val !== undefined ? val : !isOpen.value;
  };

  const open = (tab: string = 'general') => {
    activeTab.value = tab;
    isOpen.value = true;
  };

  const close = () => toggle(false);

  return {
    isOpen,
    activeTab,
    settings,
    init,
    save,
    toggle,
    open,
    close
  };
});
