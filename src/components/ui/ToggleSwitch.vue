<script setup lang="ts">
const props = defineProps<{
  modelValue: boolean;
  label?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();
</script>

<template>
  <label class="inline-flex cursor-pointer items-center gap-2 select-none">
    <button
      type="button"
      role="switch"
      :aria-checked="modelValue"
      class="focus-visible:ring-primary relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2"
      :class="modelValue ? 'bg-primary' : 'bg-gray-300'"
      @click="emit('update:modelValue', !modelValue)"
    >
      <span
        class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-sm ring-0 transition duration-200 ease-in-out"
        :class="modelValue ? 'translate-x-4' : 'translate-x-0'"
      />
    </button>
    <span v-if="label" class="text-text-secondary text-sm">{{ label }}</span>
  </label>
</template>
