<script setup lang="ts">
import { AlertTriangle, Info, AlertCircle, Trash2, X } from 'lucide-vue-next'

interface Props {
  title: string
  message: string
  variant?: 'danger' | 'warning' | 'info'
  confirmLabel?: string
  cancelLabel?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'danger',
  confirmLabel: 'Confirm',
  cancelLabel: 'Cancel',
})

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const iconMap = {
  danger: Trash2,
  warning: AlertTriangle,
  info: Info,
}

const colorMap = {
  danger: {
    bg: 'bg-danger-light',
    icon: 'text-danger',
    btn: 'bg-danger hover:bg-danger/90 text-white',
  },
  warning: {
    bg: 'bg-warning/10',
    icon: 'text-warning',
    btn: 'bg-warning hover:bg-warning/90 text-white',
  },
  info: {
    bg: 'bg-primary-light',
    icon: 'text-primary',
    btn: 'bg-primary hover:bg-primary-hover text-text-inverse',
  },
}
</script>

<template>
  <!-- Backdrop -->
  <Teleport to="body">
    <div
      class="fixed inset-0 z-[200] flex items-center justify-center"
      @click.self="emit('cancel')"
    >
      <!-- Scrim -->
      <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" @click="emit('cancel')" />

      <!-- Dialog -->
      <div
        class="relative z-10 w-full max-w-sm mx-4 bg-surface rounded-xl shadow-modal border border-border overflow-hidden
               animate-[scale-in_0.15s_ease-out]"
        role="dialog"
        aria-modal="true"
      >
        <!-- Header -->
        <div class="flex items-start gap-3 p-5 pb-3">
          <div :class="['p-2 rounded-lg shrink-0', colorMap[variant].bg]">
            <component :is="iconMap[variant]" :size="18" :class="colorMap[variant].icon" />
          </div>
          <div class="flex-1 min-w-0 pt-0.5">
            <h3 class="text-[14px] font-semibold text-text-primary leading-tight">{{ title }}</h3>
            <p class="mt-1 text-[13px] text-text-secondary leading-relaxed">{{ message }}</p>
          </div>
          <button
            class="shrink-0 p-1 rounded-md text-text-tertiary hover:text-text-secondary hover:bg-hover transition-colors"
            @click="emit('cancel')"
          >
            <X :size="16" />
          </button>
        </div>

        <!-- Divider -->
        <div class="h-px bg-border mx-5" />

        <!-- Footer -->
        <div class="flex items-center justify-end gap-2 p-4">
          <button
            class="px-4 py-2 rounded-lg text-[13px] font-medium text-text-secondary hover:bg-hover border border-border transition-colors cursor-pointer"
            @click="emit('cancel')"
          >
            {{ cancelLabel }}
          </button>
          <button
            :class="['px-4 py-2 rounded-lg text-[13px] font-medium transition-colors shadow-sm cursor-pointer', colorMap[variant].btn]"
            @click="emit('confirm')"
          >
            {{ confirmLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
@keyframes scale-in {
  from { opacity: 0; transform: scale(0.95); }
  to   { opacity: 1; transform: scale(1); }
}
</style>
