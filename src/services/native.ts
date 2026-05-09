import * as Neutralino from '@neutralinojs/lib';

/**
 * NativeService wraps built-in Neutralino APIs to provide a consistent,
 * mockable, and type-safe interface for storage, filesystem, and OS operations.
 */
export class NativeService {
  /**
   * Storage APIs
   */
  static storage = {
    async set<T = any>(key: string, data: T): Promise<void> {
      if (!window.NL_PORT) return;
      const stringified = typeof data === 'string' ? data : JSON.stringify(data);
      await Neutralino.storage.setData(key, stringified);
    },

    async get<T = any>(key: string): Promise<T | null> {
      if (!window.NL_PORT) return null;
      try {
        const data = await Neutralino.storage.getData(key);
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

  /**
   * OS / UI Dialog APIs
   */
  static os = {
    async showSaveDialog(
      title: string,
      options: { filters: Array<{ name: string; extensions: string[] }> }
    ): Promise<string | null> {
      if (!window.NL_PORT) return null;
      return await Neutralino.os.showSaveDialog(title, options);
    },

    async showOpenDialog(title: string, options: any): Promise<string[] | null> {
      if (!window.NL_PORT) return null;
      return await Neutralino.os.showOpenDialog(title, options);
    },

    async showMessageBox(
      title: string,
      content: string,
      icon: 'INFO' | 'WARNING' | 'ERROR' | 'QUESTION' = 'INFO'
    ): Promise<void> {
      if (!window.NL_PORT) return;
      await Neutralino.os.showMessageBox(
        title,
        content,
        Neutralino.MessageBoxChoice.OK,
        Neutralino.Icon[icon]
      );
    }
  };

  /**
   * Filesystem APIs
   */
  static fs = {
    async writeFile(path: string, content: string): Promise<void> {
      if (!window.NL_PORT) return;
      await Neutralino.filesystem.writeFile(path, content);
    },

    async readFile(path: string): Promise<string | null> {
      if (!window.NL_PORT) return null;
      return await Neutralino.filesystem.readFile(path);
    }
  };
}
