<script setup lang="ts">
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';
import { ChevronRight, Key, Table2 } from 'lucide-vue-next';
import { computed, inject, ref } from 'vue';

const props = defineProps<{
  connectionId: string;
  schemaName: string;
  tableName: string;
  dbName?: string;
  isActive: boolean;
}>();

const emit = defineEmits<{
  open: [];
}>();

const schemaStore = useSchemaStore();
const tabsStore = useTabsStore();
const onEntityContextMenu =
  inject<(event: MouseEvent, type: string, payload: any) => void>('onEntityContextMenu');

const isExpanded = ref(false);
const isIndexesExpanded = ref(false);

const indexes = computed(() => {
  return schemaStore.tableIndexes[props.connectionId]?.[props.tableName] || [];
});

const toggleTable = () => {
  isExpanded.value = !isExpanded.value;
  if (isExpanded.value && indexes.value.length === 0) {
    schemaStore.loadTableIndexes(props.connectionId, props.tableName);
  }
};

const toggleIndexes = () => {
  isIndexesExpanded.value = !isIndexesExpanded.value;
};

const openIndex = (indexName: string) => {
  tabsStore.openIndexTab(indexName, props.tableName, props.schemaName, props.connectionId, props.dbName);
};

const isIndexActive = (indexName: string) =>
  tabsStore.activeTab?.type === 'index' &&
  tabsStore.activeTab?.indexName === indexName &&
  tabsStore.activeTab?.tableName === props.tableName &&
  tabsStore.activeTab?.schema === props.schemaName &&
  tabsStore.activeTab?.connectionId === props.connectionId &&
  tabsStore.activeTab?.dbName === props.dbName;
</script>

<template>
  <div class="table-node">
    <div
      class="flex w-full cursor-pointer items-center gap-1.5 rounded-sm py-0.75 pr-2 pl-9 text-[12px] transition-colors"
      :class="isActive ? 'bg-active text-primary font-medium' : 'text-text-primary hover:bg-hover'"
      @contextmenu.prevent.stop="
        onEntityContextMenu?.($event, 'table', {
          connId: connectionId,
          schemaName,
          dbName,
          tableName
        })
      "
    >
      <div class="flex items-center" @click.stop="toggleTable">
        <ChevronRight
          :size="12"
          class="text-text-tertiary shrink-0 transition-transform duration-150"
          :class="isExpanded ? 'rotate-90' : ''"
        />
      </div>
      <div class="flex flex-1 items-center gap-1.5 truncate" @click="emit('open')">
        <Table2 :size="12" class="shrink-0 opacity-60" />
        <span class="truncate">{{ tableName }}</span>
      </div>
    </div>

    <!-- Indexes group -->
    <div v-if="isExpanded" class="flex flex-col gap-1 py-1 pr-2 pl-12">
      <div
        class="hover:bg-hover flex cursor-pointer items-center gap-1.5 py-0.75 pr-2 transition-colors"
        @click="toggleIndexes"
      >
        <ChevronRight
          :size="10"
          class="text-text-tertiary shrink-0 transition-transform duration-150"
          :class="isIndexesExpanded ? 'rotate-90' : ''"
        />
        <Key :size="11" class="text-text-tertiary shrink-0" />
        <span class="text-text-secondary flex-1 text-[11px]">Indexes</span>
        <span class="text-text-tertiary text-[10px]">({{ indexes.length }})</span>
      </div>

      <div v-if="isIndexesExpanded" class="flex flex-col gap-1 pl-4">
        <div v-if="indexes.length === 0" class="text-text-tertiary py-0.5 text-[10px] italic">
          No indexes found
        </div>
        <button
          v-for="idx in indexes"
          :key="idx.name"
          class="flex w-full items-center gap-1.5 rounded-sm py-0.75 pr-2 pl-1 text-[11px] transition-colors"
          :class="
            isIndexActive(idx.name)
              ? 'bg-active text-primary font-medium'
              : 'text-text-primary hover:bg-hover'
          "
          @click="openIndex(idx.name)"
        >
          <Key :size="10" class="shrink-0 opacity-50" />
          <span class="truncate">{{ idx.name }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
