<script setup lang="ts">
import DbIcon from '@/components/icons/DbIcon.vue';
import DatabaseNode from './DatabaseNode.vue';
import SchemaNode from './SchemaNode.vue';

import { useConnectionsStore } from '@/stores/connections';
import { useSchemaStore } from '@/stores/schema';
import { ChevronRight, Loader2, MoreVertical } from 'lucide-vue-next';
import { computed } from 'vue';

import type { Connection } from '@/types';

const props = defineProps<{
  connection: Connection;
  isExpanded: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle'): void;
  (e: 'dblclick'): void;
  (e: 'contextmenu', event: MouseEvent): void;
}>();

const connectionsStore = useConnectionsStore();
const schemaStore = useSchemaStore();

const colorMap: Record<string, string> = {
  indigo: 'bg-conn-indigo',
  blue: 'bg-conn-blue',
  teal: 'bg-conn-teal',
  green: 'bg-conn-green',
  amber: 'bg-conn-amber',
  orange: 'bg-conn-orange',
  pink: 'bg-conn-pink',
  gray: 'bg-conn-gray',
  purple: 'bg-conn-purple',
  rose: 'bg-conn-rose',
  emerald: 'bg-conn-emerald',
  cyan: 'bg-conn-cyan',
  violet: 'bg-conn-violet',
  red: 'bg-conn-red'
};

const connectionSchemas = computed(
  () => schemaStore.schemasByConnection[props.connection.id]?.schemas ?? []
);
const connectionDatabases = computed(
  () => schemaStore.schemasByConnection[props.connection.id]?.databases ?? []
);
</script>

<template>
  <div class="connection-node">
    <!-- Connection row -->
    <div
      class="group hover:bg-hover relative flex w-full cursor-pointer items-center gap-1.5 overflow-hidden px-2 py-1.5 transition-colors duration-100"
      :class="connectionsStore.activeConnectionId === connection.id ? 'bg-active' : ''"
      @click="emit('toggle')"
      @dblclick="emit('dblclick')"
      @contextmenu.prevent.stop="emit('contextmenu', $event)"
    >
      <!-- Connection Color Flag -->
      <div
        class="absolute top-0 bottom-0 left-0 w-0.75 transition-all duration-200"
        :class="[
          colorMap[connection.color] ?? 'bg-conn-gray',
          connectionsStore.activeConnectionId === connection.id
            ? 'opacity-100'
            : 'opacity-60 group-hover:opacity-100'
        ]"
      />

      <ChevronRight
        :size="13"
        class="text-text-tertiary ml-1 shrink-0 transition-transform duration-150"
        :class="isExpanded ? 'rotate-90' : ''"
      />

      <span
        v-if="connection.isConnected"
        class="bg-success h-1.5 w-1.5 shrink-0 animate-pulse rounded-full"
      />

      <DbIcon
        :type="connection.type"
        size="14"
        :class="
          connection.isConnected
            ? ''
            : 'opacity-70 grayscale group-hover:opacity-100 group-hover:grayscale-0'
        "
      />

      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-1.5 overflow-hidden">
          <span
            class="truncate text-[13px] leading-tight font-medium"
            :class="
              connectionsStore.activeConnectionId === connection.id
                ? 'text-primary'
                : 'text-text-primary'
            "
          >
            {{ connection.name }}
          </span>
          <span
            v-for="tag in (connection.tags?.split(',') || []).map((t) => t.trim()).filter(Boolean)"
            :key="tag"
            class="bg-primary/10 text-primary border-primary/20 shrink-0 rounded border px-1 py-0.5 text-[9px] leading-none font-bold uppercase"
          >
            {{ tag }}
          </span>
        </div>
        <div class="text-text-tertiary truncate text-[11px] leading-tight">
          {{ connection.host }}:{{ connection.port }}
        </div>
      </div>

      <Loader2
        v-if="schemaStore.isConnectionLoading(connection.id)"
        :size="13"
        class="text-text-tertiary shrink-0 animate-spin"
      />

      <Button
        variant="text"
        severity="secondary"
        class="h-5 w-5 shrink-0 opacity-0 group-hover:opacity-100"
        @click.stop="emit('contextmenu', $event)"
      >
        <template #icon>
          <MoreVertical class="h-3 w-3" />
        </template>
      </Button>
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
        <div
          v-if="connectionSchemas.length === 0"
          class="text-text-tertiary py-2 pr-2 pl-10 text-[11px] italic"
        >
          No schemas found
        </div>
      </template>
    </div>

    <!-- Loading skeleton -->
    <div
      v-else-if="isExpanded && schemaStore.isConnectionLoading(connection.id)"
      class="py-2 pr-2 pl-9"
    >
      <div class="bg-border mb-1.5 h-3 w-24 animate-pulse rounded" />
      <div class="bg-border mb-1.5 h-3 w-32 animate-pulse rounded" />
      <div class="bg-border h-3 w-20 animate-pulse rounded" />
    </div>
  </div>
</template>
