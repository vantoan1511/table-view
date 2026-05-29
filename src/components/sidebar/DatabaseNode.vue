<script setup lang="ts">
import { useSchemaStore } from '@/stores/schema';
import { AlertCircle, ChevronRight, Database, Loader2, Lock } from 'lucide-vue-next';
import { computed, inject } from 'vue';
import SchemaNode from './SchemaNode.vue';
import Tooltip from '@/components/ui/Tooltip.vue';

const props = defineProps<{
  connectionId: string;
  dbName: string;
  isActive: boolean;
  isAccessible: boolean;
}>();

const schemaStore = useSchemaStore();
const onEntityContextMenu =
  inject<(event: MouseEvent, type: string, payload: any) => void>('onEntityContextMenu');

const isExpanded = computed(() => schemaStore.isDbExpanded(props.connectionId, props.dbName));
const isLoading = computed(() => schemaStore.isDbLoading(props.connectionId, props.dbName));
const error = computed(() => schemaStore.getDbError(props.connectionId, props.dbName));
const dbSchema = computed(() => schemaStore.getDbSchema(props.connectionId, props.dbName));

const toggle = async () => {
  if (!props.isAccessible) return;

  const nextExpanded = !isExpanded.value;
  schemaStore.setDbExpanded(props.connectionId, props.dbName, nextExpanded);

  if (nextExpanded && !dbSchema.value) {
    await schemaStore.loadDbSchema(props.connectionId, props.dbName);
  }
};

const schemas = computed(() => dbSchema.value?.schemas ?? []);
</script>

<template>
  <div class="database-node">
    <!-- Database row -->
    <div
      v-if="isAccessible"
      class="group hover:bg-hover flex cursor-pointer items-center gap-1.5 py-1 pr-2 pl-7 transition-colors"
      @click="toggle"
      @contextmenu.prevent.stop="
        onEntityContextMenu?.($event, 'database', { connId: connectionId, dbName })
      "
    >
      <ChevronRight
        :size="12"
        class="text-text-tertiary shrink-0 transition-transform duration-150"
        :class="isExpanded ? 'rotate-90' : ''"
      />

      <Loader2 v-if="isLoading" :size="13" class="text-text-tertiary shrink-0 animate-spin" />
      <span v-else-if="error" class="inline-flex shrink-0">
        <AlertCircle :size="13" class="text-danger" />
        <Tooltip :text="error" position="right" />
      </span>
      <Database
        v-else
        :size="13"
        class="shrink-0"
        :class="isActive ? 'text-primary' : 'text-text-secondary'"
      />

      <span
        class="flex-1 truncate text-[12px]"
        :class="isActive ? 'text-primary font-semibold' : 'text-text-primary'"
      >
        {{ dbName }}
      </span>

      <span
        v-if="isActive"
        class="text-primary/60 bg-primary/10 rounded px-1 text-[9px] font-bold uppercase"
        >active</span
      >
    </div>

    <!-- Inaccessible database -->
    <div
      v-else
      class="flex cursor-default items-center gap-1.5 py-1 pr-2 pl-7 opacity-50 select-none"
    >
      <span class="w-3 shrink-0" />
      <Lock :size="11" class="text-text-tertiary shrink-0" />
      <Database :size="13" class="text-text-tertiary shrink-0" />
      <span class="text-text-tertiary flex-1 truncate text-[12px]">{{ dbName }}</span>
      <Tooltip text="Not accessible via this connection" position="right" />
    </div>

    <!-- Children (Schemas) -->
    <div v-if="isAccessible && isExpanded && !isLoading">
      <div v-if="schemas.length > 0">
        <SchemaNode
          v-for="schemaName in schemas"
          :key="schemaName"
          :connection-id="connectionId"
          :schema-name="schemaName"
          :db-name="dbName"
          indent
        />
      </div>
      <div v-else-if="!error" class="text-text-tertiary py-1 pr-2 pl-14 text-[11px] italic">
        No schemas found
      </div>
    </div>
  </div>
</template>
