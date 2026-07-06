<script setup lang="ts">
import InputDialog from './InputDialog.vue';

import { useGridStore } from '@/stores/grid';

const props = defineProps<{
  connectionId: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gridStore = useGridStore();

const handleCreate = async (name: string) => {
  try {
    await gridStore.createDatabase(props.connectionId, name);
    emit('close');
  } catch {
    // Error handled by store/toast
  }
};
</script>

<template>
  <InputDialog
    title="Create Database"
    label="Database Name"
    placeholder="Enter database name..."
    confirm-label="Create Database"
    @submit="handleCreate"
    @close="$emit('close')"
  />
</template>
