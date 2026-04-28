<script setup lang="ts">
import { useGridStore } from '@/stores/grid'
import { computed } from 'vue'

const gridStore = useGridStore()
const messages = computed(() => gridStore.sqlMessages)
</script>

<template>
  <div class="flex-1 overflow-auto p-4 font-mono text-[12px] bg-surface">
    <div v-if="messages.length === 0" class="text-text-tertiary italic">
      No output generated yet.
    </div>
    <div v-for="(msg, i) in messages" :key="i" class="mb-2 last:mb-0">
      <div class="flex items-center gap-2 mb-1 opacity-70 text-[10px]">
        <span class="text-primary">{{ new Date(msg.timestamp).toLocaleTimeString() }}</span>
      </div>
      <div :class="{
        'text-success': msg.type === 'info',
        'text-danger': msg.type === 'error'
      }">
        {{ msg.text }}
      </div>
    </div>
  </div>
</template>
