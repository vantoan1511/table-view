<script setup lang="ts">
import { useToastStore } from '@/stores/toast';
import { computed } from 'vue';
import ToastMessage from './ToastMessage.vue';

const toastStore = useToastStore();

const groupedToasts = computed(() => {
  const groups: Record<string, typeof toastStore.toasts> = {
    'top-right': [],
    'top-left': [],
    'bottom-right': [],
    'bottom-left': [],
    'top-center': [],
    'bottom-center': []
  };
  for (const toast of toastStore.toasts) {
    const pos = toast.position || 'bottom-right';
    if (groups[pos]) {
      groups[pos].push(toast);
    } else {
      groups['bottom-right']?.push(toast);
    }
  }
  return groups;
});

const getPositionClass = (pos: string) => {
  switch (pos) {
    case 'top-right':
      return 'top-4 right-4 items-end';
    case 'top-left':
      return 'top-4 left-4 items-start';
    case 'top-center':
      return 'top-4 left-1/2 -translate-x-1/2 items-center';
    case 'bottom-right':
      return 'bottom-4 right-4 items-end flex-col-reverse';
    case 'bottom-left':
      return 'bottom-4 left-4 items-start flex-col-reverse';
    case 'bottom-center':
      return 'bottom-4 left-1/2 -translate-x-1/2 items-center flex-col-reverse';
    default:
      return 'bottom-4 right-4 items-end flex-col-reverse';
  }
};
</script>

<template>
  <div class="pointer-events-none fixed inset-0 z-9999 overflow-hidden">
    <TransitionGroup
      v-for="(toasts, pos) in groupedToasts"
      :key="pos"
      name="toast"
      tag="div"
      class="pointer-events-none absolute flex gap-3"
      :class="getPositionClass(String(pos))"
    >
      <ToastMessage
        v-for="toast in toasts"
        :key="toast.id"
        :toast="toast"
        @close="toastStore.removeToast(toast.id!)"
      />
    </TransitionGroup>
  </div>
</template>

<style scoped>
/* Toast transition animations */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.9);
}

.toast-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

/* For top positions, we probably want to slide down instead of up */
.top-4 .toast-enter-from {
  transform: translateY(-20px) scale(0.9);
}
</style>
