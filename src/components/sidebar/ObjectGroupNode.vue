<script setup lang="ts">
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';
import { ChevronRight, Eye, FunctionSquare, Table2 } from 'lucide-vue-next';
import { computed, inject } from 'vue';

const props = defineProps<{
  connectionId: string;
  schemaName: string;
  groupType: 'tables' | 'views' | 'functions';
  dbName?: string;
}>();

const schemaStore = useSchemaStore();
const tabsStore = useTabsStore();
const onEntityContextMenu =
  inject<(event: MouseEvent, type: string, payload: any) => void>('onEntityContextMenu');

const groupKey = computed(() => {
  const prefix = props.dbName ? `${props.connectionId}::${props.dbName}` : props.connectionId;
  return `${prefix}::${props.schemaName}::${props.groupType}`;
});

const isExpanded = computed(() => {
  // Tables default to expanded, others collapsed
  return schemaStore.expandedGroups?.[groupKey.value] ?? props.groupType === 'tables';
});

const toggle = () => {
  if (!schemaStore.expandedGroups) schemaStore.expandedGroups = {};
  schemaStore.expandedGroups[groupKey.value] = !isExpanded.value;
};

const objects = computed(() => {
  if (props.groupType === 'tables')
    return schemaStore.getFilteredTables(props.connectionId, props.schemaName, props.dbName);
  if (props.groupType === 'views')
    return schemaStore.getFilteredViews(props.connectionId, props.schemaName, props.dbName);
  return schemaStore.getFilteredFunctions(props.connectionId, props.schemaName, props.dbName);
});

const label = computed(() => {
  if (props.groupType === 'tables') return 'Tables';
  if (props.groupType === 'views') return 'Views';
  return 'Functions';
});

const icon = computed(() => {
  if (props.groupType === 'tables') return Table2;
  if (props.groupType === 'views') return Eye;
  return FunctionSquare;
});

const isActive = (tableName: string) =>
  tabsStore.activeTab?.tableName === tableName &&
  tabsStore.activeTab?.schema === props.schemaName &&
  tabsStore.activeTab?.connectionId === props.connectionId &&
  tabsStore.activeTab?.dbName === props.dbName;

const openObject = (name: string) => {
  if (props.groupType === 'tables' || props.groupType === 'views') {
    tabsStore.openTableTab(name, props.schemaName, props.connectionId, props.dbName);
  }
};
</script>

<template>
  <div class="object-group">
    <div
      class="hover:bg-hover flex cursor-pointer items-center gap-1.5 py-1 pr-2 pl-14 transition-colors"
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
      <component :is="icon" :size="13" class="text-text-tertiary shrink-0" />
      <span class="text-text-secondary flex-1 text-[12px]">{{ label }}</span>
      <span class="text-text-tertiary text-[11px]">({{ objects.length }})</span>
    </div>

    <div v-if="isExpanded" class="object-list">
      <div
        v-if="objects.length === 0"
        class="text-text-tertiary py-1 pr-2 pl-16 text-[11px] italic"
      >
        No {{ label.toLowerCase() }} found
      </div>
      <div class="flex flex-col gap-1 py-1 pr-2 pl-7">
        <button
          v-for="obj in objects"
          :key="obj.name"
          class="flex w-full items-center gap-2 rounded-sm py-0.75 pr-2 pl-14 text-[12px] transition-colors"
          :class="
            isActive(obj.name)
              ? 'bg-active text-primary font-medium'
              : 'text-text-primary hover:bg-hover'
          "
          @click="openObject(obj.name)"
          @contextmenu.prevent.stop="
            groupType === 'tables'
              ? onEntityContextMenu?.($event, 'table', {
                  connId: connectionId,
                  schemaName,
                  dbName,
                  tableName: obj.name
                })
              : null
          "
        >
          <component :is="icon" :size="12" class="shrink-0 opacity-60" />
          <span class="truncate">{{ obj.name }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
