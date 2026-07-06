<script setup lang="ts">
import { AlertTriangle, Info, Trash2, X } from 'lucide-vue-next';

interface Props {
  title: string;
  message: string;
  variant?: 'danger' | 'warning' | 'info';
  confirmLabel?: string;
  cancelLabel?: string;
  showCheckbox?: boolean;
  checkboxLabel?: string;
  checkboxValue?: boolean;
}

withDefaults(defineProps<Props>(), {
  variant: 'danger',
  confirmLabel: 'Confirm',
  cancelLabel: 'Cancel',
  showCheckbox: false,
  checkboxLabel: '',
  checkboxValue: false
});

const emit = defineEmits<{
  confirm: [];
  cancel: [];
  'update:checkboxValue': [value: boolean];
}>();

const iconMap = {
  danger: Trash2,
  warning: AlertTriangle,
  info: Info
};

const colorMap = {
  danger: {
    bg: 'bg-danger-light',
    icon: 'text-danger',
    variant: 'danger' as const
  },
  warning: {
    bg: 'bg-warning/10',
    icon: 'text-warning',
    variant: 'warning' as const
  },
  info: {
    bg: 'bg-primary-light',
    icon: 'text-primary',
    variant: 'primary' as const
  }
};
</script>

<template>
  <!-- Backdrop -->
  <Teleport to="body">
    <div class="fixed inset-0 z-200 flex items-center justify-center" @click.self="emit('cancel')">
      <!-- Scrim -->
      <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" @click="emit('cancel')" />

      <!-- Dialog -->
      <div
        class="bg-surface shadow-modal border-border relative z-10 mx-4 w-full max-w-sm animate-[scale-in_0.15s_ease-out] overflow-hidden rounded-xl border"
        role="dialog"
        aria-modal="true"
      >
        <!-- Header -->
        <div class="flex items-start gap-3 p-5 pb-3">
          <div :class="['shrink-0 rounded-lg p-2', colorMap[variant].bg]">
            <component :is="iconMap[variant]" :size="18" :class="colorMap[variant].icon" />
          </div>
          <div class="min-w-0 flex-1 pt-0.5">
            <h3 class="text-text-primary text-[14px] leading-tight font-semibold">{{ title }}</h3>
            <p class="text-text-secondary mt-1 text-[13px] leading-relaxed">{{ message }}</p>
            <div v-if="showCheckbox" class="mt-3">
              <div class="flex items-center gap-2">
                <Checkbox
                  :model-value="checkboxValue"
                  @update:model-value="(val) => emit('update:checkboxValue', val)"
                  inputId="confirm-dialog-check"
                  binary
                />
                <label
                  for="confirm-dialog-check"
                  class="text-text-secondary cursor-pointer text-sm select-none"
                >
                  {{ checkboxLabel }}
                </label>
              </div>
            </div>
          </div>
          <Button variant="text" severity="secondary" @click="emit('cancel')">
            <template #icon>
              <X class="h-4 w-4" />
            </template>
          </Button>
        </div>

        <!-- Divider -->
        <div class="bg-border mx-5 h-px" />

        <!-- Footer -->
        <div class="flex items-center justify-end gap-2 p-4">
          <Button variant="outlined" severity="secondary" @click="emit('cancel')">
            {{ cancelLabel }}
          </Button>
          <Button :severity="colorMap[variant].variant" @click="emit('confirm')">
            {{ confirmLabel }}
          </Button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
@keyframes scale-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }

  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
