<script setup lang="ts">
import Button from 'primevue/button';
import { ExternalLink, Info } from 'lucide-vue-next';

import { isAvailable, os } from '@/services/nativeService';

export interface MigrationBannerProps {
  targetVersion?: string;
  learnMoreUrl?: string;
}

const props = withDefaults(defineProps<MigrationBannerProps>(), {
  targetVersion: '0.8.0',
  learnMoreUrl: 'https://github.com/vantoan1511/table-view/issues/90'
});

const onLearnMore = async () => {
  if (isAvailable()) {
    await os.open(props.learnMoreUrl);
    return;
  }

  window.open(props.learnMoreUrl, '_blank', 'noopener,noreferrer');
};
</script>

<template>
  <div
    data-testid="migration-banner"
    class="flex shrink-0 items-center justify-between gap-3 border-b border-(--color-border) bg-(--color-primary-light) px-3 py-1.5 text-xs text-(--color-text-primary)"
  >
    <div class="flex min-w-0 items-center gap-2">
      <Info :size="14" class="shrink-0 text-(--color-primary)" />
      <p class="truncate">
        <span class="font-medium">0.7.9 Legacy Release:</span>
        <span class="text-(--color-text-secondary)">
          Table View {{ targetVersion }} is moving to a fully native Rust/GPUI desktop engine.
        </span>
      </p>
    </div>

    <Button
      data-testid="btn-migration-learn-more"
      label="Learn More"
      variant="text"
      size="small"
      class="h-6 shrink-0 px-2 text-xs"
      @click="onLearnMore"
    >
      <template #icon>
        <ExternalLink :size="12" class="mr-1" />
      </template>
    </Button>
  </div>
</template>
