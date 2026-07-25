<script setup lang="ts">
import { ChevronDown } from '@lucide/vue';
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';

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
const menuRef = ref<HTMLElement | null>(null);
const menuStyle = ref<Record<string, string>>({});

const selectedOption = computed(() =>
  props.options.find((option) => option.value === props.modelValue)
);

const updatePosition = () => {
  if (!rootRef.value || !isOpen.value) return;

  const rect = rootRef.value.getBoundingClientRect();
  const windowWidth = window.innerWidth;
  const windowHeight = window.innerHeight;

  const style: Record<string, string> = {
    minWidth: `${rect.width}px`,
    position: 'fixed',
    zIndex: '9999'
  };

  // Vertical positioning
  if (props.placement === 'top') {
    style.bottom = `${windowHeight - rect.top + 4}px`;
  } else {
    style.top = `${rect.bottom + 4}px`;
  }

  // Horizontal positioning
  if (props.align === 'right') {
    style.right = `${windowWidth - rect.right}px`;
  } else {
    style.left = `${rect.left}px`;
  }

  menuStyle.value = style;

  // Handle screen boundary for bottom placement
  if (props.placement === 'bottom') {
    nextTick(() => {
      if (menuRef.value) {
        const menuRect = menuRef.value.getBoundingClientRect();
        if (menuRect.bottom > windowHeight - 10) {
          // Flip to top if not enough space at bottom
          style.top = 'auto';
          style.bottom = `${windowHeight - rect.top + 4}px`;
          menuStyle.value = { ...style };
        }
      }
    });
  }
};

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
    // Also check if click is inside the teleported menu
    if (menuRef.value && menuRef.value.contains(event.target as Node)) {
      return;
    }
    close();
  }
};

const handleEscape = (event: KeyboardEvent) => {
  if (isOpen.value && event.key === 'Escape') {
    close();
  }
};

watch(isOpen, (val) => {
  if (val) {
    nextTick(updatePosition);
  }
});

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('keydown', handleEscape);
  window.addEventListener('resize', updatePosition);
  window.addEventListener('scroll', updatePosition, true);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('keydown', handleEscape);
  window.removeEventListener('resize', updatePosition);
  window.removeEventListener('scroll', updatePosition, true);
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

    <Teleport to="body">
      <div
        v-if="isOpen"
        ref="menuRef"
        class="bg-surface border-border fixed z-9999 min-w-20 rounded-lg border py-1 shadow-lg"
        :style="menuStyle"
        :class="menuClass"
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
    </Teleport>
  </div>
</template>
