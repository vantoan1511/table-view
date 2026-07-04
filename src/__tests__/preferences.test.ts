import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import { usePreferencesStore } from '../stores/preferences';

describe('Preferences Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('initializes with isOpen = false', () => {
    const store = usePreferencesStore();
    expect(store.isOpen).toBe(false);
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
});
