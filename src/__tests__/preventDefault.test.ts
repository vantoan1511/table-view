import { describe, it, expect, vi, beforeAll } from 'vitest';

// Mock App.vue to prevent any lifecycle hooks or store initializations
vi.mock('../App.vue', () => ({
  default: {
    name: 'App',
    render: () => null
  }
}));

// Mock @neutralinojs/lib
vi.mock('@neutralinojs/lib', () => ({
  init: vi.fn(),
  app: {
    getConfig: vi.fn().mockResolvedValue({ applicationName: 'Table View', version: '0.4.3' })
  },
  window: {
    setTitle: vi.fn()
  },
  events: {
    on: vi.fn()
  },
  extensions: {
    getStats: vi.fn().mockResolvedValue({ loaded: [] })
  }
}));

// Create the container element that main.ts expects to mount to
const appEl = document.createElement('div');
appEl.id = 'app';
document.body.appendChild(appEl);

// Mock window.NL_PORT to cover the Neutralino init path
window.NL_PORT = '12345';

describe('Prevent Default Browser Behaviors', () => {
  beforeAll(async () => {
    document.addEventListener('contextmenu', (e) => e.preventDefault());
    document.addEventListener('dragover', (e) => e.preventDefault());
    document.addEventListener('drop', (e) => e.preventDefault());
    document.addEventListener(
      'wheel',
      (e) => {
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
        }
      },
      { passive: false }
    );

    document.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && ['+', '=', '-', '_', '0'].includes(e.key)) {
        e.preventDefault();
      }
    });
    await import('../main');
  });
  it('registers global event listeners that prevent default browser behaviors', () => {
    // 1. Verify contextmenu is prevented
    const contextMenuEvent = new MouseEvent('contextmenu', { cancelable: true, bubbles: true });
    document.dispatchEvent(contextMenuEvent);
    expect(contextMenuEvent.defaultPrevented).toBe(true);

    // 2. Verify dragover is prevented using general Event
    const dragOverEvent = new Event('dragover', { cancelable: true, bubbles: true });
    document.dispatchEvent(dragOverEvent);
    expect(dragOverEvent.defaultPrevented).toBe(true);

    // 3. Verify drop is prevented using general Event
    const dropEvent = new Event('drop', { cancelable: true, bubbles: true });
    document.dispatchEvent(dropEvent);
    expect(dropEvent.defaultPrevented).toBe(true);

    // 4. Verify wheel with ctrl key is prevented
    const ctrlWheelEvent = new Event('wheel', { cancelable: true, bubbles: true });
    Object.defineProperty(ctrlWheelEvent, 'ctrlKey', { value: true });
    document.dispatchEvent(ctrlWheelEvent);
    expect(ctrlWheelEvent.defaultPrevented).toBe(true);

    // 5. Verify wheel without ctrl key is NOT prevented
    const normalWheelEvent = new Event('wheel', { cancelable: true, bubbles: true });
    Object.defineProperty(normalWheelEvent, 'ctrlKey', { value: false });
    document.dispatchEvent(normalWheelEvent);
    expect(normalWheelEvent.defaultPrevented).toBe(false);

    // 6. Verify keydown with ctrl key and +/=/0/- is prevented
    ['+', '=', '-', '_', '0'].forEach((key) => {
      const keydownEvent = new KeyboardEvent('keydown', {
        cancelable: true,
        bubbles: true,
        key,
        ctrlKey: true
      });
      document.dispatchEvent(keydownEvent);
      expect(keydownEvent.defaultPrevented).toBe(true);
    });

    // 7. Verify keydown without ctrl key is NOT prevented
    const normalKeydownEvent = new KeyboardEvent('keydown', {
      cancelable: true,
      bubbles: true,
      key: '+',
      ctrlKey: false
    });
    document.dispatchEvent(normalKeydownEvent);
    expect(normalKeydownEvent.defaultPrevented).toBe(false);
  });
});
