<script setup lang="ts">
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Select from 'primevue/select';

import { DbType } from '@/types';

import { Check, Edit2, Plus, Trash2 } from 'lucide-vue-next';
import { computed } from 'vue';

export interface ConstraintDef {
  id: string;
  name: string;
  constraintType: string; // PRIMARY KEY, FOREIGN KEY, UNIQUE, CHECK
  definition: string;
  _originalName?: string;
  _isNew?: boolean;
  _deleted?: boolean;
  _editing?: boolean;
}

const props = withDefaults(
  defineProps<{
    modelValue: ConstraintDef[];
    dbType: DbType;
    mode: 'create' | 'alter';
  }>(),
  {
    mode: 'create'
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: ConstraintDef[]): void;
}>();

const constraintTypeOptions = [
  { label: 'PRIMARY KEY', value: 'PRIMARY KEY' },
  { label: 'FOREIGN KEY', value: 'FOREIGN KEY' },
  { label: 'UNIQUE', value: 'UNIQUE' },
  { label: 'CHECK', value: 'CHECK' }
];

const visibleConstraints = computed(() => props.modelValue.filter((c) => !c._deleted));

const addConstraint = () => {
  const isAlter = props.mode === 'alter';
  let name = '';
  let index = 1;
  while (props.modelValue.some((c) => c.name === `new_constraint_${index}`)) {
    index++;
  }
  name = `new_constraint_${index}`;

  const newConstraint: ConstraintDef = {
    id: crypto.randomUUID(),
    name,
    constraintType: 'PRIMARY KEY',
    definition: '',
    _isNew: isAlter,
    _editing: true
  };
  emit('update:modelValue', [...props.modelValue, newConstraint]);
};

const removeConstraint = (id: string) => {
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
    .filter((c): c is ConstraintDef => c !== null);
  emit('update:modelValue', updated);
};

const editConstraint = (c: ConstraintDef) => {
  c._editing = true;
};

const saveConstraint = (c: ConstraintDef) => {
  c._editing = false;
};
</script>

<template>
  <div>
    <div class="border-border overflow-hidden rounded-lg border">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr class="bg-muted border-border text-text-secondary border-b text-[12px] font-medium">
            <th class="w-[30%] px-4 py-2">Constraint Name</th>
            <th class="w-[20%] px-4 py-2">Type</th>
            <th class="w-[40%] px-4 py-2">Definition</th>
            <th class="w-[10%] px-4 py-2 text-right">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="c in visibleConstraints"
            :key="c.id"
            class="border-border/50 hover:bg-muted/50 border-b text-[13px] last:border-0"
          >
            <!-- Constraint Name -->
            <td class="px-4 py-2">
              <InputText
                v-if="c._editing && (mode === 'create' || c._isNew)"
                v-model.trim="c.name"
                size="small"
                class="w-full"
                :invalid="!c.name.trim()"
              />
              <span v-else class="text-text-primary">{{ c.name }}</span>
            </td>

            <!-- Constraint Type -->
            <td class="px-4 py-2">
              <Select
                v-if="c._editing && (mode === 'create' || c._isNew)"
                :model-value="c.constraintType"
                @update:model-value="(val) => (c.constraintType = String(val))"
                :options="constraintTypeOptions"
                optionLabel="label"
                optionValue="value"
                size="small"
                class="w-full text-xs"
              />
              <span
                v-else
                class="bg-muted text-text-secondary rounded px-2 py-0.5 text-[11px] font-semibold"
                >{{ c.constraintType }}</span
              >
            </td>

            <!-- Definition -->
            <td class="px-4 py-2">
              <InputText
                v-if="c._editing && (mode === 'create' || c._isNew)"
                v-model="c.definition"
                placeholder="e.g. (id) or (user_id) REFERENCES users(id)"
                size="small"
                class="w-full font-mono text-[12px]"
                :invalid="!c.definition.trim()"
              />
              <span v-else class="text-text-secondary font-mono text-[12px]">{{
                c.definition || '—'
              }}</span>
            </td>

            <!-- Actions -->
            <td class="px-4 py-2 text-right">
              <div class="flex items-center justify-end gap-1">
                <Button
                  v-if="c._editing"
                  v-tooltip.top="'Save'"
                  size="small"
                  variant="text"
                  severity="success"
                  class="h-7! w-7! p-0!"
                  :disabled="!c.name.trim() || !c.definition.trim()"
                  @click="saveConstraint(c)"
                >
                  <template #icon>
                    <Check :size="14" />
                  </template>
                </Button>
                <Button
                  v-else-if="mode === 'create' || c._isNew"
                  v-tooltip.top="'Edit'"
                  size="small"
                  variant="text"
                  severity="secondary"
                  class="h-7! w-7! p-0!"
                  @click="editConstraint(c)"
                >
                  <template #icon>
                    <Edit2 :size="14" />
                  </template>
                </Button>

                <Button
                  v-tooltip.top="'Delete'"
                  size="small"
                  variant="text"
                  severity="danger"
                  class="h-7! w-7! p-0!"
                  @click="removeConstraint(c.id)"
                >
                  <template #icon>
                    <Trash2 :size="14" />
                  </template>
                </Button>
              </div>
            </td>
          </tr>
          <tr v-if="visibleConstraints.length === 0">
            <td colspan="4" class="text-text-tertiary px-4 py-6 text-center text-[13px]">
              No constraints found.
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Add Constraint Button -->
    <div class="mt-4">
      <Button size="small" variant="outlined" severity="primary" @click="addConstraint">
        <Plus :size="14" />
        <span>Add Constraint</span>
      </Button>
    </div>
  </div>
</template>
