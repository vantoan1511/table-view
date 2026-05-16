<script setup lang="ts">
import Checkbox from './Checkbox.vue';
import DropdownMenu from './DropdownMenu.vue';

import { useConnectionsStore } from '@/stores/connections';
import { useGridStore, type TableColumn } from '@/stores/grid';
import { DbType } from '@/types';
import { AlertCircle, Check, Edit2, Plus, Trash2, X } from 'lucide-vue-next';
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

const getSupportedDataTypes = (type: DbType) => {
  switch (type) {
    case DbType.POSTGRESQL:
      return [
        'integer',
        'bigint',
        'text',
        'varchar(255)',
        'boolean',
        'timestamp',
        'date',
        'numeric',
        'jsonb',
        'uuid'
      ];
    case DbType.MYSQL:
    case DbType.MARIADB:
      return [
        'int',
        'bigint',
        'varchar(255)',
        'text',
        'boolean',
        'datetime',
        'date',
        'decimal',
        'json'
      ];
    case DbType.SQLITE:
      return ['integer', 'text', 'real', 'blob', 'numeric'];
    case DbType.ORACLE:
      return ['NUMBER', 'VARCHAR2(255)', 'DATE', 'TIMESTAMP', 'CLOB', 'BLOB'];
    case DbType.SQLSERVER:
      return ['int', 'bigint', 'nvarchar(255)', 'text', 'bit', 'datetime2', 'date', 'decimal'];
    default:
      return ['text', 'varchar(255)', 'integer', 'boolean'];
  }
};

const typeOptions = computed(() =>
  getSupportedDataTypes(dbType.value).map((t) => ({ label: t, value: t }))
);

interface ColumnDef {
  id: string;
  name: string;
  dataType: string;
  nullable: boolean;
  isPrimaryKey: boolean;
  default: string | null;
  _editing?: boolean;
}

const tableName = ref('new_table');
const columns = ref<ColumnDef[]>([
  {
    id: crypto.randomUUID(),
    name: 'id',
    dataType: 'integer',
    nullable: false,
    isPrimaryKey: true,
    default: null,
    _editing: false
  }
]);

const addColumn = () => {
  columns.value.push({
    id: crypto.randomUUID(),
    name: `column_${columns.value.length + 1}`,
    dataType: 'varchar(255)',
    nullable: true,
    isPrimaryKey: false,
    default: null,
    _editing: true
  });
};

const removeColumn = (id: string) => {
  columns.value = columns.value.filter((c) => c.id !== id);
};

const editColumn = (col: ColumnDef) => {
  col._editing = true;
};

const saveColumn = (col: ColumnDef) => {
  col._editing = false;
};

const togglePrimaryKey = (col: ColumnDef) => {
  if (!col.isPrimaryKey) {
    // Only one primary key for simplicity in UI for now, or allow composite?
    // Let's allow multiple but many DBs might need special handling.
    // For now let's just toggle.
    col.isPrimaryKey = true;
    col.nullable = false;
  } else {
    col.isPrimaryKey = false;
  }
};

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

        <div class="border-border overflow-hidden rounded-lg border">
          <table class="w-full border-collapse text-left">
            <thead>
              <tr
                class="bg-muted border-border text-text-secondary border-b text-[12px] font-medium"
              >
                <th class="w-[30%] px-4 py-2">Column Name</th>
                <th class="w-[25%] px-4 py-2">Data Type</th>
                <th class="w-[10%] px-4 py-2 text-center">PK</th>
                <th class="w-[10%] px-4 py-2 text-center">Null</th>
                <th class="w-[15%] px-4 py-2">Default</th>
                <th class="w-[10%] px-4 py-2 text-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="col in columns"
                :key="col.id"
                class="border-border/50 hover:bg-muted/50 border-b text-[13px] last:border-0"
              >
                <td class="px-4 py-2">
                  <input
                    v-if="col._editing"
                    v-model.trim="col.name"
                    type="text"
                    class="bg-surface border-primary/50 focus:border-primary w-full rounded border px-2 py-1 outline-none"
                    :class="{ 'border-danger! !focus:border-danger': !col.name.trim() }"
                  />
                  <span v-else class="text-text-primary">{{ col.name }}</span>
                </td>

                <td class="px-4 py-2">
                  <div v-if="col._editing" class="w-full">
                    <DropdownMenu
                      v-model="col.dataType"
                      :options="typeOptions"
                      class="w-full"
                      button-class="w-full justify-between !bg-surface"
                    />
                  </div>
                  <span v-else class="text-text-secondary">{{ col.dataType }}</span>
                </td>

                <td class="px-4 py-2 text-center">
                  <Checkbox
                    :model-value="col.isPrimaryKey"
                    @update:model-value="togglePrimaryKey(col)"
                  />
                </td>

                <td class="px-4 py-2 text-center">
                  <Checkbox v-model="col.nullable" :disabled="col.isPrimaryKey" />
                </td>

                <td class="px-4 py-2">
                  <input
                    v-if="col._editing"
                    v-model="col.default"
                    type="text"
                    placeholder="—"
                    class="bg-surface border-primary/50 focus:border-primary w-full rounded border px-2 py-1 outline-none"
                  />
                  <span v-else class="text-text-secondary">{{ col.default || '—' }}</span>
                </td>

                <td class="px-4 py-2 text-right">
                  <div class="flex items-center justify-end gap-2">
                    <button
                      v-if="col._editing"
                      @click="saveColumn(col)"
                      class="text-success hover:text-success/80 transition-colors"
                      title="Save"
                    >
                      <Check :size="14" />
                    </button>
                    <button
                      v-else
                      @click="editColumn(col)"
                      class="text-text-tertiary hover:text-primary transition-colors"
                      title="Edit"
                    >
                      <Edit2 :size="14" />
                    </button>

                    <button
                      @click="removeColumn(col.id)"
                      class="text-text-tertiary hover:text-danger transition-colors"
                      title="Delete"
                    >
                      <Trash2 :size="14" />
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="mt-4">
          <button
            @click="addColumn"
            class="border-primary/30 text-primary hover:bg-primary/10 flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-[13px] font-medium transition-colors"
          >
            <Plus :size="14" />
            <span>Add Column</span>
          </button>
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
