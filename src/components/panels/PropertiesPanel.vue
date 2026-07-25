<script setup lang="ts">
import { useGridStore } from '@/stores/grid';
import { useDiagramStore } from '@/stores/diagram';
import { useTabsStore } from '@/stores/tabs';
import { computed, watch } from 'vue';
import { ArrowRight, Link } from '@lucide/vue';

const gridStore = useGridStore();
const diagramStore = useDiagramStore();
const tabsStore = useTabsStore();

const columns = computed(() => gridStore.columns);
const activeTable = computed(() => gridStore.activeTableName);
const activeTableSchema = computed(() => gridStore.activeTableSchema);
const activeConnectionId = computed(() => gridStore.activeConnectionId);
const activeDbName = computed(() => gridStore.activeDbName);

// Watch active table schema details and fetch if needed
watch(
  [activeConnectionId, activeTableSchema, activeDbName, activeTable],
  async ([connId, schema, db, table]) => {
    if (connId && schema && table) {
      await diagramStore.fetchSchemaDetails(connId, schema, db);
    }
  },
  { immediate: true }
);

// Cache key matching the schema details store
const cacheKey = computed(() => {
  if (!activeConnectionId.value || !activeTableSchema.value) return '';
  return diagramStore.getCacheKey(
    activeConnectionId.value,
    activeTableSchema.value,
    activeDbName.value
  );
});

const schemaDetails = computed(() => {
  const key = cacheKey.value;
  return key ? diagramStore.diagrams[key] || null : null;
});

// Outgoing foreign key relation mapping for a specific column
const getForeignKeyReference = (colName: string) => {
  if (!schemaDetails.value || !activeTable.value) return null;
  return (
    schemaDetails.value.relations.find(
      (r) => r.sourceTable === activeTable.value && r.sourceColumn === colName
    ) || null
  );
};

// Outbound & Inbound relationships for the current table
const relations = computed(() => {
  if (!schemaDetails.value || !activeTable.value) {
    return { outbound: [], inbound: [] };
  }
  const rels = schemaDetails.value.relations || [];
  return {
    outbound: rels.filter((r) => r.sourceTable === activeTable.value),
    inbound: rels.filter((r) => r.targetTable === activeTable.value)
  };
});

const navigateToTable = (tableName: string) => {
  if (!activeConnectionId.value || !activeTableSchema.value) return;
  tabsStore.openTableTab(
    tableName,
    activeTableSchema.value,
    activeConnectionId.value,
    activeDbName.value
  );
};
</script>

<template>
  <div class="bg-surface custom-scrollbar flex-1 overflow-auto p-4 text-[12px]">
    <div v-if="!activeTable" class="text-text-tertiary italic">
      Select a table to view its properties.
    </div>
    <div v-else class="flex flex-col gap-5">
      <!-- Table Header Info -->
      <div>
        <div class="text-text-primary text-[13px] font-semibold">Columns for {{ activeTable }}</div>
        <div class="text-text-tertiary mt-0.5 text-[10px]">
          Schema: {{ activeTableSchema }} <span v-if="activeDbName">| DB: {{ activeDbName }}</span>
        </div>
      </div>

      <!-- Columns list -->
      <div v-if="columns.length === 0" class="text-text-tertiary">No columns found.</div>
      <div v-else class="flex flex-col gap-2">
        <div
          v-for="col in columns"
          :key="col.name"
          class="border-border bg-surface-alt hover:border-border/60 flex flex-col gap-1 rounded-lg border p-2.5 transition-colors"
        >
          <div class="flex items-center justify-between">
            <span class="text-text-primary font-mono font-semibold">{{ col.name }}</span>
            <div class="flex items-center gap-1.5">
              <span
                v-if="col.isPrimaryKey"
                class="rounded border border-amber-500/20 bg-amber-500/10 px-1.5 py-0.5 text-[9px] font-bold text-amber-500"
                title="Primary Key"
                >PK</span
              >
              <span
                v-if="getForeignKeyReference(col.name)"
                class="cursor-pointer rounded border border-indigo-500/20 bg-indigo-500/10 px-1.5 py-0.5 text-[9px] font-bold text-indigo-400"
                title="Foreign Key - Click to navigate"
                @click="navigateToTable(getForeignKeyReference(col.name)!.targetTable)"
                >FK</span
              >
            </div>
          </div>
          <div class="text-text-secondary flex justify-between text-[11px]">
            <span
              >Type: <span class="text-primary font-mono">{{ col.dataType }}</span></span
            >
            <span class="text-text-tertiary">{{ col.isNullable ? 'nullable' : 'not null' }}</span>
          </div>

          <!-- FK target inline reference -->
          <div
            v-if="getForeignKeyReference(col.name)"
            class="text-text-tertiary border-border/10 mt-1.5 flex items-center gap-1 border-t pt-1.5 text-[10.5px]"
          >
            <Link :size="10" class="shrink-0 text-indigo-400" />
            <span>References</span>
            <button
              @click="navigateToTable(getForeignKeyReference(col.name)!.targetTable)"
              class="font-mono text-[11px] font-medium text-indigo-400 transition-colors hover:underline"
            >
              {{ getForeignKeyReference(col.name)!.targetTable }}
            </button>
            <span class="text-text-tertiary"
              >({{ getForeignKeyReference(col.name)!.targetColumn }})</span
            >
          </div>
        </div>
      </div>

      <!-- Relations Section -->
      <div
        v-if="relations.outbound.length > 0 || relations.inbound.length > 0"
        class="border-border/40 flex flex-col gap-4 border-t pt-4"
      >
        <div class="text-text-primary text-[13px] font-semibold">Foreign Key Relations</div>

        <!-- Outbound Relations (Referenced Tables) -->
        <div v-if="relations.outbound.length > 0" class="flex flex-col gap-2">
          <div class="text-text-secondary text-[11px] font-medium">
            Referenced Tables (Outbound)
          </div>
          <div class="flex flex-col gap-1.5">
            <div
              v-for="rel in relations.outbound"
              :key="rel.constraintName"
              class="bg-surface-alt border-border/30 flex flex-col gap-1 rounded-lg border p-2"
            >
              <div class="flex items-center justify-between text-[10.5px]">
                <span class="text-text-tertiary truncate font-mono" :title="rel.constraintName">{{
                  rel.constraintName
                }}</span>
              </div>
              <div class="flex items-center gap-1.5 font-mono text-[11px]">
                <span class="text-text-primary font-medium">{{ rel.sourceColumn }}</span>
                <ArrowRight :size="10" class="text-text-tertiary shrink-0" />
                <button
                  @click="navigateToTable(rel.targetTable)"
                  class="font-semibold text-indigo-400 hover:underline"
                >
                  {{ rel.targetTable }}
                </button>
                <span class="text-text-secondary">({{ rel.targetColumn }})</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Inbound Relations (Referencing Tables) -->
        <div v-if="relations.inbound.length > 0" class="flex flex-col gap-2">
          <div class="text-text-secondary text-[11px] font-medium">
            Referencing Tables (Inbound)
          </div>
          <div class="flex flex-col gap-1.5">
            <div
              v-for="rel in relations.inbound"
              :key="rel.constraintName"
              class="bg-surface-alt border-border/30 flex flex-col gap-1 rounded-lg border p-2"
            >
              <div class="flex items-center justify-between text-[10.5px]">
                <span class="text-text-tertiary truncate font-mono" :title="rel.constraintName">{{
                  rel.constraintName
                }}</span>
              </div>
              <div class="flex items-center gap-1.5 font-mono text-[11px]">
                <button
                  @click="navigateToTable(rel.sourceTable)"
                  class="font-semibold text-indigo-400 hover:underline"
                >
                  {{ rel.sourceTable }}
                </button>
                <span class="text-text-secondary">({{ rel.sourceColumn }})</span>
                <ArrowRight :size="10" class="text-text-tertiary shrink-0" />
                <span class="text-text-primary font-medium">{{ rel.targetColumn }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
