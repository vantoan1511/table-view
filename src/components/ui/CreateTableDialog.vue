<script setup lang="ts">
import { useGridStore } from '@/stores/grid';
import { Check, Edit2, Key, Plus, Trash2, X } from 'lucide-vue-next';
import { ref } from 'vue';

const props = defineProps<{
  connectionId: string;
  schema: string;
  db?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gridStore = useGridStore();

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
      columns.value.map((c) => ({
        name: c.name,
        dataType: c.dataType,
        nullable: c.nullable,
        isPrimaryKey: c.isPrimaryKey,
        default: c.default
      })),
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
          <label class="text-text-secondary mb-1.5 block text-[13px] font-medium">Table Name</label>
          <input
            v-model="tableName"
            type="text"
            placeholder="Enter table name..."
            class="bg-muted border-border text-text-primary focus:border-primary w-full rounded-lg border px-4 py-2 text-[14px] transition-colors outline-none"
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
                    v-model="col.name"
                    type="text"
                    class="bg-surface border-primary/50 focus:border-primary w-full rounded border px-2 py-1 outline-none"
                  />
                  <span v-else class="text-text-primary">{{ col.name }}</span>
                </td>

                <td class="px-4 py-2">
                  <input
                    v-if="col._editing"
                    v-model="col.dataType"
                    type="text"
                    class="bg-surface border-primary/50 focus:border-primary w-full rounded border px-2 py-1 outline-none"
                  />
                  <span v-else class="text-text-secondary">{{ col.dataType }}</span>
                </td>

                <td class="px-4 py-2 text-center">
                  <button
                    @click="togglePrimaryKey(col)"
                    class="transition-colors"
                    :class="
                      col.isPrimaryKey
                        ? 'text-primary'
                        : 'text-text-tertiary hover:text-text-secondary'
                    "
                  >
                    <Key :size="14" />
                  </button>
                </td>

                <td class="px-4 py-2 text-center">
                  <input
                    v-model="col.nullable"
                    type="checkbox"
                    :disabled="col.isPrimaryKey"
                    class="accent-primary h-3.5 w-3.5 rounded border-gray-300"
                  />
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
          :disabled="!tableName.trim() || columns.length === 0"
        >
          Create Table
        </button>
      </div>
    </div>
  </div>
</template>
