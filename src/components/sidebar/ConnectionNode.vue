<script setup lang="ts">
import DbIcon from '@/components/icons/DbIcon.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useSchemaStore } from '@/stores/schema'
import { useToastStore } from '@/stores/toast'
import type { Connection } from '@/types'
import { 
  ChevronRight, 
  Database, 
  Loader2, 
  Lock, 
  MoreVertical 
} from 'lucide-vue-next'
import { computed } from 'vue'
import DatabaseNode from './DatabaseNode.vue'
import SchemaNode from './SchemaNode.vue'

const props = defineProps<{
  connection: Connection
  isExpanded: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle'): void
  (e: 'contextmenu', event: MouseEvent): void
}>()

const connectionsStore = useConnectionsStore()
const schemaStore = useSchemaStore()

const colorMap: Record<string, string> = {
  indigo: 'bg-conn-indigo',
  blue: 'bg-conn-blue',
  teal: 'bg-conn-teal',
  green: 'bg-conn-green',
  amber: 'bg-conn-amber',
  orange: 'bg-conn-orange',
  pink: 'bg-conn-pink',
  gray: 'bg-conn-gray',
}

const connectionSchemas = computed(() => schemaStore.schemasByConnection[props.connection.id]?.schemas ?? [])
const connectionDatabases = computed(() => schemaStore.schemasByConnection[props.connection.id]?.databases ?? [])
</script>

<template>
  <div class="connection-node">
    <!-- Connection row -->
    <div
      class="group relative flex items-center gap-1.5 w-full px-2 py-1.5 cursor-pointer transition-colors duration-100 hover:bg-hover overflow-hidden"
      :class="connectionsStore.activeConnectionId === connection.id ? 'bg-active' : ''" 
      @click="emit('toggle')"
      @contextmenu.prevent.stop="emit('contextmenu', $event)"
    >
      <!-- Connection Color Flag -->
      <div 
        class="absolute left-0 top-0 bottom-0 w-[3px] transition-all duration-200"
        :class="[
          colorMap[connection.color] ?? 'bg-conn-gray',
          connectionsStore.activeConnectionId === connection.id ? 'opacity-100' : 'opacity-60 group-hover:opacity-100'
        ]"
      />

      <ChevronRight 
        :size="13" 
        class="shrink-0 text-text-tertiary transition-transform duration-150 ml-1"
        :class="isExpanded ? 'rotate-90' : ''" 
      />

      <span v-if="connection.isConnected" class="w-1.5 h-1.5 rounded-full shrink-0 bg-success animate-pulse" />

      <DbIcon :type="connection.type" size="14" :class="connection.isConnected ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'" />

      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-1.5 overflow-hidden">
          <span class="text-[13px] font-medium truncate leading-tight"
            :class="connectionsStore.activeConnectionId === connection.id ? 'text-primary' : 'text-text-primary'">
            {{ connection.name }}
          </span>
          <span v-for="tag in (connection.tags?.split(',') || []).map(t => t.trim()).filter(Boolean)" :key="tag"
            class="shrink-0 px-1 py-0.5 rounded text-[9px] font-bold uppercase leading-none bg-primary/10 text-primary border border-primary/20">
            {{ tag }}
          </span>
        </div>
        <div class="text-[11px] text-text-tertiary truncate leading-tight">
          {{ connection.host }}:{{ connection.port }}
        </div>
      </div>

      <Loader2 v-if="schemaStore.isConnectionLoading(connection.id)" :size="13"
        class="shrink-0 text-text-tertiary animate-spin" />

      <span
        class="flex items-center justify-center w-5 h-5 rounded opacity-0 group-hover:opacity-100 text-text-tertiary hover:text-text-secondary hover:bg-border shrink-0 transition-opacity"
        @click.stop="emit('contextmenu', $event)">
        <MoreVertical :size="12" />
      </span>
    </div>

    <!-- Connection children -->
    <div v-if="isExpanded && schemaStore.hasSchemaLoaded(connection.id)">
      <!-- ALL DATABASES MODE -->
      <template v-if="connectionDatabases.length > 0">
        <DatabaseNode
          v-for="dbName in connectionDatabases"
          :key="dbName"
          :connection-id="connection.id"
          :db-name="dbName"
          :is-active="dbName === connection.database"
          :is-accessible="true"
        />
      </template>

      <!-- NORMAL MODE -->
      <template v-else>
        <SchemaNode 
          v-for="schemaName in connectionSchemas" 
          :key="schemaName"
          :connection-id="connection.id"
          :schema-name="schemaName"
        />
        <div v-if="connectionSchemas.length === 0"
          class="pl-10 pr-2 py-2 text-[11px] text-text-tertiary italic">
          No schemas found
        </div>
      </template>
    </div>

    <!-- Loading skeleton -->
    <div v-else-if="isExpanded && schemaStore.isConnectionLoading(connection.id)"
      class="pl-9 pr-2 py-2">
      <div class="h-3 bg-border rounded animate-pulse mb-1.5 w-24" />
      <div class="h-3 bg-border rounded animate-pulse mb-1.5 w-32" />
      <div class="h-3 bg-border rounded animate-pulse w-20" />
    </div>
  </div>
</template>
