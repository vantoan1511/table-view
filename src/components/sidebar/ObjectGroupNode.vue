<script setup lang="ts">
import { ChevronRight, Table2, Eye, FunctionSquare } from 'lucide-vue-next'
import { useSchemaStore } from '@/stores/schema'
import { useTabsStore } from '@/stores/tabs'
import { computed } from 'vue'

const props = defineProps<{
  connectionId: string
  schemaName: string
  groupType: 'tables' | 'views' | 'functions'
  dbName?: string
}>()

const schemaStore = useSchemaStore()
const tabsStore = useTabsStore()

const groupKey = computed(() => {
  const prefix = props.dbName ? `${props.connectionId}::${props.dbName}` : props.connectionId
  return `${prefix}::${props.schemaName}::${props.groupType}`
})

const isExpanded = computed(() => {
  // Tables default to expanded, others collapsed
  return schemaStore.expandedGroups?.[groupKey.value] ?? (props.groupType === 'tables')
})

const toggle = () => {
  if (!schemaStore.expandedGroups) schemaStore.expandedGroups = {}
  schemaStore.expandedGroups[groupKey.value] = !isExpanded.value
}

const objects = computed(() => {
  if (props.groupType === 'tables') return schemaStore.getFilteredTables(props.connectionId, props.schemaName, props.dbName)
  if (props.groupType === 'views') return schemaStore.getFilteredViews(props.connectionId, props.schemaName, props.dbName)
  return schemaStore.getFilteredFunctions(props.connectionId, props.schemaName, props.dbName)
})

const label = computed(() => {
  if (props.groupType === 'tables') return 'Tables'
  if (props.groupType === 'views') return 'Views'
  return 'Functions'
})

const icon = computed(() => {
  if (props.groupType === 'tables') return Table2
  if (props.groupType === 'views') return Eye
  return FunctionSquare
})

const isActive = (tableName: string) => 
  tabsStore.activeTab?.tableName === tableName && 
  tabsStore.activeTab?.schema === props.schemaName &&
  tabsStore.activeTab?.connectionId === props.connectionId &&
  tabsStore.activeTab?.dbName === props.dbName

const openObject = (name: string) => {
  if (props.groupType === 'tables' || props.groupType === 'views') {
    tabsStore.openTable(name, props.schemaName, props.connectionId, props.dbName)
  }
}
</script>

<template>
  <div class="object-group">
    <div 
      class="flex items-center gap-1.5 pl-10 pr-2 py-1 cursor-pointer hover:bg-hover transition-colors"
      @click="toggle"
    >
      <ChevronRight 
        :size="12" 
        class="shrink-0 text-text-tertiary transition-transform duration-150"
        :class="isExpanded ? 'rotate-90' : ''" 
      />
      <component :is="icon" :size="13" class="shrink-0 text-text-tertiary" />
      <span class="text-[12px] text-text-secondary flex-1">{{ label }}</span>
      <span class="text-[11px] text-text-tertiary">({{ objects.length }})</span>
    </div>

    <div v-if="isExpanded" class="object-list">
      <div v-if="objects.length === 0" class="pl-16 pr-2 py-1 text-[11px] text-text-tertiary italic">
        No {{ label.toLowerCase() }} found
      </div>
      <button 
        v-for="obj in objects" 
        :key="obj.name"
        class="flex items-center gap-2 w-full pl-14 pr-2 py-[3px] text-[12px] rounded-sm transition-colors"
        :class="isActive(obj.name) ? 'bg-active text-primary font-medium' : 'text-text-primary hover:bg-hover'" 
        @click="openObject(obj.name)"
      >
        <component :is="icon" :size="12" class="shrink-0 opacity-60" />
        <span class="truncate">{{ obj.name }}</span>
      </button>
    </div>
  </div>
</template>
