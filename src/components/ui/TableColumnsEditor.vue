<script setup lang="ts">
import Checkbox from './Checkbox.vue';
import DropdownMenu from './DropdownMenu.vue';
import Tooltip from './Tooltip.vue';

import { DbType } from '@/types';

import { Check, Edit2, Plus, Trash2 } from 'lucide-vue-next';
import { computed } from 'vue';

export interface ColumnDef {
  id: string;
  name: string;
  dataType: string;
  nullable: boolean;
  isPrimaryKey: boolean;
  default: string | null;
  _fkStr?: string;
  _originalName?: string;
  _isNew?: boolean;
  _deleted?: boolean;
  _editing?: boolean;
}

const props = withDefaults(
  defineProps<{
    modelValue: ColumnDef[];
    dbType: DbType;
    mode: 'create' | 'alter';
  }>(),
  {
    mode: 'create'
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: ColumnDef[]): void;
}>();

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
  getSupportedDataTypes(props.dbType).map((t) => {
    const base = t.split('(')[0] || t;
    return { label: base, value: base };
  })
);

const isVarcharType = (dataTypeStr: string): boolean => {
  const lower = dataTypeStr.toLowerCase();
  return ['varchar', 'varchar2', 'nvarchar', 'char', 'nchar'].some((t) => lower.includes(t));
};

const isNumericType = (dataTypeStr: string): boolean => {
  const lower = dataTypeStr.toLowerCase();
  return ['numeric', 'decimal', 'number'].some((t) => lower.includes(t));
};

const parseDataType = (dataTypeStr: string) => {
  const normalized = dataTypeStr.trim();
  const match = normalized.match(/^([a-zA-Z0-9_\s]+)\s*(?:\(\s*(\d+)\s*(?:,\s*(\d+)\s*)?\))?$/i);
  if (!match) {
    return { baseType: normalized, length: '', precision: '', scale: '' };
  }
  const baseType = (match[1] || '').trim();
  const param1 = match[2] || '';
  const param2 = match[3] || '';

  const isVarchar = isVarcharType(baseType);
  const isNumeric = isNumericType(baseType);

  return {
    baseType,
    length: isVarchar ? param1 : '',
    precision: isNumeric ? param1 : '',
    scale: isNumeric ? param2 : ''
  };
};

const updateDataType = (
  col: ColumnDef,
  parts: {
    baseType?: string;
    length?: string | number;
    precision?: string | number;
    scale?: string | number;
  }
) => {
  const current = parseDataType(col.dataType);
  const baseType = parts.baseType !== undefined ? parts.baseType : current.baseType;

  if (isVarcharType(baseType)) {
    let length = parts.length !== undefined ? parts.length : current.length;
    if (!length && parts.baseType !== undefined) {
      length = '255';
    }
    col.dataType = length ? `${baseType}(${length})` : baseType;
  } else if (isNumericType(baseType)) {
    let precision = parts.precision !== undefined ? parts.precision : current.precision;
    let scale = parts.scale !== undefined ? parts.scale : current.scale;
    if (!precision && !scale && parts.baseType !== undefined) {
      precision = '10';
      scale = '2';
    }
    if (precision && scale) {
      col.dataType = `${baseType}(${precision},${scale})`;
    } else if (precision) {
      col.dataType = `${baseType}(${precision})`;
    } else {
      col.dataType = baseType;
    }
  } else {
    col.dataType = baseType;
  }
};

const visibleColumns = computed(() => props.modelValue.filter((c) => !c._deleted));

const addColumn = () => {
  const isAlter = props.mode === 'alter';
  let name = '';
  if (isAlter) {
    let index = 1;
    while (props.modelValue.some((c) => c.name === `new_column${index}`)) {
      index++;
    }
    name = `new_column${index}`;
  } else {
    name = `column_${props.modelValue.length + 1}`;
  }

  const newCol: ColumnDef = {
    id: crypto.randomUUID(),
    name,
    dataType: props.dbType === DbType.ORACLE ? 'VARCHAR2(255)' : 'varchar(255)',
    nullable: true,
    isPrimaryKey: false,
    default: null,
    _fkStr: '',
    _isNew: isAlter,
    _editing: true
  };
  emit('update:modelValue', [...props.modelValue, newCol]);
};

const removeColumn = (id: string) => {
  const updated = props.modelValue
    .map((c) => {
      if (c.id === id) {
        if (props.mode === 'alter' && !c._isNew) {
          return { ...c, _deleted: true };
        }
        return null;
      }
      return c;
    })
    .filter((c): c is ColumnDef => c !== null);
  emit('update:modelValue', updated);
};

const editColumn = (col: ColumnDef) => {
  col._editing = true;
};

const saveColumn = (col: ColumnDef) => {
  col._editing = false;
};

const togglePrimaryKey = (col: ColumnDef) => {
  if (!col.isPrimaryKey) {
    col.isPrimaryKey = true;
    col.nullable = false;
  } else {
    col.isPrimaryKey = false;
  }
};
</script>

<template>
  <div>
    <div class="border-border overflow-hidden rounded-lg border">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr class="bg-muted border-border text-text-secondary border-b text-[12px] font-medium">
            <th class="w-[30%] px-4 py-2">Column Name</th>
            <th class="w-[20%] px-4 py-2">Data Type</th>
            <th v-if="mode === 'create'" class="w-[8%] px-4 py-2 text-center">PK</th>
            <th class="w-[12%] px-4 py-2 text-center">Nullable</th>
            <th class="w-[15%] px-4 py-2">Default</th>
            <th class="w-[15%] px-4 py-2">FK (table.col)</th>
            <th class="w-[10%] px-4 py-2 text-right">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="col in visibleColumns"
            :key="col.id"
            class="border-border/50 hover:bg-muted/50 border-b text-[13px] last:border-0"
          >
            <!-- Column Name -->
            <td class="px-4 py-2">
              <input
                v-if="col._editing"
                v-model.trim="col.name"
                type="text"
                class="bg-surface border-primary/50 focus:border-primary text-text-primary w-full rounded border px-2 py-1 outline-none"
                :class="{ 'border-danger! !focus:border-danger': !col.name.trim() }"
              />
              <span v-else class="text-text-primary">{{ col.name }}</span>
            </td>

            <!-- Data Type -->
            <td class="px-4 py-2">
              <div
                v-if="col._editing && (mode === 'create' || col._isNew)"
                class="flex flex-col gap-2"
              >
                <DropdownMenu
                  :model-value="parseDataType(col.dataType).baseType"
                  @update:model-value="
                    (val) => updateDataType(col, { baseType: val ? String(val) : '' })
                  "
                  :options="typeOptions"
                  class="w-full"
                  button-class="w-full justify-between !bg-surface text-text-primary text-[13px]"
                />

                <!-- Additional constraints based on datatype -->
                <div v-if="isVarcharType(col.dataType)" class="flex items-center gap-2">
                  <span class="text-text-secondary text-[11px] font-medium whitespace-nowrap"
                    >Length:</span
                  >
                  <input
                    type="number"
                    min="1"
                    :value="parseDataType(col.dataType).length"
                    @input="
                      (e) => updateDataType(col, { length: (e.target as HTMLInputElement).value })
                    "
                    class="bg-surface border-border focus:border-primary text-text-primary w-full rounded border px-2 py-0.5 text-center text-[12px] outline-none"
                  />
                </div>

                <div v-else-if="isNumericType(col.dataType)" class="flex flex-col gap-1">
                  <span class="text-text-secondary text-[11px] font-medium whitespace-nowrap"
                    >Precision, Scale:</span
                  >
                  <div class="flex w-full items-center gap-1">
                    <span class="relative flex-1">
                      <input
                        type="number"
                        min="1"
                        :value="parseDataType(col.dataType).precision"
                        @input="
                          (e) =>
                            updateDataType(col, { precision: (e.target as HTMLInputElement).value })
                        "
                        placeholder="10"
                        class="bg-surface border-border focus:border-primary text-text-primary w-full rounded border px-1 py-0.5 text-center text-[12px] outline-none"
                      />
                      <Tooltip text="Precision (total digits)" position="top" />
                    </span>
                    <span class="text-text-tertiary">,</span>
                    <span class="relative flex-1">
                      <input
                        type="number"
                        min="0"
                        :value="parseDataType(col.dataType).scale"
                        @input="
                          (e) =>
                            updateDataType(col, { scale: (e.target as HTMLInputElement).value })
                        "
                        placeholder="2"
                        class="bg-surface border-border focus:border-primary text-text-primary w-full rounded border px-1 py-0.5 text-center text-[12px] outline-none"
                      />
                      <Tooltip text="Scale (decimal digits)" position="top" />
                    </span>
                  </div>
                </div>
              </div>
              <span v-else class="text-text-secondary">{{ col.dataType }}</span>
            </td>

            <!-- Primary Key (Only in Create Mode) -->
            <td v-if="mode === 'create'" class="px-4 py-2 text-center">
              <Checkbox
                :model-value="col.isPrimaryKey"
                @update:model-value="togglePrimaryKey(col)"
              />
            </td>

            <!-- Nullable -->
            <td class="px-4 py-2 text-center">
              <Checkbox
                v-model="col.nullable"
                :disabled="mode === 'alter' ? !col._isNew || !col._editing : col.isPrimaryKey"
              />
            </td>

            <!-- Default Value -->
            <td class="px-4 py-2">
              <input
                v-if="col._editing && (mode === 'create' || col._isNew)"
                v-model="col.default"
                type="text"
                placeholder="—"
                class="bg-surface border-primary/50 focus:border-primary text-text-primary w-full rounded border px-2 py-1 outline-none"
              />
              <span v-else class="text-text-secondary">{{ col.default || '—' }}</span>
            </td>

            <!-- Foreign Key -->
            <td class="px-4 py-2">
              <input
                v-if="col._editing && (mode === 'create' || col._isNew)"
                v-model="col._fkStr"
                type="text"
                placeholder="[schema.]table.col"
                title="Format: table.column or schema.table.column"
                class="bg-surface border-primary/50 focus:border-primary text-text-primary w-full rounded border px-2 py-1 outline-none text-[12px]"
              />
              <span v-else class="text-text-secondary text-[12px]">{{ col._fkStr || '—' }}</span>
            </td>

            <!-- Actions -->
            <td class="px-4 py-2 text-right">
              <div class="flex items-center justify-end gap-2">
                <button
                  v-if="col._editing"
                  @click="saveColumn(col)"
                  class="text-success hover:text-success/80 cursor-pointer transition-colors"
                  :disabled="!col.name.trim()"
                >
                  <Check :size="14" />
                  <Tooltip text="Save" position="top" />
                </button>
                <button
                  v-else
                  @click="editColumn(col)"
                  class="text-text-tertiary hover:text-primary cursor-pointer transition-colors"
                >
                  <Edit2 :size="14" />
                  <Tooltip text="Edit" position="top" />
                </button>

                <button
                  @click="removeColumn(col.id)"
                  class="text-text-tertiary hover:text-danger cursor-pointer transition-colors"
                >
                  <Trash2 :size="14" />
                  <Tooltip text="Delete" position="top" />
                </button>
              </div>
            </td>
          </tr>
          <tr v-if="visibleColumns.length === 0">
            <td
              :colspan="mode === 'create' ? 6 : 5"
              class="text-text-tertiary px-4 py-6 text-center text-[13px]"
            >
              No columns found.
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Add Column Button -->
    <div class="mt-4">
      <button
        @click="addColumn"
        class="border-primary/30 text-primary hover:bg-primary/10 flex cursor-pointer items-center gap-1.5 rounded-lg border px-3 py-1.5 text-[13px] font-medium transition-colors"
      >
        <Plus :size="14" />
        <span>Add Column</span>
      </button>
    </div>
  </div>
</template>
