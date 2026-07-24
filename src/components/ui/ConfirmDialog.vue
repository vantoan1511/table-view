<script setup lang="ts">
import Dialog from 'primevue/dialog';
import Checkbox from 'primevue/checkbox';
import Button from 'primevue/button';
import { AlertTriangle, Info, Trash2 } from 'lucide-vue-next';

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
    severity: 'danger' as const
  },
  warning: {
    bg: 'bg-warning/10',
    icon: 'text-warning',
    severity: 'warn' as const
  },
  info: {
    bg: 'bg-primary-light',
    icon: 'text-primary',
    severity: 'primary' as const
  }
};
</script>

<template>
  <Dialog
    visible
    modal
    :header="title"
    :style="{ width: '24rem' }"
    :closable="true"
    @update:visible="
      (val) => {
        if (!val) emit('cancel');
      }
    "
  >
    <div class="flex items-start gap-3 py-2">
      <div :class="['shrink-0 rounded-lg p-2', colorMap[variant].bg]">
        <component :is="iconMap[variant]" :size="18" :class="colorMap[variant].icon" />
      </div>
      <div class="min-w-0 flex-1 pt-0.5">
        <p class="text-text-secondary text-[13px] leading-relaxed">{{ message }}</p>
        <div v-if="showCheckbox" class="mt-3">
          <div class="flex items-center gap-2">
            <Checkbox
              :model-value="checkboxValue"
              @update:model-value="(val) => emit('update:checkboxValue', !!val)"
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
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2 pt-2">
        <Button variant="outlined" severity="secondary" @click="emit('cancel')">
          {{ cancelLabel }}
        </Button>
        <Button :severity="colorMap[variant].severity" @click="emit('confirm')">
          {{ confirmLabel }}
        </Button>
      </div>
    </template>
  </Dialog>
</template>
