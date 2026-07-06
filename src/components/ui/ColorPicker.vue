<script setup lang="ts">
import { ConnectionColor } from '@/types';

defineProps<{
  modelValue: ConnectionColor;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: ConnectionColor];
}>();

const colors: { key: ConnectionColor; value: string }[] = [
  { key: ConnectionColor.INDIGO, value: '#4F46E5' },
  { key: ConnectionColor.BLUE, value: '#3B82F6' },
  { key: ConnectionColor.TEAL, value: '#14B8A6' },
  { key: ConnectionColor.GREEN, value: '#22C55E' },
  { key: ConnectionColor.AMBER, value: '#F59E0B' },
  { key: ConnectionColor.ORANGE, value: '#F97316' },
  { key: ConnectionColor.PINK, value: '#EC4899' },
  { key: ConnectionColor.GRAY, value: '#9CA3AF' },
  { key: ConnectionColor.PURPLE, value: '#A855F7' },
  { key: ConnectionColor.ROSE, value: '#F43F5E' },
  { key: ConnectionColor.EMERALD, value: '#10B981' },
  { key: ConnectionColor.CYAN, value: '#06B6D4' },
  { key: ConnectionColor.VIOLET, value: '#7C3AED' },
  { key: ConnectionColor.RED, value: '#EF4444' }
];
</script>

<template>
  <div class="flex flex-wrap items-center gap-2.5">
    <button
      v-for="color in colors"
      :key="color.key"
      type="button"
      v-tooltip.top="color.key"
      class="relative h-8 w-8 cursor-pointer rounded-full border-2 transition-all duration-150 hover:scale-110 focus:outline-none"
      :class="
        modelValue === color.key ? 'scale-110 border-current shadow-md' : 'border-transparent'
      "
      :style="{ backgroundColor: color.value, color: color.value }"
      @click="emit('update:modelValue', color.key)"
    >
      <!-- Checkmark -->
      <svg
        v-if="modelValue === color.key"
        class="absolute inset-0 m-auto h-4 w-4 text-white"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="3"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="20 6 9 17 4 12" />
      </svg>
    </button>
  </div>
</template>
