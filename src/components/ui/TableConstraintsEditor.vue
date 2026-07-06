<script setup lang="ts">
import DropdownMenu from './DropdownMenu.vue';

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
              <input
                v-if="c._editing && (mode === 'create' || c._isNew)"
                v-model.trim="c.name"
                type="text"
                class="bg-surface border-primary/50 focus:border-primary text-text-primary w-full rounded border px-2 py-1 outline-none"
                :class="{ 'border-danger! !focus:border-danger': !c.name.trim() }"
              />
              <span v-else class="text-text-primary">{{ c.name }}</span>
            </td>

            <!-- Constraint Type -->
            <td class="px-4 py-2">
              <DropdownMenu
                v-if="c._editing && (mode === 'create' || c._isNew)"
                :model-value="c.constraintType"
                @update:model-value="(val) => (c.constraintType = String(val))"
                :options="constraintTypeOptions"
                class="w-full"
                button-class="w-full justify-between !bg-surface text-text-primary text-[13px]"
              />
              <span
                v-else
                class="bg-muted text-text-secondary rounded px-2 py-0.5 text-[11px] font-semibold"
                >{{ c.constraintType }}</span
              >
            </td>

            <!-- Definition -->
            <td class="px-4 py-2">
              <input
                v-if="c._editing && (mode === 'create' || c._isNew)"
                v-model="c.definition"
                type="text"
                placeholder="e.g. (id) or (user_id) REFERENCES users(id)"
                class="bg-surface border-primary/50 focus:border-primary text-text-primary w-full rounded border px-2 py-1 text-[12px] outline-none"
                :class="{ 'border-danger! !focus:border-danger': !c.definition.trim() }"
              />
              <span v-else class="text-text-secondary font-mono text-[12px]">{{
                c.definition || '—'
              }}</span>
            </td>

            <!-- Actions -->
            <td class="px-4 py-2 text-right">
              <div class="flex items-center justify-end gap-2">
                <button
                  v-if="c._editing"
                  @click="saveConstraint(c)"
                  v-tooltip.top="'Save'"
                  class="text-success hover:text-success/80 cursor-pointer transition-colors"
                  :disabled="!c.name.trim() || !c.definition.trim()"
                >
                  <Check :size="14" />
                </button>
                <button
                  v-else-if="mode === 'create' || c._isNew"
                  @click="editConstraint(c)"
                  v-tooltip.top="'Edit'"
                  class="text-text-tertiary hover:text-primary cursor-pointer transition-colors"
                >
                  <Edit2 :size="14" />
                </button>

                <button
                  @click="removeConstraint(c.id)"
                  v-tooltip.top="'Delete'"
                  class="text-text-tertiary hover:text-danger cursor-pointer transition-colors"
                >
                  <Trash2 :size="14" />
                </button>
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
      <button
        @click="addConstraint"
        class="border-primary/30 text-primary hover:bg-primary/10 flex cursor-pointer items-center gap-1.5 rounded-lg border px-3 py-1.5 text-[13px] font-medium transition-colors"
      >
        <Plus :size="14" />
        <span>Add Constraint</span>
      </button>
    </div>
  </div>
</template>
