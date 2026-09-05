<script setup lang="ts">
import TableColumnsEditor, { type ColumnDef } from './TableColumnsEditor.vue';
import TableConstraintsEditor, { type ConstraintDef } from './TableConstraintsEditor.vue';

import { useConnectionsStore } from '@/stores/connections';
import { useGridStore } from '@/stores/grid';
import { useToastStore } from '@/stores/toast';
import { DbType } from '@/types';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
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
const constraints = ref<ConstraintDef[]>([]);
const loading = ref(true);
const activeTab = ref('Columns');
const tabs = ['Columns', 'Constraints', 'Indexes', 'Options', 'Comment'];

onMounted(async () => {
  try {
    const [cols, cons] = await Promise.all([
      gridStore.getTableColumns(props.tableName),
      gridStore.getTableConstraints(props.tableName)
    ]);

    columns.value = cols.map((c: any) => ({
      id: crypto.randomUUID(),
      name: c.name,
      dataType: c.dataType,
      nullable: c.nullable,
      isPrimaryKey: c.isPrimaryKey || false,
      default: c.default,
      _originalName: c.name,
      _fkStr: c.foreignKey ? `${c.foreignKey.targetTable}.${c.foreignKey.targetColumn}` : '',
      _isNew: false,
      _deleted: false,
      _editing: false
    }));

    constraints.value = cons.map((c: any) => ({
      id: crypto.randomUUID(),
      name: c.name,
      constraintType: c.constraintType,
      definition: c.definition,
      _originalName: c.name,
      _isNew: false,
      _deleted: false,
      _editing: false
    }));
  } catch (err: any) {
    console.error('Failed to fetch table structure:', err);
    toastStore.addToast({
      severity: 'error',
      title: 'Failed to Load Table Structure',
      message: err.message || 'Could not retrieve table columns or constraints.'
    });
  } finally {
    loading.value = false;
  }
});

const parseFkStr = (fkStr?: string) => {
  if (!fkStr || !fkStr.trim()) return undefined;
  const lastDot = fkStr.lastIndexOf('.');
  if (lastDot !== -1) {
    const targetTable = fkStr.substring(0, lastDot).trim();
    const targetColumn = fkStr.substring(lastDot + 1).trim();
    if (targetTable && targetColumn) {
      return { targetTable, targetColumn };
    }
  }
  return undefined;
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
        default: col.default,
        foreignKey: parseFkStr(col._fkStr)
      });
    } else if (col.name !== col._originalName) {
      operations.push({
        type: 'RENAME_COLUMN',
        oldName: col._originalName,
        newName: col.name
      });
    }
  }

  for (const con of constraints.value) {
    if (con._deleted) {
      if (!con._isNew) {
        operations.push({
          type: 'DROP_CONSTRAINT',
          constraintName: con._originalName
        });
      }
      continue;
    }

    if (con._isNew) {
      operations.push({
        type: 'ADD_CONSTRAINT',
        name: con.name,
        definition: con.definition
      });
    }
  }

  emit('apply', operations);
};
</script>

<template>
  <Dialog
    visible
    modal
    :header="`Alter Table: ${tableName}`"
    :style="{ width: '52rem' }"
    :closable="true"
    @update:visible="
      (val) => {
        if (!val) emit('close');
      }
    "
  >
    <!-- Tabs -->
    <div class="border-border hide-scrollbar -mx-5 -mt-2 mb-4 flex border-b px-5">
      <button
        v-for="tab in tabs"
        :key="tab"
        @click="activeTab = tab"
        class="border-b-2 px-4 py-2.5 text-[13px] font-medium whitespace-nowrap transition-colors"
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
    <div class="max-h-[60vh] min-h-75 overflow-y-auto">
      <div v-if="loading" class="flex justify-center py-10">
        <div
          class="border-primary h-6 w-6 animate-spin rounded-full border-2 border-t-transparent"
        ></div>
      </div>

      <div v-else-if="activeTab === 'Columns'">
        <TableColumnsEditor v-model="columns" :db-type="dbType" mode="alter" />
      </div>

      <div v-else-if="activeTab === 'Constraints'">
        <TableConstraintsEditor v-model="constraints" :db-type="dbType" mode="alter" />
      </div>

      <div v-else class="text-text-tertiary flex h-40 items-center justify-center text-[13px]">
        {{ activeTab }} configuration is not available yet.
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2 pt-2">
        <Button variant="outlined" severity="secondary" @click="$emit('close')">Cancel</Button>
        <Button severity="primary" :disabled="loading" @click="applyChanges">
          Apply Changes
        </Button>
      </div>
    </template>
  </Dialog>
</template>
