<script setup lang="ts">
import TableColumnsEditor, { type ColumnDef } from './TableColumnsEditor.vue';

import { useConnectionsStore } from '@/stores/connections';
import { useGridStore, type TableColumn } from '@/stores/grid';
import { DbType } from '@/types';
import { AlertCircle, X } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const props = defineProps<{
  connectionId: string;
  schema: string;
  db?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gridStore = useGridStore();
const connectionsStore = useConnectionsStore();

const dbType = computed(() => connectionsStore.activeConnection?.type ?? DbType.POSTGRESQL);

const tableName = ref('new_table');
const columns = ref<ColumnDef[]>([
  {
    id: crypto.randomUUID(),
    name: 'id',
    dataType: dbType.value === DbType.ORACLE ? 'NUMBER' : 'integer',
    nullable: false,
    isPrimaryKey: true,
    default: null,
    _editing: false
  }
]);

const handleCreate = async () => {
  if (!tableName.value.trim()) return;
  if (columns.value.length === 0) return;

  try {
    await gridStore.createTable(
      tableName.value,
      columns.value.map(
        (c): TableColumn => ({
          name: c.name,
          dataType: c.dataType,
          nullable: c.nullable,
          isPrimaryKey: c.isPrimaryKey,
          default: c.default || undefined
        })
      ),
      props.connectionId,
      props.schema,
      props.db
    );
    emit('close');
  } catch (err) {
    // Error handled by store/toast
  }
};
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
    <div
      class="bg-surface border-border flex max-h-[85vh] w-180 flex-col overflow-hidden rounded-xl border shadow-2xl"
    >
      <!-- Header -->
      <div class="border-border flex items-center justify-between border-b px-5 py-4">
        <h3 class="text-text-primary text-base font-semibold">Create Table</h3>
        <button
          @click="$emit('close')"
          class="text-text-tertiary hover:text-text-primary transition-colors"
        >
          <X :size="18" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto p-5">
        <div class="mb-6">
          <div class="mb-1.5 flex items-center justify-between">
            <label class="text-text-secondary block text-[13px] font-medium">Table Name</label>
            <span
              v-if="!tableName.trim()"
              class="text-danger animate-fade-in-scale flex items-center gap-1 text-[11px] font-medium"
            >
              <AlertCircle :size="12" />
              Required
            </span>
          </div>
          <input
            v-model.trim="tableName"
            type="text"
            placeholder="Enter table name..."
            class="bg-muted border-border text-text-primary focus:border-primary w-full rounded-lg border px-4 py-2 text-[14px] transition-colors outline-none"
            :class="{ 'border-danger! !focus:border-danger': !tableName.trim() }"
          />
        </div>

        <TableColumnsEditor v-model="columns" :db-type="dbType" mode="create" />
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
          @click="handleCreate"
          class="bg-primary hover:bg-primary-hover rounded-lg px-4 py-2 text-[13px] font-medium text-white shadow-sm transition-colors"
          :disabled="
            !tableName.trim() || columns.length === 0 || columns.some((c) => !c.name.trim())
          "
        >
          Create Table
        </button>
      </div>
    </div>
  </div>
</template>
