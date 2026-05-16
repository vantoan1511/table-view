<script setup lang="ts">
import { useGridStore } from '@/stores/grid';
import { useToastStore } from '@/stores/toast';
import { Check, Edit2, Plus, Trash2, X } from 'lucide-vue-next';
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

interface ColumnDef {
  id: string;
  name: string;
  dataType: string;
  nullable: boolean;
  default: string | null;
  _originalName?: string;
  _isNew?: boolean;
  _deleted?: boolean;
  _editing?: boolean;
}

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

const visibleColumns = computed(() => columns.value.filter((c) => !c._deleted));

const addColumn = () => {
  columns.value.push({
    id: crypto.randomUUID(),
    name: 'new_column',
    dataType: 'varchar(255)',
    nullable: true,
    default: null,
    _isNew: true,
    _editing: true
  });
};

const removeColumn = (col: ColumnDef) => {
  col._deleted = true;
};

const editColumn = (col: ColumnDef) => {
  col._editing = true;
};

const saveColumn = (col: ColumnDef) => {
  col._editing = false;
};

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
          <div class="border-border overflow-hidden rounded-lg border">
            <table class="w-full border-collapse text-left">
              <thead>
                <tr
                  class="bg-muted border-border text-text-secondary border-b text-[12px] font-medium"
                >
                  <th class="w-[30%] px-4 py-2">Column Name</th>
                  <th class="w-[25%] px-4 py-2">Data Type</th>
                  <th class="w-[15%] px-4 py-2">Nullable</th>
                  <th class="w-[20%] px-4 py-2">Default</th>
                  <th class="w-[10%] px-4 py-2 text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="col in visibleColumns"
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
                      v-if="col._editing && col._isNew"
                      v-model="col.dataType"
                      type="text"
                      class="bg-surface border-primary/50 focus:border-primary w-full rounded border px-2 py-1 outline-none"
                    />
                    <span v-else class="text-text-secondary">{{ col.dataType }}</span>
                  </td>

                  <td class="px-4 py-2">
                    <select
                      v-if="col._editing && col._isNew"
                      v-model="col.nullable"
                      class="bg-surface border-primary/50 focus:border-primary w-full rounded border px-2 py-1 outline-none"
                    >
                      <option :value="true">YES</option>
                      <option :value="false">NO</option>
                    </select>
                    <span v-else class="text-text-secondary">{{
                      col.nullable ? 'YES' : 'NO'
                    }}</span>
                  </td>

                  <td class="px-4 py-2">
                    <input
                      v-if="col._editing && col._isNew"
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
                        @click="removeColumn(col)"
                        class="text-text-tertiary hover:text-danger transition-colors"
                        title="Delete"
                      >
                        <Trash2 :size="14" />
                      </button>
                    </div>
                  </td>
                </tr>
                <tr v-if="visibleColumns.length === 0">
                  <td colspan="5" class="text-text-tertiary px-4 py-6 text-center text-[13px]">
                    No columns found.
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
