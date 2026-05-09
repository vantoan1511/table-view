<script setup lang="ts">
import { ref } from 'vue';

const props = withDefaults(
  defineProps<{
    text: string;
    position?: 'top' | 'bottom' | 'left' | 'right';
    delay?: number;
  }>(),
  {
    position: 'bottom',
    delay: 0
  }
);

const show = ref(false);
let timeout: any = null;

const onMouseEnter = () => {
  timeout = setTimeout(() => {
    show.value = true;
  }, props.delay);
};

const onMouseLeave = () => {
  clearTimeout(timeout);
  show.value = false;
};
</script>

<template>
  <div class="group relative" @mouseenter="onMouseEnter" @mouseleave="onMouseLeave">
    <slot />

    <Teleport to="body">
      <div
        v-if="show && text"
        class="bg-surface border-border text-text-primary pointer-events-none fixed z-9999 rounded border px-2 py-1 text-[11px] font-medium whitespace-nowrap shadow-xl transition-all"
        :class="[
          position === 'bottom' ? 'translate-y-1' : '',
          position === 'top' ? '-translate-y-1' : '',
          position === 'left' ? '-translate-x-1' : '',
          position === 'right' ? 'translate-x-1' : ''
        ]"
        :style="{} /* We'll use a better positioning strategy */"
      >
        {{ text }}
      </div>
    </Teleport>
  </div>
</template>
