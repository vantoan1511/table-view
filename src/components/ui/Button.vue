<script setup lang="ts">
import { Loader2 } from 'lucide-vue-next';
import { computed } from 'vue';

interface Props {
  variant?:
    | 'primary'
    | 'secondary'
    | 'outline'
    | 'ghost'
    | 'danger'
    | 'warning'
    | 'success'
    | 'subtle'
    | 'none';
  size?: 'sm' | 'md' | 'lg' | 'icon';
  loading?: boolean;
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
  icon?: any;
  iconRight?: any;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'secondary',
  size: 'md',
  loading: false,
  disabled: false,
  type: 'button'
});

const variantClasses = {
  primary: 'bg-primary hover:bg-primary-hover text-text-inverse shadow-sm',
  secondary: 'text-text-secondary hover:bg-hover border-border border',
  outline: 'text-text-secondary hover:bg-hover border-border border',
  ghost: 'text-text-tertiary hover:text-text-secondary hover:bg-hover border border-transparent',
  danger: 'bg-danger hover:bg-danger/90 text-white shadow-sm',
  warning: 'bg-warning hover:bg-warning/90 text-white shadow-sm',
  success: 'bg-success hover:bg-success/90 text-white shadow-sm',
  subtle: 'bg-active text-primary border-primary/20 border font-medium',
  none: ''
};

const sizeClasses = {
  sm: 'px-2.5 py-1.5 text-[12px] gap-1.5 rounded-md',
  md: 'px-4 py-2 text-[13px] gap-2 rounded-lg',
  lg: 'px-6 py-3 text-[14px] gap-2.5 rounded-xl',
  icon: 'p-1.5 rounded-md'
};

const classes = computed(() => {
  if (props.variant === 'none') return '';

  return [
    'inline-flex items-center justify-center font-medium transition-all duration-200 cursor-pointer active:scale-[0.98]',
    variantClasses[props.variant],
    sizeClasses[props.size],
    (props.disabled || props.loading) &&
      'opacity-60 cursor-not-allowed pointer-events-none active:scale-100'
  ]
    .filter(Boolean)
    .join(' ');
});
</script>

<template>
  <button :type="type" :class="classes" :disabled="disabled || loading">
    <Loader2 v-if="loading" :size="size === 'sm' ? 14 : 16" class="shrink-0 animate-spin" />
    <component
      v-if="icon && !loading"
      :is="icon"
      :size="size === 'sm' ? 14 : 16"
      class="shrink-0"
    />
    <slot v-if="size !== 'icon'" />
    <component
      v-if="iconRight && !loading"
      :is="iconRight"
      :size="size === 'sm' ? 14 : 16"
      class="shrink-0"
    />
  </button>
</template>
