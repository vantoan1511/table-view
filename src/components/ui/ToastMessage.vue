<script setup lang="ts">
import type { ToastOptions } from '@/stores/toast';
import { AlertCircle, AlertTriangle, CheckCircle2, Info, X } from 'lucide-vue-next';
import { computed } from 'vue';

const props = defineProps<{
  toast: ToastOptions;
}>();

defineEmits<{
  (e: 'close'): void;
}>();

const defaultIcons = {
  success: CheckCircle2,
  error: AlertCircle,
  warning: AlertTriangle,
  info: Info
};

const iconComponent = computed(() => {
  if (props.toast.icon) return props.toast.icon;
  return defaultIcons[props.toast.severity || 'info'];
});

const variationClass = computed(() => {
  const sev = props.toast.severity || 'info';
  const v = props.toast.variation || 'subtle';

  if (v === 'filled') {
    return {
      'bg-success text-white border-success': sev === 'success',
      'bg-danger text-white border-danger': sev === 'error',
      'bg-warning text-white border-warning': sev === 'warning',
      'bg-primary text-white border-primary': sev === 'info'
    };
  } else if (v === 'outlined') {
    return {
      'bg-surface border-success text-success': sev === 'success',
      'bg-surface border-danger text-danger': sev === 'error',
      'bg-surface border-warning text-warning': sev === 'warning',
      'bg-surface border-primary text-primary': sev === 'info'
    };
  } else {
    // subtle
    return {
      'bg-success/10 border-success/20 text-success': sev === 'success',
      'bg-danger/10 border-danger/20 text-danger': sev === 'error',
      'bg-warning/10 border-warning/20 text-warning': sev === 'warning',
      'bg-primary/10 border-primary/20 text-primary': sev === 'info'
    };
  }
});

const closeBtnClass = computed(() => {
  const v = props.toast.variation || 'subtle';
  if (v === 'filled') {
    return 'text-white/80 hover:text-white';
  }
  return 'opacity-70 hover:opacity-100';
});

const textClass = computed(() => {
  const v = props.toast.variation || 'subtle';
  if (v === 'filled') return 'text-white/90';
  return 'text-text-primary';
});
</script>

<template>
  <div
    class="bg-opacity-75 pointer-events-auto flex max-w-100 min-w-75 items-start gap-3 rounded-xl border p-3.5 shadow-lg backdrop-blur-md"
    :class="variationClass"
  >
    <component :is="iconComponent" :size="20" class="mt-0.5 shrink-0" />
    <div class="min-w-0 flex-1">
      <h4
        v-if="toast.title"
        class="mb-0.5 text-[14px] leading-tight font-semibold"
        :class="toast.variation === 'filled' ? 'text-white' : ''"
      >
        {{ toast.title }}
      </h4>
      <p class="text-[13px] leading-snug wrap-break-word" :class="textClass">
        {{ toast.message }}
      </p>
      <div v-if="toast.actions && toast.actions.length" class="mt-2.5 flex items-center gap-2">
        <button
          v-for="action in toast.actions"
          :key="action.label"
          @click="action.onClick()"
          class="cursor-pointer rounded-md px-2.5 py-1 text-[12px] font-medium transition-all duration-200"
          :class="
            action.primary
              ? toast.variation === 'filled'
                ? 'text-text-primary bg-white shadow-sm hover:bg-white/90'
                : 'bg-primary hover:bg-primary-hover text-white shadow-sm'
              : toast.variation === 'filled'
                ? 'text-white/80 hover:bg-white/10 hover:text-white'
                : 'bg-surface hover:bg-hover border-border text-text-secondary border'
          "
        >
          {{ action.label }}
        </button>
      </div>
    </div>
    <button
      @click="$emit('close')"
      class="shrink-0 cursor-pointer transition-opacity"
      :class="closeBtnClass"
    >
      <X :size="16" />
    </button>
  </div>
</template>
