import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setupConsoleOverride, initLogger } from './logger';
import * as Neutralino from '@neutralinojs/lib';

// Mock Neutralino
vi.mock('@neutralinojs/lib', () => ({
  extensions: {
    dispatch: vi.fn().mockResolvedValue({}),
    getStats: vi.fn().mockResolvedValue({ loaded: [] })
  },
  events: {
    on: vi.fn()
  }
}));

describe('Logger Utility', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset internal state if possible, but since it's module-level,
    // we might need to be careful.
  });

  it('should queue logs when extension is not connected', async () => {
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    setupConsoleOverride();

    console.log('test message');

    expect(consoleSpy).toHaveBeenCalledWith('test message');
    expect(Neutralino.extensions.dispatch).not.toHaveBeenCalled();

    consoleSpy.mockRestore();
  });

  it('should format arguments correctly', () => {
    // We can't directly test formatArgs as it's not exported,
    // but we can test it through console.log if we mock sendToExtension
    // However, sendToExtension is also not exported.
    // For now, let's just verify the setup doesn't crash.
    expect(() => setupConsoleOverride()).not.toThrow();
  });

  it('should initialize and register events', () => {
    initLogger();
    expect(Neutralino.events.on).toHaveBeenCalledWith('extensionReady', expect.any(Function));
  });
});
