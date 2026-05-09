<script setup lang="ts">
import type { ConnectionColor } from '@/types';

const props = defineProps<{
  modelValue: ConnectionColor;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: ConnectionColor];
}>();

const colors: { key: ConnectionColor; value: string }[] = [
  { key: 'indigo', value: '#4F46E5' },
  { key: 'blue', value: '#3B82F6' },
  { key: 'teal', value: '#14B8A6' },
  { key: 'green', value: '#22C55E' },
  { key: 'amber', value: '#F59E0B' },
  { key: 'orange', value: '#F97316' },
  { key: 'pink', value: '#EC4899' },
  { key: 'gray', value: '#9CA3AF' }
];
</script>

<template>
  <div class="flex items-center gap-2.5">
    <button
      v-for="color in colors"
      :key="color.key"
      type="button"
      class="relative h-8 w-8 cursor-pointer rounded-full border-2 transition-all duration-150 hover:scale-110 focus:outline-none"
      :class="
        modelValue === color.key ? 'scale-110 border-current shadow-md' : 'border-transparent'
      "
      :style="{ backgroundColor: color.value, color: color.value }"
      :title="color.key"
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
