<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';

const props = defineProps<{
  show: boolean;
  x: number;
  y: number;
  widthClass?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const menuRef = ref<HTMLElement | null>(null);

const handleClickOutside = (e: MouseEvent) => {
  if (props.show && menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit('close');
  }
};

const handleEscape = (e: KeyboardEvent) => {
  if (props.show && e.key === 'Escape') {
    emit('close');
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('contextmenu', handleClickOutside);
  document.addEventListener('keydown', handleEscape);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('contextmenu', handleClickOutside);
  document.removeEventListener('keydown', handleEscape);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="context-menu">
      <div
        v-if="show"
        ref="menuRef"
        class="bg-surface border-border context-menu fixed z-9999 rounded-lg border py-1.5 shadow-xl"
        :class="widthClass || 'w-52'"
        :style="{ top: y + 'px', left: x + 'px' }"
        @click.stop
      >
        <slot></slot>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.context-menu-enter-active,
.context-menu-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
  transform-origin: top left;
}

.context-menu-enter-from,
.context-menu-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
