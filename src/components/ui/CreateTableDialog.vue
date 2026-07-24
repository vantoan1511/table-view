<script setup lang="ts">
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import TableColumnsEditor, { type ColumnDef } from './TableColumnsEditor.vue';
import { useConnectionsStore } from '@/stores/connections';
import { useGridStore, type TableColumn } from '@/stores/grid';
import { DbType } from '@/types';
import { AlertCircle } from 'lucide-vue-next';
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
          default: c.default || undefined,
          foreignKey: parseFkStr(c._fkStr)
        })
      ),
      props.connectionId,
      props.schema,
      props.db
    );
    emit('close');
  } catch {
    // Error handled by store/toast
  }
};
</script>

<template>
  <Dialog
    visible
    modal
    header="Create Table"
    :style="{ width: '46rem' }"
    :closable="true"
    @update:visible="
      (val) => {
        if (!val) emit('close');
      }
    "
  >
    <div class="py-2">
      <div class="mb-6">
        <div class="mb-1.5 flex items-center justify-between">
          <label class="text-text-secondary block text-[13px] font-medium">Table Name</label>
          <span
            v-if="!tableName.trim()"
            class="text-danger flex items-center gap-1 text-[11px] font-medium"
          >
            <AlertCircle :size="12" />
            Required
          </span>
        </div>
        <InputText
          v-model.trim="tableName"
          type="text"
          placeholder="Enter table name..."
          class="w-full"
          :invalid="!tableName.trim()"
        />
      </div>

      <TableColumnsEditor v-model="columns" :db-type="dbType" mode="create" />
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2 pt-2">
        <Button variant="outlined" severity="secondary" @click="$emit('close')"> Cancel </Button>
        <Button
          severity="primary"
          :disabled="
            !tableName.trim() || columns.length === 0 || columns.some((c) => !c.name.trim())
          "
          @click="handleCreate"
        >
          Create Table
        </Button>
      </div>
    </template>
  </Dialog>
</template>
