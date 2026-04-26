<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater'
import { Download, Info, Loader2, RefreshCw, X } from 'lucide-vue-next'
import { onMounted } from 'vue'

const updaterStore = useUpdaterStore()

onMounted(() => {
  updaterStore.init()
  // Auto-check after 5 seconds
  setTimeout(() => {
    updaterStore.checkForUpdates()
  }, 5000)
})

const close = () => {
  if (!updaterStore.isUpdating) {
    updaterStore.updateAvailable = null
    updaterStore.error = null
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="updaterStore.updateAvailable || updaterStore.error"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
        
        <div class="bg-surface border border-border rounded-xl shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in duration-200">
          <!-- Header -->
          <div class="flex items-center justify-between px-5 py-4 border-b border-border bg-surface-lighter">
            <div class="flex items-center gap-3">
              <div class="p-2 rounded-lg bg-primary/10 text-primary">
                <RefreshCw v-if="updaterStore.isUpdating" :size="18" class="animate-spin" />
                <Download v-else :size="18" />
              </div>
              <h3 class="font-semibold text-text-primary">
                {{ updaterStore.isUpdating ? 'Updating Application' : 'Update Available' }}
              </h3>
            </div>
            <button v-if="!updaterStore.isUpdating" @click="close"
              class="p-1.5 text-text-tertiary hover:text-text-primary hover:bg-hover rounded-md transition-colors">
              <X :size="18" />
            </button>
          </div>

          <!-- Body -->
          <div class="p-6">
            <div v-if="updaterStore.error" class="flex flex-col items-center gap-4 text-center">
              <div class="p-3 rounded-full bg-danger/10 text-danger">
                <Info :size="32" />
              </div>
              <div class="space-y-1">
                <p class="font-medium text-text-primary">Update Check Failed</p>
                <p class="text-sm text-text-tertiary">{{ updaterStore.error }}</p>
              </div>
              <button @click="updaterStore.checkForUpdates(true)"
                class="mt-2 px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm font-medium transition-all shadow-sm">
                Try Again
              </button>
            </div>

            <div v-else-if="updaterStore.isUpdating" class="space-y-6">
              <div class="flex flex-col items-center gap-4 text-center">
                <Loader2 :size="40" class="text-primary animate-spin" />
                <div class="space-y-1">
                  <p class="font-medium text-text-primary">{{ updaterStore.updateStatus }}</p>
                  <p class="text-sm text-text-tertiary italic">Please do not close the application.</p>
                </div>
              </div>
              <!-- Simulated Progress Bar (Native Neutralino installer doesn't provide progress easily) -->
              <div class="w-full bg-border rounded-full h-2 overflow-hidden">
                <div class="bg-primary h-full animate-pulse transition-all duration-500" style="width: 100%"></div>
              </div>
            </div>

            <div v-else-if="updaterStore.updateAvailable" class="space-y-4">
              <div class="flex flex-col gap-1">
                <p class="text-sm font-medium text-text-secondary">Version {{ updaterStore.updateAvailable.version }} is ready</p>
                <p class="text-[13px] text-text-tertiary leading-relaxed">
                  {{ updaterStore.updateAvailable.data.releaseNotes || 'A new version of Table View is available with bug fixes and performance improvements.' }}
                </p>
              </div>
              
              <div class="p-3 rounded-lg bg-surface-lighter border border-border flex items-start gap-3">
                <Info :size="16" class="text-amber-500 mt-0.5 shrink-0" />
                <p class="text-[12px] text-text-secondary">
                  The application will automatically restart once the installation is complete.
                </p>
              </div>

              <div class="flex items-center gap-3 pt-2">
                <button @click="updaterStore.installUpdates"
                  class="flex-1 px-4 py-2.5 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm font-semibold transition-all shadow-md active:scale-[0.98]">
                  Install and Restart
                </button>
                <button @click="close"
                  class="px-4 py-2.5 bg-surface-lighter hover:bg-hover border border-border text-text-secondary rounded-lg text-sm font-medium transition-all">
                  Later
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>
