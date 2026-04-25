<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircle2, AlertCircle, Info, AlertTriangle, X } from 'lucide-vue-next'
import type { ToastOptions } from '@/stores/toast'

const props = defineProps<{
  toast: ToastOptions
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const defaultIcons = {
  success: CheckCircle2,
  error: AlertCircle,
  warning: AlertTriangle,
  info: Info
}

const iconComponent = computed(() => {
  if (props.toast.icon) return props.toast.icon
  return defaultIcons[props.toast.severity || 'info']
})

const variationClass = computed(() => {
  const sev = props.toast.severity || 'info'
  const v = props.toast.variation || 'subtle'

  if (v === 'filled') {
    return {
      'bg-success text-white border-success': sev === 'success',
      'bg-danger text-white border-danger': sev === 'error',
      'bg-warning text-white border-warning': sev === 'warning',
      'bg-primary text-white border-primary': sev === 'info',
    }
  } else if (v === 'outlined') {
    return {
      'bg-surface border-success text-success': sev === 'success',
      'bg-surface border-danger text-danger': sev === 'error',
      'bg-surface border-warning text-warning': sev === 'warning',
      'bg-surface border-primary text-primary': sev === 'info',
    }
  } else {
    // subtle
    return {
      'bg-success/10 border-success/20 text-success': sev === 'success',
      'bg-danger/10 border-danger/20 text-danger': sev === 'error',
      'bg-warning/10 border-warning/20 text-warning': sev === 'warning',
      'bg-primary/10 border-primary/20 text-primary': sev === 'info',
    }
  }
})

const closeBtnClass = computed(() => {
  const v = props.toast.variation || 'subtle'
  if (v === 'filled') {
    return 'text-white/80 hover:text-white'
  }
  return 'opacity-70 hover:opacity-100'
})

const textClass = computed(() => {
  const v = props.toast.variation || 'subtle'
  if (v === 'filled') return 'text-white/90'
  return 'text-text-primary'
})
</script>

<template>
  <div 
    class="flex items-start gap-3 p-3.5 min-w-[300px] max-w-[400px] border rounded-xl shadow-lg pointer-events-auto backdrop-blur-md bg-opacity-75"
    :class="variationClass"
  >
    <component :is="iconComponent" :size="20" class="shrink-0 mt-0.5" />
    <div class="flex-1 min-w-0">
      <h4 v-if="toast.title" class="text-[14px] font-semibold mb-0.5 leading-tight" :class="toast.variation === 'filled' ? 'text-white' : ''">
        {{ toast.title }}
      </h4>
      <p class="text-[13px] leading-snug wrap-break-word" :class="textClass">
        {{ toast.message }}
      </p>
    </div>
    <button @click="$emit('close')" class="shrink-0 transition-opacity cursor-pointer" :class="closeBtnClass">
      <X :size="16" />
    </button>
  </div>
</template>
