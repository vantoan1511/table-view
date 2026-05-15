<script setup lang="ts">
import { ChevronDown } from 'lucide-vue-next';
import { computed, onMounted, onUnmounted, ref } from 'vue';

export type DropdownValue = string | number;

export interface DropdownOption {
  label: string;
  value: DropdownValue;
}

const props = withDefaults(
  defineProps<{
    modelValue: DropdownValue;
    options: DropdownOption[];
    ariaLabel?: string;
    placement?: 'top' | 'bottom';
    align?: 'left' | 'right';
    buttonClass?: string;
    menuClass?: string;
    showChevron?: boolean;
  }>(),
  {
    ariaLabel: 'Select option',
    placement: 'bottom',
    align: 'left',
    buttonClass: '',
    menuClass: '',
    showChevron: true
  }
);

const emit = defineEmits<{
  'update:modelValue': [value: DropdownValue];
  select: [value: DropdownValue];
}>();

const isOpen = ref(false);
const rootRef = ref<HTMLElement | null>(null);

const selectedOption = computed(() =>
  props.options.find((option) => option.value === props.modelValue)
);

const menuPositionClass = computed(() => [
  props.placement === 'top' ? 'bottom-full mb-1' : 'top-full mt-1',
  props.align === 'right' ? 'right-0' : 'left-0'
]);

const toggle = () => {
  isOpen.value = !isOpen.value;
};

const close = () => {
  isOpen.value = false;
};

const selectOption = (value: DropdownValue) => {
  emit('update:modelValue', value);
  emit('select', value);
  close();
};

const handleClickOutside = (event: MouseEvent) => {
  if (isOpen.value && rootRef.value && !rootRef.value.contains(event.target as Node)) {
    close();
  }
};

const handleEscape = (event: KeyboardEvent) => {
  if (isOpen.value && event.key === 'Escape') {
    close();
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('keydown', handleEscape);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('keydown', handleEscape);
});
</script>

<template>
  <div ref="rootRef" class="relative inline-flex">
    <button
      type="button"
      class="border-border hover:bg-hover flex cursor-pointer items-center gap-1 rounded-md border px-2 py-1 text-[12px] transition-colors"
      :class="buttonClass"
      :aria-label="ariaLabel"
      :aria-expanded="isOpen"
      aria-haspopup="listbox"
      @click.stop="toggle"
    >
      <slot name="trigger" :open="isOpen" :selected="selectedOption">
        {{ selectedOption?.label ?? modelValue }}
      </slot>
      <ChevronDown
        v-if="showChevron"
        :size="12"
        :class="{ 'rotate-180': isOpen }"
        class="transition-transform"
      />
    </button>

    <div
      v-if="isOpen"
      class="bg-surface border-border absolute z-50 min-w-20 rounded-lg border py-1 shadow-lg"
      :class="[menuPositionClass, menuClass]"
      role="listbox"
      @click.stop
    >
      <slot :close="close" :select="selectOption">
        <button
          v-for="option in options"
          :key="option.value"
          type="button"
          class="hover:bg-hover flex w-full cursor-pointer items-center justify-between gap-3 px-3 py-1.5 text-left text-[12px]"
          :class="modelValue === option.value ? 'text-primary font-medium' : 'text-text-primary'"
          role="option"
          :aria-selected="modelValue === option.value"
          @click="selectOption(option.value)"
        >
          <span>{{ option.label }}</span>
          <span v-if="modelValue === option.value" class="text-primary text-[10px]">✓</span>
        </button>
      </slot>
    </div>
  </div>
</template>
