import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import MigrationBanner from '../MigrationBanner.vue';
import { isAvailable, os } from '@/services/nativeService';

vi.mock('@/services/nativeService', () => ({
  isAvailable: vi.fn(),
  os: {
    open: vi.fn()
  }
}));

const mountMigrationBanner = (props?: { targetVersion?: string; learnMoreUrl?: string }) => {
  return mount(MigrationBanner, {
    props,
    global: {
      stubs: {
        Button: {
          props: ['label', 'variant', 'size'],
          emits: ['click'],
          template:
            '<button v-bind="$attrs" @click="$emit(\'click\', $event)">{{ label }}<slot name="icon" /><slot /></button>'
        }
      }
    }
  });
};

describe('MigrationBanner.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders constant migration notice mentioning 0.7.9 and native 0.8.0 GPUI engine', () => {
    const wrapper = mountMigrationBanner();
    const banner = wrapper.find('[data-testid="migration-banner"]');

    expect(banner.exists()).toBe(true);
    expect(banner.isVisible()).toBe(true);
    expect(banner.text()).toContain('0.7.9');
    expect(banner.text()).toContain('0.8.0');
    expect(banner.text()).toContain('GPUI');
  });

  it('renders custom targetVersion and learnMoreUrl when provided via props', async () => {
    vi.mocked(isAvailable).mockReturnValue(true);
    vi.mocked(os.open).mockResolvedValueOnce(undefined);

    const wrapper = mountMigrationBanner({
      targetVersion: '0.8.0-rc1',
      learnMoreUrl: 'https://example.com/migration'
    });

    expect(wrapper.find('[data-testid="migration-banner"]').text()).toContain('0.8.0-rc1');

    await wrapper.find('[data-testid="btn-migration-learn-more"]').trigger('click');
    await flushPromises();

    expect(os.open).toHaveBeenCalledTimes(1);
    expect(os.open).toHaveBeenCalledWith('https://example.com/migration');
  });

  it('opens learnMoreUrl via nativeService.os.open when Neutralino runtime is available', async () => {
    vi.mocked(isAvailable).mockReturnValue(true);
    vi.mocked(os.open).mockResolvedValueOnce(undefined);

    const wrapper = mountMigrationBanner();
    await wrapper.find('[data-testid="btn-migration-learn-more"]').trigger('click');
    await flushPromises();

    expect(os.open).toHaveBeenCalledTimes(1);
    expect(os.open).toHaveBeenCalledWith('https://github.com/vantoan1511/table-view/issues/90');
  });

  it('falls back to window.open when Neutralino runtime is unavailable', async () => {
    vi.mocked(isAvailable).mockReturnValue(false);
    const windowOpenSpy = vi.spyOn(window, 'open').mockImplementation(() => null);

    const wrapper = mountMigrationBanner();
    await wrapper.find('[data-testid="btn-migration-learn-more"]').trigger('click');
    await flushPromises();

    expect(os.open).not.toHaveBeenCalled();
    expect(windowOpenSpy).toHaveBeenCalledTimes(1);
    expect(windowOpenSpy).toHaveBeenCalledWith(
      'https://github.com/vantoan1511/table-view/issues/90',
      '_blank',
      'noopener,noreferrer'
    );

    windowOpenSpy.mockRestore();
  });
});
