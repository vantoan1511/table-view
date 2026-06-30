<script setup lang="ts">
import { useSchemaStore } from '@/stores/schema';
import type { Tab, DbIndex } from '@/types';
import { computed } from 'vue';
import { Key } from 'lucide-vue-next';

const props = defineProps<{
  tab: Tab;
}>();

const schemaStore = useSchemaStore();

const indexInfo = computed<DbIndex | undefined>(() => {
  if (!props.tab.connectionId || !props.tab.tableName || !props.tab.indexName) return undefined;
  const indexes = schemaStore.tableIndexes[props.tab.connectionId]?.[props.tab.tableName] || [];
  return indexes.find((idx) => idx.name === props.tab.indexName);
});
</script>

<template>
  <div class="index-details flex h-full flex-col overflow-y-auto p-6">
    <div v-if="indexInfo" class="flex flex-col gap-6">
      <div class="flex items-center gap-3">
        <div class="bg-primary/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg">
          <Key class="text-primary" :size="20" />
        </div>
        <div>
          <h1 class="text-text-primary text-xl font-semibold">{{ indexInfo.name }}</h1>
          <p class="text-text-tertiary mt-1 text-sm">
            {{ tab.schema ? `${tab.schema}.` : '' }}{{ tab.tableName }}
          </p>
        </div>
      </div>

      <!-- Properties -->
      <div class="border-border bg-surface-elevated flex flex-col rounded-md border">
        <div class="border-border text-text-secondary border-b px-4 py-2.5 text-sm font-medium">
          Properties
        </div>
        <div class="flex flex-col gap-3 p-4">
          <div class="flex items-center">
            <span class="text-text-tertiary w-32 text-sm">Primary Key:</span>
            <span class="text-text-primary text-sm">{{
              indexInfo.isPrimaryKey ? 'Yes' : 'No'
            }}</span>
          </div>
          <div class="flex items-center">
            <span class="text-text-tertiary w-32 text-sm">Unique:</span>
            <span class="text-text-primary text-sm">{{ indexInfo.isUnique ? 'Yes' : 'No' }}</span>
          </div>
          <div class="flex items-center" v-if="indexInfo.indexType">
            <span class="text-text-tertiary w-32 text-sm">Index Type:</span>
            <span class="text-text-primary text-sm uppercase">{{ indexInfo.indexType }}</span>
          </div>
        </div>
      </div>

      <!-- Columns -->
      <div class="border-border bg-surface-elevated flex flex-col rounded-md border">
        <div class="border-border text-text-secondary border-b px-4 py-2.5 text-sm font-medium">
          Columns
        </div>
        <div class="flex flex-col p-2">
          <div
            v-for="(col, i) in indexInfo.columns"
            :key="i"
            class="text-text-primary hover:bg-hover flex items-center rounded-sm px-3 py-2 text-sm transition-colors"
          >
            {{ col }}
          </div>
        </div>
      </div>

      <!-- DDL -->
      <div
        v-if="indexInfo.ddl"
        class="border-border bg-surface-elevated flex flex-col rounded-md border"
      >
        <div class="border-border text-text-secondary border-b px-4 py-2.5 text-sm font-medium">
          DDL
        </div>
        <div class="p-4">
          <pre class="text-text-primary font-mono text-sm whitespace-pre-wrap">{{
            indexInfo.ddl
          }}</pre>
        </div>
      </div>
    </div>

    <div v-else class="text-text-tertiary flex h-full flex-col items-center justify-center">
      <p>Index information not found</p>
    </div>
  </div>
</template>
