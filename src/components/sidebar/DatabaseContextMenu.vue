<script setup lang="ts">
import ContextMenu from '../ui/ContextMenu.vue';

import { Database, RefreshCw, Trash2 } from '@lucide/vue';

defineProps<{
  show: boolean;
  x: number;
  y: number;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'action', action: string): void;
}>();
</script>

<template>
  <ContextMenu :show="show" :x="x" :y="y" @close="emit('close')">
    <button
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'refresh')"
    >
      <RefreshCw :size="13" class="text-text-secondary" />
      <span>Refresh</span>
    </button>
    <button
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'createSchema')"
    >
      <Database :size="13" class="text-text-secondary" />
      <span>Create Schema</span>
    </button>
    <div class="bg-border my-1 h-px w-full" />
    <button
      class="text-danger hover:bg-danger/10 flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'deleteDatabase')"
    >
      <Trash2 :size="13" />
      <span>Delete Database</span>
    </button>
  </ContextMenu>
</template>
