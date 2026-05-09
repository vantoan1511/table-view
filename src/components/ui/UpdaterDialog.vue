<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater';
import { Download, Info, Loader2, RefreshCw, X } from 'lucide-vue-next';
import { onMounted } from 'vue';

const updaterStore = useUpdaterStore();

onMounted(() => {
  updaterStore.init();
  // Auto-check after 5 seconds
  setTimeout(() => {
    updaterStore.checkForUpdates();
  }, 5000);
});

const close = () => {
  if (!updaterStore.isUpdating) {
    updaterStore.updateAvailable = null;
    updaterStore.error = null;
  }
};
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="updaterStore.updateAvailable || updaterStore.error"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
      >
        <div
          class="bg-surface border-border animate-in zoom-in w-full max-w-md overflow-hidden rounded-xl border shadow-2xl duration-200"
        >
          <!-- Header -->
          <div
            class="border-border bg-surface-lighter flex items-center justify-between border-b px-5 py-4"
          >
            <div class="flex items-center gap-3">
              <div class="bg-primary/10 text-primary rounded-lg p-2">
                <RefreshCw v-if="updaterStore.isUpdating" :size="18" class="animate-spin" />
                <Download v-else :size="18" />
              </div>
              <h3 class="text-text-primary font-semibold">
                {{ updaterStore.isUpdating ? 'Updating Application' : 'Update Available' }}
              </h3>
            </div>
            <button
              v-if="!updaterStore.isUpdating"
              @click="close"
              class="text-text-tertiary hover:text-text-primary hover:bg-hover rounded-md p-1.5 transition-colors"
            >
              <X :size="18" />
            </button>
          </div>

          <!-- Body -->
          <div class="p-6">
            <div v-if="updaterStore.error" class="flex flex-col items-center gap-4 text-center">
              <div class="bg-danger/10 text-danger rounded-full p-3">
                <Info :size="32" />
              </div>
              <div class="space-y-1">
                <p class="text-text-primary font-medium">Update Check Failed</p>
                <p class="text-text-tertiary text-sm">{{ updaterStore.error }}</p>
              </div>
              <button
                @click="updaterStore.checkForUpdates(true)"
                class="bg-primary hover:bg-primary-hover mt-2 rounded-lg px-4 py-2 text-sm font-medium text-white shadow-sm transition-all"
              >
                Try Again
              </button>
            </div>

            <div v-else-if="updaterStore.isUpdating" class="space-y-6">
              <div class="flex flex-col items-center gap-4 text-center">
                <Loader2 :size="40" class="text-primary animate-spin" />
                <div class="space-y-1">
                  <p class="text-text-primary font-medium">{{ updaterStore.updateStatus }}</p>
                  <p class="text-text-tertiary text-sm italic">
                    Please do not close the application.
                  </p>
                </div>
              </div>
              <!-- Simulated Progress Bar (Native Neutralino installer doesn't provide progress easily) -->
              <div class="bg-border h-2 w-full overflow-hidden rounded-full">
                <div
                  class="bg-primary h-full animate-pulse transition-all duration-500"
                  style="width: 100%"
                ></div>
              </div>
            </div>

            <div v-else-if="updaterStore.updateAvailable" class="space-y-4">
              <div class="flex flex-col gap-1">
                <p class="text-text-secondary text-sm font-medium">
                  Version {{ updaterStore.updateAvailable.version }} is ready
                </p>
                <p class="text-text-tertiary text-[13px] leading-relaxed">
                  {{
                    updaterStore.updateAvailable.data.releaseNotes ||
                    'A new version of Table View is available with bug fixes and performance improvements.'
                  }}
                </p>
              </div>

              <div
                class="bg-surface-lighter border-border flex items-start gap-3 rounded-lg border p-3"
              >
                <Info :size="16" class="mt-0.5 shrink-0 text-amber-500" />
                <p class="text-text-secondary text-[12px]">
                  The application will automatically restart once the installation is complete.
                </p>
              </div>

              <div class="flex flex-col gap-3 pt-2">
                <button
                  @click="updaterStore.installUpdates"
                  class="bg-primary hover:bg-primary-hover flex w-full items-center justify-center gap-2 rounded-lg px-4 py-2.5 text-sm font-semibold text-white shadow-md transition-all active:scale-[0.98]"
                >
                  <Download :size="16" />
                  Install and Restart
                </button>
                <div class="flex gap-3">
                  <button
                    @click="close"
                    class="bg-surface-lighter hover:bg-hover border-border text-text-secondary flex-1 rounded-lg border px-4 py-2.5 text-[13px] font-medium transition-all"
                  >
                    Later
                  </button>
                  <button
                    @click="updaterStore.ignoreUpdate(updaterStore.updateAvailable.version)"
                    class="bg-surface-lighter hover:bg-hover border-border text-text-tertiary flex-1 rounded-lg border px-4 py-2.5 text-[13px] font-medium transition-all"
                  >
                    Skip version
                  </button>
                </div>
              </div>
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
