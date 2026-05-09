<script setup lang="ts">
import { 
  Code2, 
  Copy, 
  Pencil, 
  RefreshCw, 
  Trash2 
} from 'lucide-vue-next'
import ContextMenu from '../ui/ContextMenu.vue'

defineProps<{
  show: boolean
  x: number
  y: number
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'action', action: string): void
}>()
</script>

<template>
  <ContextMenu :show="show" :x="x" :y="y" @close="emit('close')">
    <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover"
      @click="emit('action', 'sql')">
      <Code2 :size="13" class="text-text-secondary" />
      <span>Open SQL Editor</span>
    </button>
    <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover"
      @click="emit('action', 'refresh')">
      <RefreshCw :size="13" class="text-text-secondary" />
      <span>Refresh Schema</span>
    </button>
    <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover"
      @click="emit('action', 'edit')">
      <Pencil :size="13" class="text-text-secondary" />
      <span>Edit Connection</span>
    </button>
    <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover"
      @click="emit('action', 'duplicate')">
      <Copy :size="13" class="text-text-secondary" />
      <span>Duplicate</span>
    </button>
    <div class="h-px bg-border my-1 w-full" />
    <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-danger hover:bg-danger-light"
      @click="emit('action', 'delete')">
      <Trash2 :size="13" />
      <span>Delete Connection</span>
    </button>
  </ContextMenu>
</template>
