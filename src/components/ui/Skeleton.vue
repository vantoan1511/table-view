<script setup lang="ts">
interface Props {
  width?: string;
  height?: string;
  rounded?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  animation?: 'pulse' | 'shimmer' | 'none';
}

withDefaults(defineProps<Props>(), {
  width: '100%',
  height: '1rem',
  rounded: 'md',
  animation: 'shimmer'
});
</script>

<template>
  <div
    class="skeleton bg-muted relative overflow-hidden"
    :class="[
      `rounded-${rounded}`,
      animation === 'pulse' ? 'animate-pulse' : '',
      animation === 'shimmer' ? 'shimmer-effect' : ''
    ]"
    :style="{ width, height }"
  ></div>
</template>

<style scoped>
.shimmer-effect::after {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  transform: translateX(-100%);
  background-image: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0,
    rgba(255, 255, 255, 0.05) 20%,
    rgba(255, 255, 255, 0.1) 60%,
    rgba(255, 255, 255, 0)
  );
  animation: shimmer 2s infinite;
}

@keyframes shimmer {
  100% {
    transform: translateX(100%);
  }
}

/* Dark mode adjustments */
:global(.dark) .shimmer-effect::after {
  background-image: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0,
    rgba(255, 255, 255, 0.03) 20%,
    rgba(255, 255, 255, 0.08) 60%,
    rgba(255, 255, 255, 0)
  );
}
</style>
