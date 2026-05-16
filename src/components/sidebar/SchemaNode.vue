<script setup lang="ts">
import ObjectGroupNode from './ObjectGroupNode.vue';

import { useSchemaStore } from '@/stores/schema';
import { ChevronRight, Database } from 'lucide-vue-next';
import { computed, inject } from 'vue';

const props = defineProps<{
  connectionId: string;
  schemaName: string;
  dbName?: string;
  indent?: boolean;
}>();

const schemaStore = useSchemaStore();
const onEntityContextMenu =
  inject<(event: MouseEvent, type: string, payload: any) => void>('onEntityContextMenu');

const expansionKey = computed(() =>
  props.dbName ? `__db__${props.dbName}.${props.schemaName}` : props.schemaName
);
const isExpanded = computed(() =>
  schemaStore.isSchemaExpanded(props.connectionId, expansionKey.value)
);

const toggle = () => {
  schemaStore.setSchemaExpanded(props.connectionId, expansionKey.value, !isExpanded.value);
};
</script>

<template>
  <div class="schema-node">
    <div
      class="group hover:bg-hover flex cursor-pointer items-center gap-1.5 py-1 pr-2 pl-7 transition-colors"
      :class="indent ? 'pl-11' : 'pl-7'"
      @click="toggle"
      @contextmenu.prevent.stop="
        onEntityContextMenu?.($event, 'schema', { connId: connectionId, schemaName, dbName })
      "
    >
      <ChevronRight
        :size="12"
        class="text-text-tertiary shrink-0 transition-transform duration-150"
        :class="isExpanded ? 'rotate-90' : ''"
      />
      <Database :size="13" class="text-text-secondary shrink-0" />
      <span class="text-text-secondary flex-1 truncate text-[12px] font-medium">{{
        schemaName
      }}</span>
    </div>

    <div v-if="isExpanded" class="schema-children">
      <ObjectGroupNode
        :connection-id="connectionId"
        :schema-name="schemaName"
        :db-name="dbName"
        group-type="tables"
      />
      <ObjectGroupNode
        :connection-id="connectionId"
        :schema-name="schemaName"
        :db-name="dbName"
        group-type="views"
      />
      <ObjectGroupNode
        :connection-id="connectionId"
        :schema-name="schemaName"
        :db-name="dbName"
        group-type="functions"
      />
    </div>
  </div>
</template>
