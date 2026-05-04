<script setup lang="ts">
import ConnectionList from '@/components/sidebar/ConnectionList.vue'
import SchemaTree from '@/components/sidebar/SchemaTree.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useSchemaStore } from '@/stores/schema'
import { useToastStore } from '@/stores/toast'
import {
  ChevronDown,
  Database,
  Plus,
  RefreshCw,
  Search,
} from 'lucide-vue-next'
import { onMounted, onUnmounted, ref } from 'vue'

const connectionsStore = useConnectionsStore()
const schemaStore = useSchemaStore()
const toastStore = useToastStore()

// ─── Connection Dropdown ──────────────────────────────────────────────────────
const showConnectionMenu = ref(false)
const connectionMenuRef = ref<HTMLElement | null>(null)

const toggleConnectionMenu = () => {
  showConnectionMenu.value = !showConnectionMenu.value
}

const selectConnection = async (id: string) => {
  showConnectionMenu.value = false
  if (connectionsStore.activeConnectionId === id) return
  try {
    await connectionsStore.setActiveConnection(id)
    toastStore.addToast({
      title: 'Connected',
      message: `Switched to ${connectionsStore.connections.find(c => c.id === id)?.name ?? id}`,
      severity: 'success',
      variation: 'filled',
      position: 'bottom-center',
    })
  } catch (err: any) {
    toastStore.addToast({
      title: 'Connection Failed',
      message: err.message,
      severity: 'error',
      variation: 'filled',
      position: 'bottom-center',
    })
  }
}

// ─── Schema Dropdown ──────────────────────────────────────────────────────────
const showSchemaMenu = ref(false)

const toggleSchemaMenu = () => {
  showSchemaMenu.value = !showSchemaMenu.value
}

const selectSchema = async (s: string) => {
  schemaStore.setSelectedSchema(s)
  showSchemaMenu.value = false
  await schemaStore.loadSchema(schemaStore.loadedAllSchemas, undefined, s)
}

const refreshSchema = () => {
  schemaStore.loadSchema(schemaStore.loadedAllSchemas, undefined, schemaStore.selectedSchema)
  toastStore.addToast({
    title: 'Schema Refreshed',
    message: 'Schema has been reloaded.',
    severity: 'info',
    variation: 'outlined',
    position: 'bottom-center',
  })
}

// Close menus on outside click
const handleOutsideClick = () => {
  showConnectionMenu.value = false
  showSchemaMenu.value = false
}

onMounted(() => window.addEventListener('click', handleOutsideClick))
onUnmounted(() => window.removeEventListener('click', handleOutsideClick))
</script>

<template>
  <aside class="flex flex-col w-(--sidebar-width) bg-sidebar border-r border-border shrink-0 overflow-hidden">
    <!-- Connection Dropdown -->
    <div class="flex items-center gap-2 px-3 py-2.5 border-b border-border relative">
      <button
        class="flex items-center gap-2 flex-1 px-2.5 py-1.5 bg-surface border border-border rounded-lg text-[13px] text-text-primary hover:border-border-strong transition-colors cursor-pointer"
        @click.stop="toggleConnectionMenu">
        <span class="w-2.5 h-2.5 rounded-full shrink-0"
          :class="connectionsStore.activeConnection?.isConnected ? 'bg-success' : 'bg-text-tertiary'"></span>
        <span class="flex-1 text-left truncate font-medium">
          {{ connectionsStore.activeConnection?.name ?? 'No Connection' }}
        </span>
        <ChevronDown :size="14" class="text-text-tertiary shrink-0 transition-transform duration-200"
          :class="{ 'rotate-180': showConnectionMenu }" />
      </button>

      <!-- Connection dropdown menu -->
      <div v-if="showConnectionMenu"
        class="absolute left-3 right-3 top-full mt-1 z-50 bg-surface border border-border rounded-lg shadow-lg py-1 text-[12px]"
        @click.stop>
        <div v-if="connectionsStore.connections.length === 0" class="px-3 py-2 text-text-tertiary italic">
          No connections saved
        </div>
        <button v-for="conn in connectionsStore.connections" :key="conn.id"
          class="flex items-center gap-2 w-full px-3 py-2 hover:bg-hover transition-colors text-left cursor-pointer"
          :class="connectionsStore.activeConnectionId === conn.id ? 'text-primary font-medium' : 'text-text-primary'"
          @click="selectConnection(conn.id)">
          <span class="w-2 h-2 rounded-full shrink-0" :class="conn.isConnected ? 'bg-success' : 'bg-text-tertiary'" />
          <span class="truncate flex-1">{{ conn.name }}</span>
          <span class="text-text-tertiary text-[11px] uppercase">{{ conn.type }}</span>
        </button>
        <div class="h-px bg-border my-1" />
        <button
          class="flex items-center gap-2 w-full px-3 py-2 hover:bg-hover text-text-secondary transition-colors cursor-pointer"
          @click="connectionsStore.toggleConnectionModal(true); showConnectionMenu = false">
          <Plus :size="13" />
          New Connection
        </button>
      </div>
    </div>

    <!-- Connections Header -->
    <div class="flex items-center justify-between px-3 py-2">
      <span class="text-[11px] font-semibold text-text-tertiary uppercase tracking-wider">Connections</span>
      <button
        class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-text-secondary hover:bg-hover cursor-pointer transition-colors"
        @click="connectionsStore.toggleConnectionModal(true)" title="Add Connection">
        <Plus :size="14" />
      </button>
    </div>

    <!-- Connection List -->
    <ConnectionList />

    <!-- Database Section -->
    <div class="px-3 py-2 border-t border-border relative">
      <div class="flex items-center justify-between mb-1.5">
        <span class="text-[11px] font-semibold text-text-tertiary uppercase tracking-wider">Schema</span>
        <button
          class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-text-secondary hover:bg-hover cursor-pointer transition-colors"
          title="Refresh Schema" @click="refreshSchema">
          <RefreshCw :size="12" />
        </button>
      </div>
      <button
        class="flex items-center gap-2 w-full px-2.5 py-1.5 bg-surface border border-border rounded-lg text-[13px] text-text-primary hover:border-border-strong transition-colors cursor-pointer"
        @click.stop="toggleSchemaMenu">
        <Database :size="13" class="text-text-tertiary shrink-0" />
        <span class="flex-1 text-left truncate">{{ schemaStore.selectedSchema }}</span>
        <ChevronDown :size="14" class="text-text-tertiary transition-transform duration-200"
          :class="{ 'rotate-180': showSchemaMenu }" />
      </button>

      <!-- Schema dropdown menu -->
      <div v-if="showSchemaMenu && schemaStore.schema.schemas.length > 0"
        class="absolute left-3 right-3 top-full mt-1 z-50 max-h-64 overflow-y-auto bg-surface border border-border rounded-lg shadow-lg py-1 text-[12px]"
        @click.stop>
        <button v-for="s in schemaStore.schema.schemas" :key="s"
          class="flex items-center gap-2 w-full px-3 py-2 hover:bg-hover transition-colors cursor-pointer"
          :class="schemaStore.selectedSchema === s ? 'text-primary font-medium' : 'text-text-primary'"
          @click="selectSchema(s)">
          <Database :size="12" class="text-text-tertiary shrink-0" />
          {{ s }}
        </button>
      </div>
    </div>

    <!-- Search Filter -->
    <div class="px-3 py-1.5">
      <div
        class="flex items-center gap-2 px-2.5 py-1.5 bg-surface border border-border rounded-lg text-[13px] focus-within:border-primary/50 transition-colors">
        <Search :size="13" class="text-text-tertiary shrink-0" />
        <input type="text" placeholder="Filter objects..."
          class="flex-1 bg-transparent border-none outline-none text-[13px] text-text-primary placeholder-text-tertiary"
          :value="schemaStore.filterQuery" @input="schemaStore.setFilter(($event.target as HTMLInputElement).value)" />
      </div>
    </div>

    <!-- Schema Tree -->
    <div class="flex-1 overflow-y-auto">
      <SchemaTree />
    </div>
  </aside>
</template>
