<script setup lang="ts">
import { useSchemaStore } from '@/stores/schema'
import { 
  ChevronRight, 
  Database, 
  Lock,
  Loader2,
  AlertCircle
} from 'lucide-vue-next'
import SchemaNode from './SchemaNode.vue'
import { computed } from 'vue'

const props = defineProps<{
  connectionId: string
  dbName: string
  isActive: boolean
  isAccessible: boolean
}>()

const schemaStore = useSchemaStore()

const isExpanded = computed(() => schemaStore.isDbExpanded(props.connectionId, props.dbName))
const isLoading = computed(() => schemaStore.isDbLoading(props.connectionId, props.dbName))
const error = computed(() => schemaStore.getDbError(props.connectionId, props.dbName))
const dbSchema = computed(() => schemaStore.getDbSchema(props.connectionId, props.dbName))

const toggle = async () => {
  if (!props.isAccessible) return
  
  const nextExpanded = !isExpanded.value
  schemaStore.setDbExpanded(props.connectionId, props.dbName, nextExpanded)
  
  if (nextExpanded && !dbSchema.value) {
    await schemaStore.loadDbSchema(props.connectionId, props.dbName)
  }
}

const schemas = computed(() => dbSchema.value?.schemas ?? [])
</script>

<template>
  <div class="database-node">
    <!-- Database row -->
    <div v-if="isAccessible"
      class="group flex items-center gap-1.5 pl-7 pr-2 py-1 cursor-pointer hover:bg-hover transition-colors"
      @click="toggle">
      <ChevronRight :size="12" class="shrink-0 text-text-tertiary transition-transform duration-150"
        :class="isExpanded ? 'rotate-90' : ''" />
      
      <Loader2 v-if="isLoading" :size="13" class="shrink-0 text-text-tertiary animate-spin" />
      <AlertCircle v-else-if="error" :size="13" class="shrink-0 text-danger" :title="error" />
      <Database v-else :size="13" class="shrink-0" :class="isActive ? 'text-primary' : 'text-text-secondary'" />
      
      <span class="text-[12px] flex-1 truncate" :class="isActive ? 'font-semibold text-primary' : 'text-text-primary'">
        {{ dbName }}
      </span>
      
      <span v-if="isActive" class="text-[9px] font-bold uppercase text-primary/60 bg-primary/10 px-1 rounded">active</span>
    </div>

    <!-- Inaccessible database -->
    <div v-else class="flex items-center gap-1.5 pl-7 pr-2 py-1 opacity-50 cursor-default select-none"
      :title="`Not accessible via this connection`">
      <span class="w-3 shrink-0" />
      <Lock :size="11" class="shrink-0 text-text-tertiary" />
      <Database :size="13" class="shrink-0 text-text-tertiary" />
      <span class="text-[12px] text-text-tertiary flex-1 truncate">{{ dbName }}</span>
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
      <div v-else-if="!error" class="pl-14 pr-2 py-1 text-[11px] text-text-tertiary italic">
        No schemas found
      </div>
    </div>
  </div>
</template>
