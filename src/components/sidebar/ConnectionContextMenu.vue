<script setup lang="ts">
import { Code2, Copy, Pencil, RefreshCw, Trash2 } from 'lucide-vue-next';
import ContextMenu from '../ui/ContextMenu.vue';

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
      @click="emit('action', 'sql')"
    >
      <Code2 :size="13" class="text-text-secondary" />
      <span>Open SQL Editor</span>
    </button>
    <button
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'refresh')"
    >
      <RefreshCw :size="13" class="text-text-secondary" />
      <span>Refresh Schema</span>
    </button>
    <button
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'edit')"
    >
      <Pencil :size="13" class="text-text-secondary" />
      <span>Edit Connection</span>
    </button>
    <button
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'duplicate')"
    >
      <Copy :size="13" class="text-text-secondary" />
      <span>Duplicate</span>
    </button>
    <div class="bg-border my-1 h-px w-full" />
    <button
      class="text-danger hover:bg-danger-light flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'delete')"
    >
      <Trash2 :size="13" />
      <span>Delete Connection</span>
    </button>
  </ContextMenu>
</template>
