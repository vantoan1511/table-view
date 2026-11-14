import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import TitleBar from '../TitleBar.vue';
import { usePreferencesStore } from '@/stores/preferences';
import { useToastStore } from '@/stores/toast';
import { launchNativeUi } from '@/services/nativeService';

vi.mock('@/services/nativeService', () => ({
  launchNativeUi: vi.fn()
}));

const mountTitleBar = () => {
  return mount(TitleBar, {
    global: {
      directives: {
        tooltip: () => {}
      },
      stubs: {
        ContextMenu: true,
        InputText: true,
        Button: {
          props: ['disabled', 'loading', 'variant', 'severity', 'size'],
          template:
            '<button :disabled="disabled || loading" @click="$emit(\'click\', $event)"><slot name="icon" /><slot /></button>'
        }
      }
    }
  });
};

describe('TitleBar.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('does not render native UI button when experimentalFeatures is false', () => {
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.experimentalFeatures = false;

    const wrapper = mountTitleBar();
    expect(wrapper.find('[data-testid="btn-switch-native-ui"]').exists()).toBe(false);
  });

  it('renders native UI button when experimentalFeatures is true', () => {
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.experimentalFeatures = true;

    const wrapper = mountTitleBar();
    expect(wrapper.find('[data-testid="btn-switch-native-ui"]').exists()).toBe(true);
  });

  it('calls launchNativeUi and shows info toast on success', async () => {
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.experimentalFeatures = true;
    const toastStore = useToastStore();
    const addToastSpy = vi.spyOn(toastStore, 'addToast');

    vi.mocked(launchNativeUi).mockResolvedValueOnce({ launched: true, processId: 101 });

    const wrapper = mountTitleBar();
    await wrapper.find('[data-testid="btn-switch-native-ui"]').trigger('click');
    await flushPromises();

    expect(launchNativeUi).toHaveBeenCalledTimes(1);
    expect(addToastSpy).toHaveBeenCalledWith(
      expect.objectContaining({
        severity: 'info'
      })
    );
  });

  it('shows warning or error toast when launchNativeUi fails', async () => {
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.experimentalFeatures = true;
    const toastStore = useToastStore();
    const addToastSpy = vi.spyOn(toastStore, 'addToast');

    vi.mocked(launchNativeUi).mockResolvedValueOnce({
      launched: false,
      errorCode: 'BINARY_NOT_FOUND',
      message: 'Native UI binary not found'
    });

    const wrapper = mountTitleBar();
    await wrapper.find('[data-testid="btn-switch-native-ui"]').trigger('click');
    await flushPromises();

    expect(launchNativeUi).toHaveBeenCalledTimes(1);
    expect(addToastSpy).toHaveBeenCalledWith(
      expect.objectContaining({
        severity: 'warning',
        message: 'Native UI binary not found'
      })
    );
  });
});
