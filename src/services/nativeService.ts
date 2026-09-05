import type { TableViewEventMap, TableViewEventName } from '@/types/events';
import {
  app as neuApp,
  events as neuEvents,
  extensions as neuExtensions,
  filesystem as neuFilesystem,
  init as neuInit,
  os as neuOs,
  storage as neuStorage,
  updater as neuUpdater,
  window as neuWindow
} from '@neutralinojs/lib';

export const DB_BRIDGE_EXTENSION_ID = 'com.github.vantoan1511.tableview.db-bridge';

/**
 * Checks if the Neutralino native runtime environment is available.
 */
export function isAvailable(): boolean {
  const g = typeof globalThis !== 'undefined' ? (globalThis as any) : undefined;
  return Boolean(g?.NL_PORT || g?.window?.NL_PORT);
}

/**
 * Initialize Neutralinojs native API.
 */
export function init(): void {
  if (isAvailable()) {
    neuInit();
  }
}

/**
 * Safe wrapper for Neutralino app API
 */
export const app = {
  getConfig(): Promise<any> {
    return neuApp.getConfig();
  },
  restartProcess(options?: any): Promise<void> {
    return neuApp.restartProcess(options);
  },
  exit(code?: number): Promise<void> {
    return neuApp.exit(code);
  }
};

/**
 * Safe wrapper for Neutralino window API
 */
export const window = {
  setTitle(title: string): Promise<void> {
    return neuWindow.setTitle(title);
  },
  setSize(options?: any): Promise<void> {
    return neuWindow.setSize(options);
  },
  show(): Promise<void> {
    return neuWindow.show();
  },
  hide(): Promise<void> {
    return neuWindow.hide();
  },
  focus(): Promise<void> {
    return neuWindow.focus();
  }
};

/**
 * Safe wrapper for Neutralino os API
 */
export const os = {
  async showOpenDialog(
    title: string,
    options?: {
      filters?: Array<{ name: string; extensions: string[] }>;
      multiSelections?: boolean;
      defaultPath?: string;
    }
  ): Promise<string[]> {
    if (!isAvailable()) return [];
    try {
      const res = await neuOs.showOpenDialog(title, options);
      if (Array.isArray(res)) return res;
      if (typeof res === 'string' && res) return [res];
      return [];
    } catch (err: any) {
      // Neutralino throws NE_OS_DLGCDL when user cancels dialog
      if (err?.code === 'NE_OS_DLGCDL') return [];
      throw err;
    }
  },

  async showSaveDialog(
    title: string,
    options?: {
      filters?: Array<{ name: string; extensions: string[] }>;
      defaultPath?: string;
      forceOverwrite?: boolean;
    }
  ): Promise<string | null> {
    if (!isAvailable()) return null;
    try {
      const res = await neuOs.showSaveDialog(title, options);
      return res || null;
    } catch (err: any) {
      // Neutralino throws NE_OS_DLGCDL when user cancels dialog
      if (err?.code === 'NE_OS_DLGCDL') return null;
      throw err;
    }
  },

  async showMessageBox(
    title: string,
    content: string,
    choice: string = 'OK',
    icon: 'INFO' | 'WARNING' | 'ERROR' | 'QUESTION' = 'INFO'
  ): Promise<string> {
    if (!isAvailable()) return '';
    return (await neuOs.showMessageBox(
      title,
      content,
      choice as any,
      icon as any
    )) as unknown as string;
  },

  open(url: string): Promise<void> {
    return neuOs.open(url);
  },

  execCommand(command: string, options?: any): Promise<any> {
    return neuOs.execCommand(command, options);
  },

  spawnProcess(command: string): Promise<any> {
    return neuOs.spawnProcess(command);
  },

  getEnv(key: string): Promise<string> {
    return neuOs.getEnv(key);
  }
};

/**
 * Safe wrapper for Neutralino filesystem API
 */
export const filesystem = {
  async readFile(path: string, options?: any): Promise<string> {
    if (!isAvailable()) return '';
    return await neuFilesystem.readFile(path, options);
  },

  async writeFile(path: string, content: string): Promise<void> {
    if (!isAvailable()) return;
    await neuFilesystem.writeFile(path, content);
  },

  async remove(path: string): Promise<void> {
    if (!isAvailable()) return;
    if (typeof neuFilesystem.remove === 'function') {
      await neuFilesystem.remove(path);
    }
  },

  async createDirectory(path: string): Promise<void> {
    if (!isAvailable()) return;
    if (typeof neuFilesystem.createDirectory === 'function') {
      await neuFilesystem.createDirectory(path);
    }
  },

  async readDirectory(path: string): Promise<any[]> {
    if (!isAvailable()) return [];
    return await neuFilesystem.readDirectory(path);
  }
};

/**
 * Safe wrapper for Neutralino storage API
 */
export const storage = {
  async setData(key: string, data?: string | null): Promise<void> {
    if (!isAvailable()) return;
    await neuStorage.setData(key, data ?? null);
  },

  async getData(key: string): Promise<string> {
    if (!isAvailable()) return '';
    return await neuStorage.getData(key);
  },

  async getKeys(): Promise<string[]> {
    if (!isAvailable()) return [];
    return await neuStorage.getKeys();
  },

  // Typed convenience helpers from Table View architecture
  async set<T = any>(key: string, data: T): Promise<void> {
    if (!isAvailable()) return;
    const stringified = typeof data === 'string' ? data : JSON.stringify(data);
    await neuStorage.setData(key, stringified);
  },

  async get<T = any>(key: string): Promise<T | null> {
    if (!isAvailable()) return null;
    try {
      const data = await neuStorage.getData(key);
      if (!data) return null;
      try {
        return JSON.parse(data) as T;
      } catch {
        return data as unknown as T;
      }
    } catch {
      return null;
    }
  }
};

const eventHandlerMap = new Map<string, Map<any, any>>();

export interface NativeEventsService {
  on<K extends TableViewEventName>(
    event: K,
    handler: (data: TableViewEventMap[K]) => void
  ): Promise<any>;
  on(event: string, handler: (data: any) => void): Promise<any>;

  off<K extends TableViewEventName>(
    event: K,
    handler: (data: TableViewEventMap[K]) => void
  ): Promise<any>;
  off(event: string, handler: (data: any) => void): Promise<any>;

  dispatch(event: string, data?: unknown): Promise<any>;
}

/**
 * Safe wrapper for Neutralino events API
 */
export const events: NativeEventsService = {
  on(event: string, handler: (data: any) => void): Promise<any> {
    const wrapper = (evt: any) => {
      const payload = evt?.detail !== undefined ? evt.detail : evt;
      handler(payload);
    };

    let handlers = eventHandlerMap.get(event);
    if (!handlers) {
      handlers = new Map();
      eventHandlerMap.set(event, handlers);
    }
    handlers.set(handler, wrapper);

    return neuEvents.on(event, wrapper);
  },

  off(event: string, handler: (data: any) => void): Promise<any> {
    const handlers = eventHandlerMap.get(event);
    if (handlers) {
      const wrapper = handlers.get(handler);
      if (wrapper) {
        handlers.delete(handler);
        if (handlers.size === 0) {
          eventHandlerMap.delete(event);
        }
        return neuEvents.off(event, wrapper);
      }
    }
    return neuEvents.off(event, handler as any);
  },

  dispatch(event: string, data?: unknown): Promise<any> {
    return neuEvents.dispatch(event, data);
  }
};

/**
 * Safe wrapper for Neutralino extensions API
 */
export const extensions = {
  dispatch(extensionId: string, event: string, data?: unknown): Promise<any> {
    return neuExtensions.dispatch(extensionId, event, data);
  },
  getStats(): Promise<any> {
    return neuExtensions.getStats();
  }
};

/**
 * Safe wrapper for Neutralino updater API
 */
export const updater = {
  checkForUpdates(manifestUrl: string): Promise<any> {
    return neuUpdater.checkForUpdates(manifestUrl);
  }
};

/**
 * Extension dispatcher helper for db-bridge
 */
export const dbBridge = {
  dispatch(event: string, data?: unknown): Promise<any> {
    return extensions.dispatch(DB_BRIDGE_EXTENSION_ID, event, data);
  }
};

/**
 * Unified NativeService object for backward compatibility with existing codebase
 */
export const NativeService = {
  init,
  isAvailable,
  app,
  window,
  os,
  fs: filesystem,
  filesystem,
  storage,
  events,
  extensions,
  updater,
  dbBridge
};

export default NativeService;
