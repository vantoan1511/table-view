<script setup lang="ts">
import ContextMenu from '../ui/ContextMenu.vue';

import { useConnectionsStore } from '@/stores/connections';

import { Code2, Copy, Database, Pencil, Plug, RefreshCw, Trash2, Unplug } from 'lucide-vue-next';
import { computed } from 'vue';

const props = defineProps<{
  show: boolean;
  x: number;
  y: number;
  connectionId: string | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'action', action: string): void;
}>();

const connectionsStore = useConnectionsStore();
const connection = computed(() =>
  props.connectionId ? connectionsStore.connections.find((c) => c.id === props.connectionId) : null
);
const isConnected = computed(() => connection.value?.isConnected ?? false);
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
      v-if="!isConnected"
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'connect')"
    >
      <Plug :size="13" class="text-text-secondary" />
      <span>Connect</span>
    </button>
    <button
      v-if="isConnected"
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'disconnect')"
    >
      <Unplug :size="13" class="text-text-secondary" />
      <span>Disconnect</span>
    </button>
    <button
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'refresh')"
    >
      <RefreshCw :size="13" class="text-text-secondary" />
      <span>Refresh Connection</span>
    </button>
    <button
      class="text-text-primary hover:bg-hover flex w-full items-center gap-2 px-3 py-1.5 text-[12px]"
      @click="emit('action', 'createDatabase')"
    >
      <Database :size="13" class="text-text-secondary" />
      <span>Create Database</span>
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
