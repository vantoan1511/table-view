<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater';

import { Download, FileText, Info, Loader2, RefreshCw, X } from 'lucide-vue-next';
import { computed, onMounted } from 'vue';

const updaterStore = useUpdaterStore();

const parsedReleaseNotes = computed(() => {
  const notes = updaterStore.updateAvailable?.data?.releaseNotes;
  if (!notes) return [];

  const lines = notes.split('\n');
  const blocks: Array<{ type: 'heading' | 'list-item' | 'paragraph'; text: string }> = [];

  lines.forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed) return;

    if (trimmed.startsWith('#')) {
      const text = trimmed.replace(/^#+\s*/, '');
      blocks.push({ type: 'heading', text });
    } else if (trimmed.startsWith('-') || trimmed.startsWith('*') || trimmed.startsWith('•')) {
      const text = trimmed.replace(/^[-*•]\s*/, '');
      blocks.push({ type: 'list-item', text });
    } else {
      blocks.push({ type: 'paragraph', text: trimmed });
    }
  });

  return blocks;
});

onMounted(() => {
  updaterStore.init();
  // Auto-check after 5 seconds
  setTimeout(() => {
    updaterStore.checkForUpdates();
  }, 5000);
});

const formatInlineMarkdown = (text: string): string => {
  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');

  return escaped
    .replace(/\*\*(.*?)\*\*/g, '<strong class="font-bold text-text-primary">$1</strong>')
    .replace(/\*(.*?)\*/g, '<em class="italic">$1</em>')
    .replace(/_(.*?)_/g, '<em class="italic">$1</em>')
    .replace(
      /`(.*?)`/g,
      '<code class="bg-surface border border-border rounded px-1.5 py-0.5 text-[11px] font-mono text-primary">$1</code>'
    );
};

const close = () => {
  if (!updaterStore.isUpdating) {
    updaterStore.showUpdateDialog = false;
    updaterStore.error = null;
  }
};
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="updaterStore.showUpdateDialog || updaterStore.error"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
      >
        <div
          class="bg-surface border-border animate-in zoom-in w-full max-w-4xl overflow-hidden rounded-xl border shadow-2xl duration-200"
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
            <Button
              v-if="!updaterStore.isUpdating"
              rounded
              variant="text"
              severity="secondary"
              @click="close"
            >
              <template #icon>
                <X class="h-4 w-4" />
              </template>
            </Button>
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
              <div class="flex w-full max-w-xs justify-center gap-3">
                <Button
                  variant="text"
                  severity="secondary"
                  size="small"
                  class="flex-1"
                  @click="close"
                >
                  Close
                </Button>
                <Button size="small" class="flex-1" @click="updaterStore.checkForUpdates(true)">
                  Try Again
                </Button>
              </div>
            </div>

            <div v-else-if="updaterStore.isUpdating" class="space-y-6">
              <div class="flex flex-col items-center gap-4 text-center">
                <Loader2 :size="40" class="text-primary animate-spin" />
                <div class="space-y-1">
                  <p class="text-text-primary font-medium">
                    {{ updaterStore.updateStatus }}
                    <template v-if="updaterStore.downloadProgress > 0">
                      ({{ updaterStore.downloadProgress }}%)
                    </template>
                  </p>
                  <p class="text-text-tertiary text-sm italic">
                    Please do not close the application.
                  </p>
                </div>
              </div>
              <!-- Progress Bar -->
              <div class="bg-border h-2 w-full overflow-hidden rounded-full">
                <div
                  class="bg-primary h-full transition-all duration-300 ease-out"
                  :style="{ width: `${updaterStore.downloadProgress}%` }"
                ></div>
              </div>
            </div>

            <div v-else-if="updaterStore.updateAvailable" class="space-y-4">
              <div class="flex flex-col gap-1">
                <p class="text-text-secondary text-sm font-medium">
                  Version {{ updaterStore.updateAvailable.version }} is ready
                </p>
              </div>

              <!-- Premium Scrollable Changelog Section -->
              <div class="flex flex-col gap-2">
                <div
                  class="text-text-secondary flex items-center gap-1.5 text-[13px] font-semibold"
                >
                  <FileText :size="14" class="text-primary" />
                  <span>What's New in this Version</span>
                </div>

                <div
                  class="bg-surface-lighter border-border scrollbar-thin max-h-45 overflow-y-auto rounded-lg border p-3"
                >
                  <div v-if="parsedReleaseNotes.length" class="space-y-2.5">
                    <template v-for="(block, idx) in parsedReleaseNotes" :key="idx">
                      <!-- Heading -->
                      <h4
                        v-if="block.type === 'heading'"
                        class="text-text-primary mt-3 text-sm font-bold tracking-tight first:mt-0"
                        v-html="formatInlineMarkdown(block.text)"
                      ></h4>
                      <!-- List Item -->
                      <div
                        v-else-if="block.type === 'list-item'"
                        class="text-text-secondary flex items-start gap-2 pl-1 text-[13px] leading-relaxed"
                      >
                        <span class="bg-primary mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full"></span>
                        <span v-html="formatInlineMarkdown(block.text)"></span>
                      </div>
                      <!-- Paragraph -->
                      <p
                        v-else
                        class="text-text-secondary text-[13px] leading-relaxed"
                        v-html="formatInlineMarkdown(block.text)"
                      ></p>
                    </template>
                  </div>

                  <!-- Fallback Text -->
                  <div v-else class="text-text-tertiary py-1 text-[13px] leading-relaxed italic">
                    A new version of Table View is available with bug fixes and performance
                    improvements.
                  </div>
                </div>
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
                <Button fluid size="large" @click="updaterStore.installUpdates">
                  <Download class="h-4 w-4" />
                  Install and Restart
                </Button>
                <div class="flex gap-3">
                  <Button
                    variant="text"
                    severity="secondary"
                    size="small"
                    class="flex-1"
                    @click="close"
                  >
                    Later
                  </Button>
                  <Button
                    variant="text"
                    severity="secondary"
                    size="small"
                    class="text-text-tertiary! flex-1"
                    @click="updaterStore.ignoreUpdate(updaterStore.updateAvailable.version)"
                  >
                    Skip version
                  </Button>
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
