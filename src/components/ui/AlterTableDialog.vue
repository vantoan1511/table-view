<script setup lang="ts">
import TableColumnsEditor, { type ColumnDef } from './TableColumnsEditor.vue';

import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useToastStore } from '@/stores/toast';
import { DbType } from '@/types';
import { X } from 'lucide-vue-next';
import { computed, onMounted, ref } from 'vue';

const props = defineProps<{
  tableName: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'apply', operations: any[]): void;
}>();

const gridStore = useGridStore();
const toastStore = useToastStore();
const connectionsStore = useConnectionsStore();

const dbType = computed(() => connectionsStore.activeConnection?.type ?? DbType.POSTGRESQL);

const columns = ref<ColumnDef[]>([]);
const loading = ref(true);
const activeTab = ref('Columns');
const tabs = ['Columns', 'Constraints', 'Indexes', 'Options', 'Comment'];

onMounted(async () => {
  try {
    const cols = await gridStore.getTableColumns(props.tableName);
    columns.value = cols.map((c: any) => ({
      id: crypto.randomUUID(),
      name: c.name,
      dataType: c.dataType,
      nullable: c.nullable,
      isPrimaryKey: c.isPrimaryKey || false,
      default: c.default,
      _originalName: c.name,
      _isNew: false,
      _deleted: false,
      _editing: false
    }));
  } catch (err: any) {
    console.error('Failed to fetch columns:', err);
    toastStore.addToast({
      severity: 'error',
      title: 'Failed to Load Columns',
      message: err.message || 'Could not retrieve the table structure.'
    });
  } finally {
    loading.value = false;
  }
});

const applyChanges = async () => {
  const operations: any[] = [];

  for (const col of columns.value) {
    if (col._deleted) {
      if (!col._isNew) {
        operations.push({ type: 'DROP_COLUMN', name: col._originalName });
      }
      continue;
    }

    if (col._isNew) {
      operations.push({
        type: 'ADD_COLUMN',
        name: col.name,
        dataType: col.dataType,
        nullable: col.nullable,
        default: col.default
      });
    } else if (col.name !== col._originalName) {
      operations.push({
        type: 'RENAME_COLUMN',
        oldName: col._originalName,
        newName: col.name
      });
    }
  }

  emit('apply', operations);
};
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
    <div
      class="bg-surface border-border flex max-h-[85vh] w-175 flex-col overflow-hidden rounded-xl border shadow-2xl"
    >
      <!-- Header -->
      <div class="border-border flex items-center justify-between border-b px-5 py-4">
        <h3 class="text-text-primary text-base font-semibold">Alter Table: {{ tableName }}</h3>
        <button
          @click="$emit('close')"
          class="text-text-tertiary hover:text-text-primary transition-colors"
        >
          <X :size="18" />
        </button>
      </div>

      <!-- Tabs -->
      <div class="border-border hide-scrollbar flex overflow-x-auto border-b px-5">
        <button
          v-for="tab in tabs"
          :key="tab"
          @click="activeTab = tab"
          class="border-b-2 px-4 py-3 text-[13px] font-medium whitespace-nowrap transition-colors"
          :class="
            activeTab === tab
              ? 'border-primary text-primary'
              : 'text-text-tertiary hover:text-text-secondary border-transparent'
          "
        >
          {{ tab }}
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto p-5">
        <div v-if="loading" class="flex justify-center py-10">
          <div
            class="border-primary h-6 w-6 animate-spin rounded-full border-2 border-t-transparent"
          ></div>
        </div>

        <div v-else-if="activeTab === 'Columns'">
          <TableColumnsEditor v-model="columns" :db-type="dbType" mode="alter" />
        </div>

        <div v-else class="text-text-tertiary flex h-40 items-center justify-center text-[13px]">
          {{ activeTab }} configuration is not available yet.
        </div>
      </div>

      <!-- Footer -->
      <div class="border-border bg-muted/30 flex items-center justify-end gap-3 border-t px-5 py-4">
        <button
          @click="$emit('close')"
          class="text-text-secondary hover:text-text-primary border-border bg-surface hover:bg-hover rounded-lg border px-4 py-2 text-[13px] font-medium transition-colors"
        >
          Cancel
        </button>
        <button
          @click="applyChanges"
          class="bg-primary hover:bg-primary-hover rounded-lg px-4 py-2 text-[13px] font-medium text-white shadow-sm transition-colors"
          :disabled="loading"
        >
          Apply Changes
        </button>
      </div>
    </div>
  </div>
</template>
