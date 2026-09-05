import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as Neutralino from '@neutralinojs/lib';
import {
  app,
  dbBridge,
  events,
  extensions,
  filesystem,
  init,
  isAvailable,
  NativeService,
  os,
  storage,
  updater,
  window as nativeWindow
} from '../nativeService';

vi.mock('@neutralinojs/lib', () => ({
  init: vi.fn(),
  app: {
    getConfig: vi.fn().mockResolvedValue({ applicationName: 'Table View', version: '0.6.8' }),
    restartProcess: vi.fn().mockResolvedValue(undefined),
    exit: vi.fn().mockResolvedValue(undefined)
  },
  window: {
    setTitle: vi.fn().mockResolvedValue(undefined),
    setSize: vi.fn().mockResolvedValue(undefined),
    show: vi.fn().mockResolvedValue(undefined),
    hide: vi.fn().mockResolvedValue(undefined),
    focus: vi.fn().mockResolvedValue(undefined)
  },
  os: {
    showOpenDialog: vi.fn(),
    showSaveDialog: vi.fn(),
    showMessageBox: vi.fn().mockResolvedValue('OK'),
    open: vi.fn().mockResolvedValue(undefined),
    execCommand: vi.fn().mockResolvedValue({ exitCode: 0, stdOut: 'ok', stdErr: '' }),
    spawnProcess: vi.fn().mockResolvedValue({ id: 101 }),
    getEnv: vi.fn().mockResolvedValue('/home/user')
  },
  filesystem: {
    readFile: vi.fn().mockResolvedValue('file content'),
    writeFile: vi.fn().mockResolvedValue(undefined),
    remove: vi.fn().mockResolvedValue(undefined),
    createDirectory: vi.fn().mockResolvedValue(undefined),
    readDirectory: vi.fn().mockResolvedValue([{ entry: 'file.txt', type: 'FILE' }])
  },
  storage: {
    setData: vi.fn().mockResolvedValue(undefined),
    getData: vi.fn().mockResolvedValue('{"key":"value"}'),
    getKeys: vi.fn().mockResolvedValue(['key1', 'key2'])
  },
  events: {
    on: vi.fn().mockResolvedValue(undefined),
    off: vi.fn().mockResolvedValue(undefined),
    dispatch: vi.fn().mockResolvedValue(undefined)
  },
  extensions: {
    dispatch: vi.fn().mockResolvedValue(undefined),
    getStats: vi.fn().mockResolvedValue({ loaded: ['ext1'] })
  },
  updater: {
    checkForUpdates: vi.fn().mockResolvedValue({ version: '1.0.0', resourcesURL: '' })
  }
}));

describe('nativeService', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (globalThis as any).NL_PORT = '9999';
  });

  describe('isAvailable & init', () => {
    it('detects runtime availability', () => {
      expect(isAvailable()).toBe(true);

      delete (globalThis as any).NL_PORT;
      expect(isAvailable()).toBe(false);

      (globalThis as any).NL_PORT = '9999';
      expect(isAvailable()).toBe(true);
    });

    it('initializes Neutralino when available', () => {
      init();
      expect(Neutralino.init).toHaveBeenCalledTimes(1);
    });
  });

  describe('app', () => {
    it('delegates getConfig, restartProcess, exit', async () => {
      const config = await app.getConfig();
      expect(config.applicationName).toBe('Table View');
      expect(Neutralino.app.getConfig).toHaveBeenCalled();

      await app.restartProcess();
      expect(Neutralino.app.restartProcess).toHaveBeenCalled();

      await app.exit(0);
      expect(Neutralino.app.exit).toHaveBeenCalledWith(0);
    });
  });

  describe('window', () => {
    it('delegates window operations', async () => {
      await nativeWindow.setTitle('My Title');
      expect(Neutralino.window.setTitle).toHaveBeenCalledWith('My Title');

      await nativeWindow.setSize({ width: 800, height: 600 });
      expect(Neutralino.window.setSize).toHaveBeenCalledWith({ width: 800, height: 600 });

      await nativeWindow.show();
      expect(Neutralino.window.show).toHaveBeenCalled();

      await nativeWindow.hide();
      expect(Neutralino.window.hide).toHaveBeenCalled();

      await nativeWindow.focus();
      expect(Neutralino.window.focus).toHaveBeenCalled();
    });
  });

  describe('os', () => {
    it('handles showOpenDialog with array return and cancellation', async () => {
      vi.mocked(Neutralino.os.showOpenDialog).mockResolvedValueOnce(['/path/file.json']);
      const res = await os.showOpenDialog('Open File');
      expect(res).toEqual(['/path/file.json']);

      // Single string normalized to array
      vi.mocked(Neutralino.os.showOpenDialog).mockResolvedValueOnce('/path/file2.json' as any);
      const res2 = await os.showOpenDialog('Open File');
      expect(res2).toEqual(['/path/file2.json']);

      // Cancellation error handling
      vi.mocked(Neutralino.os.showOpenDialog).mockRejectedValueOnce({ code: 'NE_OS_DLGCDL' });
      const resCancelled = await os.showOpenDialog('Open File');
      expect(resCancelled).toEqual([]);

      // Other errors are rethrown
      vi.mocked(Neutralino.os.showOpenDialog).mockRejectedValueOnce(new Error('Permission denied'));
      await expect(os.showOpenDialog('Open File')).rejects.toThrow('Permission denied');
    });

    it('handles showSaveDialog with string return and cancellation', async () => {
      vi.mocked(Neutralino.os.showSaveDialog).mockResolvedValueOnce('/path/save.json');
      const res = await os.showSaveDialog('Save File');
      expect(res).toBe('/path/save.json');

      // Cancellation error handling
      vi.mocked(Neutralino.os.showSaveDialog).mockRejectedValueOnce({ code: 'NE_OS_DLGCDL' });
      const resCancelled = await os.showSaveDialog('Save File');
      expect(resCancelled).toBeNull();
    });

    it('delegates showMessageBox, open, execCommand, spawnProcess, getEnv', async () => {
      await os.showMessageBox('Alert', 'Hello', 'OK', 'INFO');
      expect(Neutralino.os.showMessageBox).toHaveBeenCalledWith('Alert', 'Hello', 'OK', 'INFO');

      await os.open('https://example.com');
      expect(Neutralino.os.open).toHaveBeenCalledWith('https://example.com');

      const cmdRes = await os.execCommand('dir');
      expect(cmdRes.stdOut).toBe('ok');

      const procRes = await os.spawnProcess('echo hi');
      expect(procRes.id).toBe(101);

      const envVal = await os.getEnv('HOME');
      expect(envVal).toBe('/home/user');
    });
  });

  describe('filesystem', () => {
    it('delegates readFile, writeFile, remove, createDirectory, readDirectory', async () => {
      const content = await filesystem.readFile('/file.txt');
      expect(content).toBe('file content');

      await filesystem.writeFile('/file.txt', 'hello');
      expect(Neutralino.filesystem.writeFile).toHaveBeenCalledWith('/file.txt', 'hello');

      await filesystem.remove('/file.txt');
      expect(Neutralino.filesystem.remove).toHaveBeenCalledWith('/file.txt');

      await filesystem.createDirectory('/dir');
      expect(Neutralino.filesystem.createDirectory).toHaveBeenCalledWith('/dir');

      const dir = await filesystem.readDirectory('/');
      expect(dir.length).toBe(1);
    });
  });

  describe('storage', () => {
    it('delegates raw storage methods', async () => {
      await storage.setData('testKey', 'testVal');
      expect(Neutralino.storage.setData).toHaveBeenCalledWith('testKey', 'testVal');

      const raw = await storage.getData('testKey');
      expect(raw).toBe('{"key":"value"}');

      const keys = await storage.getKeys();
      expect(keys).toEqual(['key1', 'key2']);
    });

    it('handles typed get and set helpers with automatic JSON serialization', async () => {
      await storage.set('prefs', { theme: 'dark' });
      expect(Neutralino.storage.setData).toHaveBeenCalledWith('prefs', '{"theme":"dark"}');

      vi.mocked(Neutralino.storage.getData).mockResolvedValueOnce('{"theme":"dark"}');
      const parsed = await storage.get<{ theme: string }>('prefs');
      expect(parsed).toEqual({ theme: 'dark' });

      // Handles non-JSON gracefully
      vi.mocked(Neutralino.storage.getData).mockResolvedValueOnce('plain string');
      const plain = await storage.get<string>('strKey');
      expect(plain).toBe('plain string');

      // Handles null / empty data
      vi.mocked(Neutralino.storage.getData).mockRejectedValueOnce(new Error('not found'));
      const notFound = await storage.get('missing');
      expect(notFound).toBeNull();
    });
  });

  describe('events', () => {
    it('wraps handler, unwraps payload, and tracks for unregistering via off', async () => {
      const received: any[] = [];
      const handler = (data: any) => received.push(data);

      await events.on('windowClose', handler);
      expect(Neutralino.events.on).toHaveBeenCalledWith('windowClose', expect.any(Function));

      // Extract the internal wrapper passed to Neutralino.events.on
      const wrapper = vi.mocked(Neutralino.events.on).mock.calls[0][1];

      // Dispatch simulated event with detail
      wrapper({ detail: { code: 0 } });
      expect(received).toEqual([{ code: 0 }]);

      // Dispatch simulated event without detail
      wrapper('raw payload');
      expect(received).toEqual([{ code: 0 }, 'raw payload']);

      // Unregister with the original handler
      await events.off('windowClose', handler);
      expect(Neutralino.events.off).toHaveBeenCalledWith('windowClose', wrapper);
    });

    it('delegates dispatch', async () => {
      await events.dispatch('customEvent', { foo: 'bar' });
      expect(Neutralino.events.dispatch).toHaveBeenCalledWith('customEvent', { foo: 'bar' });
    });
  });

  describe('extensions & dbBridge', () => {
    it('delegates extensions dispatch and getStats', async () => {
      await extensions.dispatch('my.ext', 'doSomething', { a: 1 });
      expect(Neutralino.extensions.dispatch).toHaveBeenCalledWith('my.ext', 'doSomething', { a: 1 });

      const stats = await extensions.getStats();
      expect(stats.loaded).toEqual(['ext1']);
    });

    it('dbBridge helper dispatches with correct extension ID', async () => {
      await dbBridge.dispatch('dbBridge.shutdown', {});
      expect(Neutralino.extensions.dispatch).toHaveBeenCalledWith(
        'com.github.vantoan1511.tableview.db-bridge',
        'dbBridge.shutdown',
        {}
      );
    });
  });

  describe('updater', () => {
    it('delegates checkForUpdates', async () => {
      const res = await updater.checkForUpdates('https://manifest.json');
      expect(res.version).toBe('1.0.0');
      expect(Neutralino.updater.checkForUpdates).toHaveBeenCalledWith('https://manifest.json');
    });
  });

  describe('NativeService combined alias', () => {
    it('exposes all namespaces and fs alias', () => {
      expect(NativeService.app).toBe(app);
      expect(NativeService.window).toBe(nativeWindow);
      expect(NativeService.os).toBe(os);
      expect(NativeService.filesystem).toBe(filesystem);
      expect(NativeService.fs).toBe(filesystem);
      expect(NativeService.storage).toBe(storage);
      expect(NativeService.events).toBe(events);
      expect(NativeService.extensions).toBe(extensions);
      expect(NativeService.updater).toBe(updater);
      expect(NativeService.dbBridge).toBe(dbBridge);
    });
  });
});
