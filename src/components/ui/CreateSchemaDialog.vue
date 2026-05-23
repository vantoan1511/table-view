<script setup lang="ts">
import InputDialog from './InputDialog.vue';

import { useGridStore } from '@/stores/grid';

const props = defineProps<{
  connectionId: string;
  db?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gridStore = useGridStore();

const handleCreate = async (name: string) => {
  try {
    await gridStore.createSchema(props.connectionId, name, props.db);
    emit('close');
  } catch (err) {
    // Error handled by store/toast
  }
};
</script>

<template>
  <InputDialog
    title="Create Schema"
    label="Schema Name"
    placeholder="Enter schema name..."
    confirm-label="Create Schema"
    footer-note="Note: In MySQL, this will create a new database."
    @submit="handleCreate"
    @close="$emit('close')"
  />
</template>
