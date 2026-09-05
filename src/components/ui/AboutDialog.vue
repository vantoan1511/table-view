<script setup lang="ts">
import Button from 'primevue/button';
import Dialog from 'primevue/dialog';

import { useAboutStore } from '@/stores/about';
import { useUpdaterStore } from '@/stores/updater';
import { os } from '@/services/nativeService';
import { Coffee, Github, Globe, Heart, Layers } from 'lucide-vue-next';
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
    os.open(url);
  } else {
    window.open(url, '_blank');
  }
};
</script>

<template>
  <Dialog
    :visible="aboutStore.isOpen"
    modal
    header="About Table View"
    :style="{ width: '26rem' }"
    @update:visible="
      (val) => {
        if (!val) aboutStore.close();
      }
    "
  >
    <div class="flex flex-col items-center gap-5 py-1 text-center">
      <!-- App Icon & Title -->
      <div class="flex flex-col items-center gap-2">
        <div
          class="bg-primary/10 text-primary border-primary/20 flex h-13 w-13 items-center justify-center rounded-xl border"
        >
          <Layers :size="26" />
        </div>
        <div>
          <h2 class="text-text-primary text-base font-bold tracking-tight">Table View</h2>
          <span
            class="bg-hover text-text-secondary mt-1 inline-flex rounded-full px-2.5 py-0.5 text-[11px] font-semibold tracking-wider uppercase"
          >
            Version {{ version }}
          </span>
        </div>
      </div>

      <!-- Description -->
      <p class="text-text-secondary max-w-sm text-xs leading-relaxed">
        A lightweight, high-performance desktop database management client designed for technical
        users who value speed, efficiency, and clarity.
      </p>

      <!-- Links -->
      <div class="grid w-full grid-cols-2 gap-2">
        <Button
          variant="outlined"
          severity="secondary"
          size="small"
          class="w-full justify-center gap-2"
          @click="openLink('https://github.com/vantoan1511/table-view')"
        >
          <Github :size="14" />
          <span>GitHub</span>
        </Button>
        <Button
          variant="outlined"
          severity="secondary"
          size="small"
          class="w-full justify-center gap-2"
          @click="openLink('https://github.com/vantoan1511')"
        >
          <Globe :size="14" />
          <span>Website</span>
        </Button>
      </div>

      <!-- Footer Action -->
      <div class="flex w-full flex-col items-center gap-3">
        <Button
          severity="warn"
          size="small"
          class="w-full justify-center gap-2 font-medium"
          @click="openLink('https://www.buymeacoffee.com/vantoan1511')"
        >
          <Coffee :size="14" />
          <span>Buy me a coffee</span>
        </Button>
        <div class="text-text-tertiary flex items-center justify-center gap-1.5 text-[11px]">
          Made with <Heart :size="10" class="text-danger fill-danger" /> by Toan Nguyen
        </div>
      </div>
    </div>
  </Dialog>
</template>
