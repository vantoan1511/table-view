<script setup lang="ts">
import Skeleton from 'primevue/skeleton';

interface Props {
  width?: string;
  height?: string;
  borderRadius?: string;
  shape?: 'rectangle' | 'circle';
  animation?: 'wave' | 'pulse' | 'none';
}

withDefaults(defineProps<Props>(), {
  width: '100%',
  height: '1rem',
  shape: 'rectangle',
  animation: 'wave'
});
</script>

<template>
  <Skeleton
    :width="width"
    :height="height"
    :shape="shape"
    :animation="animation"
    :border-radius="borderRadius"
  />
</template>

