<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  position?: 'bottom' | 'top' | 'left' | 'right'
}>()

const positionClass = computed(() => {
  switch (props.position) {
    case 'top':
      return 'border-b border-border'
    case 'left':
      return 'border-r border-border flex-col'
    case 'right':
      return 'border-l border-border flex-col'
    case 'bottom':
    default:
      return 'border-t border-border'
  }
})
</script>

<template>
  <div
    class="flex items-center gap-2 px-2 py-1.5 bg-surface overflow-x-auto shrink-0 w-full"
    :class="positionClass"
  >
    <slot></slot>
  </div>
</template>
