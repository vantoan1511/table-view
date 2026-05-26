<script setup lang="ts">
import { Loader2 } from 'lucide-vue-next';
import { computed } from 'vue';

type Severity =
  | 'primary'
  | 'secondary'
  | 'success'
  | 'info'
  | 'warning'
  | 'danger'
  | 'help'
  | 'contrast';

type StructuralVariant = 'filled' | 'outline' | 'ghost' | 'subtle' | 'none';

type LegacyVariant =
  | 'primary'
  | 'secondary'
  | 'outline'
  | 'ghost'
  | 'danger'
  | 'warning'
  | 'success'
  | 'subtle'
  | 'none';

interface Props {
  variant?: StructuralVariant | LegacyVariant;
  severity?: Severity;
  label?: string;
  size?: 'sm' | 'md' | 'lg' | 'icon';
  loading?: boolean;
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
  icon?: any;
  iconRight?: any;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  loading: false,
  disabled: false,
  type: 'button'
});

// Resolve the effective severity for backward compatibility
const resolvedSeverity = computed<Severity>(() => {
  if (props.severity) return props.severity;

  if (props.variant === 'primary') return 'primary';
  if (props.variant === 'danger') return 'danger';
  if (props.variant === 'warning') return 'warning';
  if (props.variant === 'success') return 'success';
  if (props.variant === 'subtle') return 'primary';
  if (props.variant === 'outline') return 'secondary';
  if (props.variant === 'ghost') return 'secondary';
  if (props.variant === 'secondary') return 'secondary';

  return 'secondary'; // Default severity
});

// Resolve the effective structural variant
const resolvedVariant = computed<StructuralVariant>(() => {
  if (props.variant === 'none') return 'none';
  if (props.severity) {
    // If severity is explicitly provided, default to 'filled' variant
    return (props.variant as StructuralVariant) || 'filled';
  }

  // Backward compatibility: map legacy variant to structural style
  if (props.variant === 'outline') return 'outline';
  if (props.variant === 'ghost') return 'ghost';
  if (props.variant === 'subtle') return 'subtle';
  if (props.variant === 'secondary') return 'outline'; // Legacy secondary was outline style

  return 'filled'; // Default structural variant
});

const buttonStyles: Record<Exclude<StructuralVariant, 'none'>, Record<Severity, string>> = {
  filled: {
    primary: 'bg-primary hover:bg-primary-hover text-text-inverse shadow-sm',
    secondary: 'bg-surface hover:bg-hover border-border border text-text-primary shadow-sm',
    success: 'bg-success hover:bg-success/90 text-white shadow-sm',
    info: 'bg-primary hover:bg-primary-hover text-text-inverse shadow-sm',
    warning: 'bg-warning hover:bg-warning/90 text-white shadow-sm',
    danger: 'bg-danger hover:bg-danger/90 text-white shadow-sm',
    help: 'bg-purple-600 hover:bg-purple-700 text-white shadow-sm',
    contrast:
      'bg-text-primary hover:bg-text-secondary text-surface shadow-sm border border-transparent'
  },
  outline: {
    primary: 'text-primary hover:bg-primary-light border-primary border shadow-sm',
    secondary: 'text-text-secondary hover:bg-hover border-border border',
    success: 'text-success hover:bg-success-light border-success border shadow-sm',
    info: 'text-primary hover:bg-primary-light border-primary border shadow-sm',
    warning: 'text-warning hover:bg-warning/10 border-warning border shadow-sm',
    danger: 'text-danger hover:bg-danger-light border-danger border shadow-sm',
    help: 'text-purple-600 hover:bg-purple-600/10 border-purple-600 border shadow-sm',
    contrast: 'text-text-primary hover:bg-text-primary/10 border-text-primary border shadow-sm'
  },
  ghost: {
    primary: 'text-primary hover:bg-primary-light',
    secondary:
      'text-text-tertiary hover:text-text-secondary hover:bg-hover border border-transparent',
    success: 'text-success hover:bg-success-light',
    info: 'text-primary hover:bg-primary-light',
    warning: 'text-warning hover:bg-warning/10',
    danger: 'text-danger hover:bg-danger-light',
    help: 'text-purple-600 hover:bg-purple-600/10',
    contrast: 'text-text-primary hover:bg-text-primary/10'
  },
  subtle: {
    primary: 'bg-active text-primary border-primary/20 border font-medium',
    secondary: 'bg-muted text-text-secondary border-border/50 border font-medium',
    success: 'bg-success-light text-success border-success/20 border font-medium',
    info: 'bg-active text-primary border-primary/20 border font-medium',
    warning: 'bg-warning/10 text-warning border-warning/20 border font-medium',
    danger: 'bg-danger-light text-danger border-danger/20 border font-medium',
    help: 'bg-purple-600/10 text-purple-600 border-purple-600/20 border font-medium',
    contrast: 'bg-muted text-text-primary border-border border font-medium'
  }
};

const sizeClasses = {
  sm: 'px-2.5 py-1.5 text-[12px] gap-1.5 rounded-md',
  md: 'px-4 py-2 text-[13px] gap-2 rounded-lg',
  lg: 'px-6 py-3 text-[14px] gap-2.5 rounded-xl',
  icon: 'p-1.5 rounded-md'
};

const classes = computed(() => {
  const variant = resolvedVariant.value;
  if (variant === 'none') return '';

  const severity = resolvedSeverity.value;
  const styleClass = buttonStyles[variant]?.[severity] || '';

  return [
    'inline-flex items-center justify-center font-medium transition-all duration-200 cursor-pointer active:scale-[0.98]',
    styleClass,
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
    <template v-if="size !== 'icon'">
      <span v-if="label">{{ label }}</span>
      <slot v-else />
    </template>
    <component
      v-if="iconRight && !loading"
      :is="iconRight"
      :size="size === 'sm' ? 14 : 16"
      class="shrink-0"
    />
  </button>
</template>
