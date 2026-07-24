<script setup lang="ts">
import ToggleSwitch from 'primevue/toggleswitch';

defineProps<{
  modelValue: boolean;
  label?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();
</script>

<template>
  <label class="inline-flex cursor-pointer items-center gap-2 select-none">
    <ToggleSwitch
      :model-value="modelValue"
      @update:model-value="(val) => emit('update:modelValue', val)"
    />
    <span v-if="label" class="text-text-secondary text-sm">{{ label }}</span>
  </label>
</template>

