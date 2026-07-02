<script setup lang="ts">
import { useGridStore } from '@/stores/grid';
import { useSchemaStore } from '@/stores/schema';
import { computed, watch } from 'vue';

const gridStore = useGridStore();
const schemaStore = useSchemaStore();

const activeTable = computed(() => gridStore.activeTableName);
const activeTableSchema = computed(() => gridStore.activeTableSchema);
const activeConnectionId = computed(() => gridStore.activeConnectionId);
const activeDbName = computed(() => gridStore.activeDbName);

// Watch active table and connection to trigger index loading
watch(
  [activeConnectionId, activeTable],
  async ([connId, table]) => {
    if (connId && table) {
      await schemaStore.loadTableIndexes(connId, table);
    }
  },
  { immediate: true }
);

// Get the indexes for the current connection and table
const indexes = computed(() => {
  const connId = activeConnectionId.value;
  const table = activeTable.value;
  if (!connId || !table) return [];
  return schemaStore.tableIndexes[connId]?.[table] ?? [];
});
</script>

<template>
  <div class="bg-surface custom-scrollbar flex-1 overflow-auto p-4 text-[12px]">
    <div v-if="!activeTable" class="text-text-tertiary italic">
      Select a table to view its indexes.
    </div>
    <div v-else class="flex flex-col gap-5">
      <!-- Table Header Info -->
      <div>
        <div class="text-text-primary text-[13px] font-semibold">Indexes for {{ activeTable }}</div>
        <div class="text-text-tertiary mt-0.5 text-[10px]">
          Schema: {{ activeTableSchema }} <span v-if="activeDbName">| DB: {{ activeDbName }}</span>
        </div>
      </div>

      <!-- Indexes list -->
      <div v-if="indexes.length === 0" class="text-text-tertiary">No indexes found.</div>
      <div v-else class="flex flex-col gap-2">
        <div
          v-for="idx in indexes"
          :key="idx.name"
          class="border-border bg-surface-alt hover:border-border/60 flex flex-col gap-1 rounded-lg border p-2.5 transition-colors"
        >
          <div class="flex items-center justify-between">
            <span class="text-text-primary font-mono font-semibold">{{ idx.name }}</span>
            <div class="flex items-center gap-1.5">
              <span
                v-if="idx.isPrimaryKey"
                class="rounded border border-amber-500/20 bg-amber-500/10 px-1.5 py-0.5 text-[9px] font-bold text-amber-500"
                title="Primary Key"
                >PK</span
              >
              <span
                v-else-if="idx.isUnique"
                class="rounded border border-emerald-500/20 bg-emerald-500/10 px-1.5 py-0.5 text-[9px] font-bold text-emerald-500"
                title="Unique Constraint"
                >UNIQ</span
              >
            </div>
          </div>
          <div class="text-text-secondary flex justify-between text-[11px]">
            <span
              >Type: <span class="text-primary font-mono">{{ idx.indexType || 'N/A' }}</span></span
            >
            <span class="text-text-tertiary max-w-[60%] truncate" :title="idx.columns.join(', ')">
              Columns:
              <span class="text-text-primary font-mono font-medium">{{
                idx.columns.join(', ')
              }}</span>
            </span>
          </div>

          <!-- Optional DDL details if we have them, styled cleanly -->
          <div
            v-if="idx.ddl"
            class="text-text-tertiary border-border/10 mt-1.5 border-t pt-1.5 font-mono text-[10px] leading-normal break-all whitespace-pre-wrap"
          >
            {{ idx.ddl }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
