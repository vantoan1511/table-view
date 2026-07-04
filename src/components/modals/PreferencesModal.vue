<script setup lang="ts">
import Button from '@/components/ui/Button.vue';
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue';
import { usePreferencesStore } from '@/stores/preferences';
import { useToastStore } from '@/stores/toast';
import {
  Code,
  Database,
  Info,
  Keyboard,
  Link2,
  Palette,
  Settings,
  Shield,
  Table,
  Terminal,
  X
} from 'lucide-vue-next';
import { computed, reactive, ref } from 'vue';

const preferencesStore = usePreferencesStore();
const toastStore = useToastStore();

const activeTab = ref('general');

const tabs = [
  { id: 'general', name: 'General', icon: Settings },
  { id: 'grid', name: 'Data Grid', icon: Table },
  { id: 'editor', name: 'Editor', icon: Code },
  { id: 'query', name: 'Query', icon: Terminal },
  { id: 'appearance', name: 'Appearance', icon: Palette },
  { id: 'shortcuts', name: 'Shortcuts', icon: Keyboard },
  { id: 'connections', name: 'Connections', icon: Link2 },
  { id: 'security', name: 'Security', icon: Shield },
  { id: 'about', name: 'About', icon: Info }
];

const settings = reactive({
  theme: 'dark',
  language: 'en',
  autoUpdate: true,
  startMinimized: false,
  maxRows: 1000,
  autoSaveHistory: true,
  playCompletionSound: false,
  telemetry: false
});

const getCurrentTabName = computed(() => {
  const current = tabs.find((t) => t.id === activeTab.value);
  return current ? current.name : '';
});

const getCurrentTabIcon = computed(() => {
  const current = tabs.find((t) => t.id === activeTab.value);
  return current ? current.icon : Settings;
});

const resetToDefaults = () => {
  settings.theme = 'dark';
  settings.language = 'en';
  settings.autoUpdate = true;
  settings.startMinimized = false;
  settings.maxRows = 1000;
  settings.autoSaveHistory = true;
  settings.playCompletionSound = false;
  settings.telemetry = false;

  toastStore.addToast({
    title: 'Preferences reset',
    message: 'Settings have been reset to default values.',
    severity: 'info',
    variation: 'outlined',
    position: 'bottom-center',
    ttl: 3000
  });
};

const savePreferences = () => {
  preferencesStore.close();
  toastStore.addToast({
    title: 'Preferences saved',
    message: 'Changes saved successfully (prototype only).',
    severity: 'success',
    variation: 'filled',
    position: 'bottom-center',
    ttl: 3000
  });
};
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="preferencesStore.isOpen"
        class="modal-backdrop fixed inset-0 z-100 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
        @click.self="preferencesStore.close"
      >
        <div
          class="bg-surface border-border animate-in zoom-in modal-container flex h-[580px] max-h-[85vh] w-full max-w-4xl flex-col overflow-hidden rounded-2xl border shadow-2xl duration-200"
        >
          <!-- Header -->
          <div class="border-border flex items-center justify-between border-b px-5 py-4">
            <div class="text-text-primary flex items-center gap-2.5">
              <Settings :size="18" class="text-primary shrink-0" />
              <h2 class="text-sm font-semibold">Preferences</h2>
            </div>
            <button
              @click="preferencesStore.close"
              class="text-text-tertiary hover:text-text-primary hover:bg-hover rounded-lg p-1.5 transition-colors"
            >
              <X :size="16" />
            </button>
          </div>

          <!-- Body -->
          <div class="flex min-h-0 flex-1">
            <!-- Sidebar -->
            <div
              class="border-border bg-surface flex w-56 shrink-0 flex-col gap-1 overflow-y-auto border-r p-3"
            >
              <button
                v-for="tab in tabs"
                :key="tab.id"
                @click="activeTab = tab.id"
                class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-left text-xs font-medium transition-colors select-none"
                :class="
                  activeTab === tab.id
                    ? 'bg-primary/10 text-primary'
                    : 'text-text-secondary hover:bg-hover hover:text-text-primary'
                "
              >
                <component :is="tab.icon" :size="14" class="shrink-0" />
                <span>{{ tab.name }}</span>
              </button>
            </div>

            <!-- Content Area -->
            <div class="flex-1 overflow-y-auto p-6">
              <!-- General Settings Tab -->
              <div v-if="activeTab === 'general'" class="max-w-2xl space-y-6">
                <div>
                  <h3 class="text-text-primary text-sm font-medium">General Settings</h3>
                  <p class="text-text-tertiary mt-0.5 text-[11px]">
                    Configure global application behavior and preferences.
                  </p>
                </div>

                <div class="bg-border/50 h-px" />

                <!-- Application Section -->
                <div class="space-y-4">
                  <h4 class="text-text-tertiary text-[10px] font-bold tracking-wider uppercase">
                    Application
                  </h4>

                  <div class="flex items-center justify-between py-1">
                    <div class="flex flex-col gap-0.5">
                      <span class="text-text-primary text-xs font-medium">Interface Theme</span>
                      <span class="text-text-tertiary text-[11px]">
                        Select how Table View looks on your device.
                      </span>
                    </div>
                    <select
                      v-model="settings.theme"
                      class="bg-surface border-border text-text-primary focus:border-primary focus:ring-primary w-40 cursor-pointer rounded-lg border px-3 py-1.5 text-xs focus:ring-1 focus:outline-none"
                    >
                      <option value="light">Light</option>
                      <option value="dark">Dark</option>
                      <option value="system">System Preference</option>
                    </select>
                  </div>

                  <div class="flex items-center justify-between py-1">
                    <div class="flex flex-col gap-0.5">
                      <span class="text-text-primary text-xs font-medium">Language</span>
                      <span class="text-text-tertiary text-[11px]">
                        Choose the language for the user interface.
                      </span>
                    </div>
                    <select
                      v-model="settings.language"
                      class="bg-surface border-border text-text-primary focus:border-primary focus:ring-primary w-40 cursor-pointer rounded-lg border px-3 py-1.5 text-xs focus:ring-1 focus:outline-none"
                    >
                      <option value="en">English (US)</option>
                      <option value="vi">Tiếng Việt</option>
                    </select>
                  </div>

                  <div class="flex items-center justify-between py-1">
                    <div class="flex flex-col gap-0.5">
                      <span class="text-text-primary text-xs font-medium">Start Minimized</span>
                      <span class="text-text-tertiary text-[11px]">
                        Launch the app minimized to the system tray.
                      </span>
                    </div>
                    <ToggleSwitch v-model="settings.startMinimized" />
                  </div>

                  <div class="flex items-center justify-between py-1">
                    <div class="flex flex-col gap-0.5">
                      <span class="text-text-primary text-xs font-medium">Automatic Updates</span>
                      <span class="text-text-tertiary text-[11px]">
                        Keep Table View up to date with automatic releases.
                      </span>
                    </div>
                    <ToggleSwitch v-model="settings.autoUpdate" />
                  </div>
                </div>

                <div class="bg-border/50 h-px" />

                <!-- Data & Files Section -->
                <div class="space-y-4">
                  <h4 class="text-text-tertiary text-[10px] font-bold tracking-wider uppercase">
                    Data & Files
                  </h4>

                  <div class="flex items-center justify-between py-1">
                    <div class="flex flex-col gap-0.5">
                      <span class="text-text-primary text-xs font-medium">Default Row Limit</span>
                      <span class="text-text-tertiary text-[11px]">
                        Maximum number of rows to retrieve in table grid query.
                      </span>
                    </div>
                    <select
                      v-model="settings.maxRows"
                      class="bg-surface border-border text-text-primary focus:border-primary focus:ring-primary w-40 cursor-pointer rounded-lg border px-3 py-1.5 text-xs focus:ring-1 focus:outline-none"
                    >
                      <option :value="100">100 rows</option>
                      <option :value="500">500 rows</option>
                      <option :value="1000">1,000 rows</option>
                      <option :value="5000">5,000 rows</option>
                      <option :value="10000">10,000 rows</option>
                    </select>
                  </div>

                  <div class="flex items-center justify-between py-1">
                    <div class="flex flex-col gap-0.5">
                      <span class="text-text-primary text-xs font-medium">
                        Auto-save Query History
                      </span>
                      <span class="text-text-tertiary text-[11px]">
                        Locally persist executed queries in session history.
                      </span>
                    </div>
                    <ToggleSwitch v-model="settings.autoSaveHistory" />
                  </div>
                </div>

                <div class="bg-border/50 h-px" />

                <!-- Other Settings Section -->
                <div class="space-y-4">
                  <h4 class="text-text-tertiary text-[10px] font-bold tracking-wider uppercase">
                    Other Settings
                  </h4>

                  <div class="flex items-center justify-between py-1">
                    <div class="flex flex-col gap-0.5">
                      <span class="text-text-primary text-xs font-medium"
                        >Play Sound on Finish</span
                      >
                      <span class="text-text-tertiary text-[11px]">
                        Play a notification sound when a long query completes.
                      </span>
                    </div>
                    <ToggleSwitch v-model="settings.playCompletionSound" />
                  </div>

                  <div class="flex items-center justify-between py-1">
                    <div class="flex flex-col gap-0.5">
                      <span class="text-text-primary text-xs font-medium">
                        Help Improve Table View
                      </span>
                      <span class="text-text-tertiary text-[11px]">
                        Send anonymous performance and usage telemetry data.
                      </span>
                    </div>
                    <ToggleSwitch v-model="settings.telemetry" />
                  </div>
                </div>
              </div>

              <!-- Placeholder Tabs -->
              <div
                v-else
                class="flex h-full flex-col items-center justify-center space-y-4 text-center"
              >
                <div
                  class="bg-muted border-border/50 flex h-14 w-14 items-center justify-center rounded-2xl border"
                >
                  <component :is="getCurrentTabIcon" :size="24" class="text-text-tertiary" />
                </div>
                <div>
                  <h3 class="text-text-primary text-xs font-semibold">{{ getCurrentTabName }}</h3>
                  <p class="text-text-secondary mt-1 text-[11px]">
                    This panel is under construction in this prototype.
                  </p>
                </div>
              </div>
            </div>
          </div>

          <!-- Footer -->
          <div class="border-border flex shrink-0 items-center justify-between border-t px-5 py-3">
            <Button variant="secondary" size="sm" @click="resetToDefaults">
              Reset to Defaults
            </Button>
            <div class="flex items-center gap-2">
              <Button variant="ghost" size="sm" @click="preferencesStore.close"> Cancel </Button>
              <Button variant="primary" size="sm" @click="savePreferences"> Save Changes </Button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
