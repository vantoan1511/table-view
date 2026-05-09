<script setup lang="ts">
import { ChevronRight, Database } from 'lucide-vue-next'
import { useSchemaStore } from '@/stores/schema'
import ObjectGroupNode from './ObjectGroupNode.vue'
import { computed } from 'vue'

const props = defineProps<{
  connectionId: string
  schemaName: string
  dbName?: string
  indent?: boolean
}>()

const schemaStore = useSchemaStore()

const expansionKey = computed(() => props.dbName ? `__db__${props.dbName}.${props.schemaName}` : props.schemaName)
const isExpanded = computed(() => schemaStore.isSchemaExpanded(props.connectionId, expansionKey.value))

const toggle = () => {
  schemaStore.setSchemaExpanded(props.connectionId, expansionKey.value, !isExpanded.value)
}
</script>

<template>
  <div class="schema-node">
    <div 
      class="group flex items-center gap-1.5 pr-2 py-1 cursor-pointer hover:bg-hover transition-colors"
      :class="indent ? 'pl-11' : 'pl-7'"
      @click="toggle"
    >
      <ChevronRight 
        :size="12" 
        class="shrink-0 text-text-tertiary transition-transform duration-150"
        :class="isExpanded ? 'rotate-90' : ''" 
      />
      <Database :size="13" class="shrink-0 text-text-secondary" />
      <span class="text-[12px] font-medium text-text-secondary flex-1 truncate">{{ schemaName }}</span>
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
