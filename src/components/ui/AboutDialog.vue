<script setup lang="ts">
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import { useAboutStore } from '@/stores/about';
import { useUpdaterStore } from '@/stores/updater';
import * as Neutralino from '@neutralinojs/lib';
import { Coffee, Github, Globe, Heart } from 'lucide-vue-next';
import { onMounted, ref } from 'vue';

const aboutStore = useAboutStore();
const version = ref('0.1.0');

onMounted(async () => {
  if (window.NL_PORT) {
    try {
      const updaterStore = useUpdaterStore();
      version.value = updaterStore.getCurrentAppVersion();
    } catch (err) {
      console.warn('Failed to get app version:', err);
    }
  }
});

const openLink = (url: string) => {
  if (window.NL_PORT) {
    Neutralino.os.open(url);
  } else {
    window.open(url, '_blank');
  }
};
</script>

<template>
  <Dialog
    :visible="aboutStore.isOpen"
    modal
    :style="{ width: '24rem' }"
    :closable="true"
    :showHeader="false"
    @update:visible="(val) => { if (!val) aboutStore.close(); }"
  >
    <div class="-mx-6 -mt-6 overflow-hidden rounded-t-2xl">
      <!-- Branded Header -->
      <div
        class="from-primary to-primary-hover relative flex h-32 items-center justify-center overflow-hidden bg-linear-to-br"
      >
        <!-- Decorative background elements -->
        <div class="absolute inset-0 opacity-10">
          <svg class="h-full w-full" viewBox="0 0 100 100" preserveAspectRatio="none">
            <path d="M0 0 L100 100 M100 0 L0 100" stroke="white" stroke-width="0.5" />
          </svg>
        </div>

        <div class="relative flex flex-col items-center gap-2">
          <div
            class="flex h-14 w-14 items-center justify-center rounded-2xl border border-white/20 bg-white/10 shadow-xl backdrop-blur-md"
          >
            <svg
              class="h-8 w-8 text-white"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <line x1="3" y1="9" x2="21" y2="9" />
              <line x1="9" y1="21" x2="9" y2="9" />
            </svg>
          </div>
          <h2 class="text-xl font-bold tracking-tight text-white">Table View</h2>
        </div>
      </div>
    </div>

    <!-- Content -->
    <div class="space-y-6 pt-4">
      <div class="space-y-2.5 text-center">
        <div
          class="bg-primary/10 text-primary inline-flex rounded-full px-2.5 py-0.5 text-[11px] font-bold tracking-wider uppercase"
        >
          Version {{ version }}
        </div>
        <p class="text-text-secondary px-2 text-[13px] leading-relaxed">
          A modern, high-performance database management tool designed for developers who
          value speed, simplicity, and a beautiful user interface.
        </p>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <Button
          variant="text"
          severity="secondary"
          @click="openLink('https://github.com/vantoan1511/table-view')"
        >
          <Github class="h-4 w-4" />
          GitHub
        </Button>
        <Button
          variant="text"
          severity="secondary"
          @click="openLink('https://github.com/vantoan1511')"
        >
          <Globe class="h-4 w-4" />
          Toan Nguyen
        </Button>
      </div>

      <div class="bg-border/50 h-px" />

      <div class="flex flex-col gap-3">
        <button
          type="button"
          @click="openLink('https://www.buymeacoffee.com/vantoan1511')"
          class="flex w-full cursor-pointer items-center justify-center gap-2 rounded-xl border-none bg-[#FFDD00] px-4 py-3 text-sm font-bold text-black shadow-sm transition-all hover:bg-[#FFDD00]/90 active:scale-[0.98]"
        >
          <Coffee class="h-4 w-4" />
          Buy me a coffee
        </button>
        <div class="text-text-tertiary flex items-center justify-center gap-1.5 text-[11px]">
          Made with <Heart :size="10" class="text-danger fill-danger" /> by Toan Nguyen
        </div>
      </div>
    </div>
  </Dialog>
</template>

