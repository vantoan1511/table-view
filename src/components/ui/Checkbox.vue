<script setup lang="ts">
import { Check } from 'lucide-vue-next';

const props = defineProps<{
  modelValue: boolean;
  disabled?: boolean;
  label?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

const toggle = () => {
  if (props.disabled) return;
  emit('update:modelValue', !props.modelValue);
};
</script>

<template>
  <label
    class="inline-flex items-center gap-2"
    :class="[disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer']"
    @click.prevent="toggle"
  >
    <div
      class="flex h-4 w-4 items-center justify-center rounded border transition-all duration-200"
      :class="[
        modelValue
          ? 'bg-primary border-primary'
          : 'bg-surface border-border hover:border-text-tertiary',
        disabled ? 'border-border' : ''
      ]"
    >
      <Check v-if="modelValue" :size="12" class="text-white" stroke-width="3" />
    </div>
    <span v-if="label" class="text-text-secondary text-sm select-none">{{ label }}</span>
  </label>
</template>
