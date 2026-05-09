<script setup lang="ts">
import { onUnmounted, ref } from 'vue';

const props = defineProps<{
  orientation: 'horizontal' | 'vertical';
  modelValue: number;
  reverse?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void;
  (e: 'resize:start'): void;
  (e: 'resize:end'): void;
}>();

const isDragging = ref(false);

const onMouseDown = (e: MouseEvent) => {
  isDragging.value = true;
  emit('resize:start');

  const startPos = props.orientation === 'horizontal' ? e.clientX : e.clientY;
  const startSize = props.modelValue;

  const onMouseMove = (moveEvent: MouseEvent) => {
    if (!isDragging.value) return;

    const currentPos = props.orientation === 'horizontal' ? moveEvent.clientX : moveEvent.clientY;
    let delta = currentPos - startPos;

    if (props.reverse) {
      delta = -delta;
    }

    emit('update:modelValue', startSize + delta);
  };

  const onMouseUp = () => {
    isDragging.value = false;
    emit('resize:end');
    window.removeEventListener('mousemove', onMouseMove);
    window.removeEventListener('mouseup', onMouseUp);
  };

  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
};

onUnmounted(() => {
  isDragging.value = false;
});
</script>

<template>
  <div
    class="resize-handle group relative transition-colors"
    :class="[
      orientation === 'horizontal'
        ? 'hover:bg-primary/50 w-1 cursor-col-resize'
        : 'hover:bg-primary/50 h-1 cursor-row-resize',
      isDragging ? 'bg-primary' : 'bg-transparent'
    ]"
    @mousedown="onMouseDown"
  >
    <div
      class="absolute inset-0 opacity-0 transition-opacity group-hover:opacity-100"
      :class="orientation === 'horizontal' ? '-right-1 -left-1' : '-top-1 -bottom-1'"
    />
  </div>
</template>
