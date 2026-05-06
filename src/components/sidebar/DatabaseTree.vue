<script setup lang="ts">
import DbIcon from '@/components/icons/DbIcon.vue'
import { useConnectionsStore } from '@/stores/connections'
import { useSchemaStore } from '@/stores/schema'
import { useTabsStore } from '@/stores/tabs'
import { useToastStore } from '@/stores/toast'
import type { Connection } from '@/types'
import {
  ChevronRight,
  Code2,
  Copy,
  Database,
  Eye,
  FunctionSquare,
  Layers,
  Loader2,
  MoreVertical,
  Pencil,
  Plus,
  RefreshCw,
  Table2,
  Trash2,
} from 'lucide-vue-next'
import { computed, ref } from 'vue'
import ConfirmDialog from '../ui/ConfirmDialog.vue'
import ContextMenu from '../ui/ContextMenu.vue'

const connectionsStore = useConnectionsStore()
const schemaStore = useSchemaStore()
const tabsStore = useTabsStore()
const toastStore = useToastStore()

// ─── Tree Expansion State ─────────────────────────────────────────────────────
// Which connection nodes are expanded in the tree
const expandedConnections = ref<Record<string, boolean>>({})
// Which object-group nodes are expanded per connection+schema key
const expandedGroups = ref<Record<string, boolean>>({})

const toggleConnection = async (conn: Connection) => {
  const wasExpanded = expandedConnections.value[conn.id]
  expandedConnections.value[conn.id] = !wasExpanded

  if (!wasExpanded) {
    // Expanding: ensure connection is active and schema is loaded
    if (!conn.isConnected) {
      try {
        await connectionsStore.setActiveConnection(conn.id)
      } catch (err: any) {
        expandedConnections.value[conn.id] = false
        toastStore.addToast({
          title: 'Connection Failed',
          message: err.message,
          severity: 'error',
          variation: 'filled',
          position: 'bottom-center',
        })
      }
    } else {
      // Already connected but may not have schema yet for this connection
      connectionsStore.activeConnectionId === conn.id
        ? null
        : await connectionsStore.setActiveConnection(conn.id)

      if (!schemaStore.hasSchemaLoaded(conn.id)) {
        schemaStore.loadSchema(conn.displayAllSchemas, conn.id)
      }
    }
  }
}

const toggleSchema = (connectionId: string, schemaName: string) => {
  schemaStore.setSchemaExpanded(
    connectionId,
    schemaName,
    !schemaStore.isSchemaExpanded(connectionId, schemaName),
  )
}

const groupKey = (connectionId: string, schemaName: string, group: string) =>
  `${connectionId}::${schemaName}::${group}`

const toggleGroup = (connectionId: string, schemaName: string, group: string) => {
  const key = groupKey(connectionId, schemaName, group)
  expandedGroups.value[key] = !expandedGroups.value[key]
}

const isGroupExpanded = (connectionId: string, schemaName: string, group: string) => {
  const key = groupKey(connectionId, schemaName, group)
  // Tables default to expanded, others collapsed
  return expandedGroups.value[key] ?? (group === 'tables')
}

// ─── Context Menu ─────────────────────────────────────────────────────────────
const contextMenu = ref({ show: false, x: 0, y: 0, connId: null as string | null })
const showDeleteConfirm = ref(false)
const idToDelete = ref<string | null>(null)

const connectionName = computed(
  () => connectionsStore.connections.find((c) => c.id === idToDelete.value)?.name || '',
)

const closeContextMenu = () => {
  contextMenu.value.show = false
  contextMenu.value.connId = null
}

const onContextMenu = (event: MouseEvent, id: string) => {
  event.preventDefault()
  event.stopPropagation()
  let x = event.clientX
  let y = event.clientY
  if (x + 200 > window.innerWidth) x -= 200
  if (y + 200 > window.innerHeight) y -= 200
  contextMenu.value = { show: true, x, y, connId: id }
}

const handleContextAction = (action: string) => {
  const id = contextMenu.value.connId
  closeContextMenu()
  if (!id) return

  if (action === 'sql') {
    tabsStore.openSqlEditor(id, '', true, true)
  } else if (action === 'edit') {
    const conn = connectionsStore.connections.find((c) => c.id === id)
    if (conn) connectionsStore.toggleConnectionModal(true, conn)
  } else if (action === 'duplicate') {
    const conn = connectionsStore.connections.find((c) => c.id === id)
    if (conn) {
      const newConn = { ...conn, id: crypto.randomUUID(), name: `${conn.name} (Copy)`, isConnected: false }
      connectionsStore.connections.push(newConn)
      connectionsStore.saveConnections()
    }
  } else if (action === 'refresh') {
    schemaStore.loadSchema(undefined, id)
  } else if (action === 'delete') {
    idToDelete.value = id
    showDeleteConfirm.value = true
  }
}

const confirmDelete = () => {
  if (!idToDelete.value) return
  const id = idToDelete.value
  const idx = connectionsStore.connections.findIndex((c) => c.id === id)
  if (idx !== -1) {
    connectionsStore.connections.splice(idx, 1)
    if (connectionsStore.activeConnectionId === id) {
      connectionsStore.activeConnectionId = null
    }
    connectionsStore.saveConnections()
    delete schemaStore.schemasByConnection[id]
    toastStore.addToast({ message: 'Connection deleted.', severity: 'success', variation: 'subtle' })
  }
  showDeleteConfirm.value = false
  idToDelete.value = null
}

// ─── Table / View click ───────────────────────────────────────────────────────
const openTable = (tableName: string, schemaName: string, connectionId: string) => {
  // Set active connection first if different
  if (connectionsStore.activeConnectionId !== connectionId) {
    connectionsStore.setActiveConnection(connectionId).catch(() => { })
  }
  tabsStore.openTable(tableName, schemaName, connectionId)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────
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

const isActiveTable = (tableName: string, schemaName: string) =>
  tabsStore.activeTab?.tableName === tableName && tabsStore.activeTab?.schema === schemaName
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0 select-none">
    <!-- Connection list -->
    <div class="flex-1 overflow-y-auto py-1">
      <div v-for="conn in connectionsStore.connections" :key="conn.id">
        <!-- Connection row -->
        <div
          class="group flex items-center gap-1.5 w-full px-2 py-1.5 cursor-pointer transition-colors duration-100 hover:bg-hover"
          :class="connectionsStore.activeConnectionId === conn.id ? 'bg-active' : ''" @click="toggleConnection(conn)"
          @contextmenu.prevent.stop="onContextMenu($event, conn.id)">
          <!-- Expand chevron -->
          <ChevronRight :size="13" class="shrink-0 text-text-tertiary transition-transform duration-150"
            :class="expandedConnections[conn.id] ? 'rotate-90' : ''" />

          <!-- Status dot -->
          <span class="w-2 h-2 rounded-full shrink-0"
            :class="conn.isConnected ? 'bg-success' : (colorMap[conn.color] ?? 'bg-conn-gray')" />

          <!-- DB type icon -->
          <DbIcon :type="conn.type" size="14" />

          <!-- Name + host -->
          <div class="flex-1 min-w-0">
            <div class="text-[13px] font-medium truncate leading-tight"
              :class="connectionsStore.activeConnectionId === conn.id ? 'text-primary' : 'text-text-primary'">
              {{ conn.name }}
            </div>
            <div class="text-[11px] text-text-tertiary truncate leading-tight">
              {{ conn.host }}:{{ conn.port }}
            </div>
          </div>

          <!-- Loading spinner -->
          <Loader2 v-if="schemaStore.isConnectionLoading(conn.id)" :size="13"
            class="shrink-0 text-text-tertiary animate-spin" />

          <!-- More button -->
          <span
            class="flex items-center justify-center w-5 h-5 rounded opacity-0 group-hover:opacity-100 text-text-tertiary hover:text-text-secondary hover:bg-border shrink-0 transition-opacity"
            @click.stop="onContextMenu($event, conn.id)">
            <MoreVertical :size="12" />
          </span>
        </div>

        <!-- Connection children (schemas) -->
        <div v-if="expandedConnections[conn.id] && schemaStore.hasSchemaLoaded(conn.id)">
          <div v-for="schemaName in schemaStore.schemasByConnection[conn.id]?.schemas ?? []" :key="schemaName">
            <!-- Schema row -->
            <div class="group flex items-center gap-1.5 pl-7 pr-2 py-1 cursor-pointer hover:bg-hover transition-colors"
              @click="toggleSchema(conn.id, schemaName)">
              <ChevronRight :size="12" class="shrink-0 text-text-tertiary transition-transform duration-150"
                :class="schemaStore.isSchemaExpanded(conn.id, schemaName) ? 'rotate-90' : ''" />
              <Database :size="13" class="shrink-0 text-text-secondary" />
              <span class="text-[12px] font-medium text-text-secondary flex-1 truncate">
                {{ schemaName }}
              </span>
            </div>

            <!-- Schema children (object groups) -->
            <div v-if="schemaStore.isSchemaExpanded(conn.id, schemaName)">
              <!-- Tables group -->
              <div>
                <div class="flex items-center gap-1.5 pl-10 pr-2 py-1 cursor-pointer hover:bg-hover transition-colors"
                  @click="toggleGroup(conn.id, schemaName, 'tables')">
                  <ChevronRight :size="12" class="shrink-0 text-text-tertiary transition-transform duration-150"
                    :class="isGroupExpanded(conn.id, schemaName, 'tables') ? 'rotate-90' : ''" />
                  <Table2 :size="13" class="shrink-0 text-text-tertiary" />
                  <span class="text-[12px] text-text-secondary flex-1">Tables</span>
                  <span class="text-[11px] text-text-tertiary">
                    ({{ schemaStore.getFilteredTables(conn.id, schemaName).length }})
                  </span>
                </div>
                <div v-if="isGroupExpanded(conn.id, schemaName, 'tables')">
                  <div v-if="schemaStore.getFilteredTables(conn.id, schemaName).length === 0"
                    class="pl-16 pr-2 py-1 text-[11px] text-text-tertiary italic">
                    No tables found
                  </div>
                  <button v-for="table in schemaStore.getFilteredTables(conn.id, schemaName)"
                    :key="`${table.schema}.${table.name}`"
                    class="flex items-center gap-2 w-full pl-14 pr-2 py-[3px] text-[12px] rounded-sm transition-colors"
                    :class="isActiveTable(table.name, table.schema)
                      ? 'bg-active text-primary font-medium'
                      : 'text-text-primary hover:bg-hover'" @click="openTable(table.name, table.schema, conn.id)">
                    <Table2 :size="12" class="shrink-0 opacity-60" />
                    <span class="truncate">{{ table.name }}</span>
                  </button>
                </div>
              </div>

              <!-- Views group -->
              <div>
                <div class="flex items-center gap-1.5 pl-10 pr-2 py-1 cursor-pointer hover:bg-hover transition-colors"
                  @click="toggleGroup(conn.id, schemaName, 'views')">
                  <ChevronRight :size="12" class="shrink-0 text-text-tertiary transition-transform duration-150"
                    :class="isGroupExpanded(conn.id, schemaName, 'views') ? 'rotate-90' : ''" />
                  <Eye :size="13" class="shrink-0 text-text-tertiary" />
                  <span class="text-[12px] text-text-secondary flex-1">Views</span>
                  <span class="text-[11px] text-text-tertiary">
                    ({{ schemaStore.getFilteredViews(conn.id, schemaName).length }})
                  </span>
                </div>
                <div v-if="isGroupExpanded(conn.id, schemaName, 'views')">
                  <div v-if="schemaStore.getFilteredViews(conn.id, schemaName).length === 0"
                    class="pl-16 pr-2 py-1 text-[11px] text-text-tertiary italic">
                    No views found
                  </div>
                  <button v-for="view in schemaStore.getFilteredViews(conn.id, schemaName)"
                    :key="`${view.schema}.${view.name}`"
                    class="flex items-center gap-2 w-full pl-14 pr-2 py-[3px] text-[12px] rounded-sm transition-colors text-text-primary hover:bg-hover">
                    <Eye :size="12" class="shrink-0 opacity-60" />
                    <span class="truncate">{{ view.name }}</span>
                  </button>
                </div>
              </div>

              <!-- Functions group -->
              <div>
                <div class="flex items-center gap-1.5 pl-10 pr-2 py-1 cursor-pointer hover:bg-hover transition-colors"
                  @click="toggleGroup(conn.id, schemaName, 'functions')">
                  <ChevronRight :size="12" class="shrink-0 text-text-tertiary transition-transform duration-150"
                    :class="isGroupExpanded(conn.id, schemaName, 'functions') ? 'rotate-90' : ''" />
                  <FunctionSquare :size="13" class="shrink-0 text-text-tertiary" />
                  <span class="text-[12px] text-text-secondary flex-1">Functions</span>
                  <span class="text-[11px] text-text-tertiary">
                    ({{ schemaStore.getFilteredFunctions(conn.id, schemaName).length }})
                  </span>
                </div>
                <div v-if="isGroupExpanded(conn.id, schemaName, 'functions')">
                  <div v-if="schemaStore.getFilteredFunctions(conn.id, schemaName).length === 0"
                    class="pl-16 pr-2 py-1 text-[11px] text-text-tertiary italic">
                    No functions found
                  </div>
                  <button v-for="fn in schemaStore.getFilteredFunctions(conn.id, schemaName)"
                    :key="`${fn.schema}.${fn.name}`"
                    class="flex items-center gap-2 w-full pl-14 pr-2 py-[3px] text-[12px] rounded-sm transition-colors text-text-primary hover:bg-hover">
                    <FunctionSquare :size="12" class="shrink-0 opacity-60" />
                    <span class="truncate">{{ fn.name }}</span>
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Empty state: connected but no schemas -->
          <div v-if="(schemaStore.schemasByConnection[conn.id]?.schemas ?? []).length === 0"
            class="pl-10 pr-2 py-2 text-[11px] text-text-tertiary italic">
            No schemas found
          </div>
        </div>

        <!-- Loading skeleton while fetching -->
        <div v-else-if="expandedConnections[conn.id] && schemaStore.isConnectionLoading(conn.id)"
          class="pl-9 pr-2 py-2">
          <div class="h-3 bg-border rounded animate-pulse mb-1.5 w-24" />
          <div class="h-3 bg-border rounded animate-pulse mb-1.5 w-32" />
          <div class="h-3 bg-border rounded animate-pulse w-20" />
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="connectionsStore.connections.length === 0"
        class="flex flex-col items-center justify-center py-12 px-4 gap-3">
        <Layers :size="32" class="text-text-tertiary opacity-50" />
        <p class="text-[12px] text-text-tertiary text-center">
          No connections configured.<br />Click <strong>+</strong> to add one.
        </p>
      </div>
    </div>

    <!-- Add Connection button -->
    <div class="border-t border-border px-3 py-2 shrink-0">
      <button
        class="flex items-center gap-2 w-full px-3 py-2 rounded-lg text-[12px] text-text-secondary hover:bg-hover hover:text-text-primary transition-colors cursor-pointer"
        @click="connectionsStore.toggleConnectionModal(true)">
        <Plus :size="14" />
        <span>Add Connection</span>
      </button>
    </div>

    <!-- Context Menu -->
    <ContextMenu :show="contextMenu.show" :x="contextMenu.x" :y="contextMenu.y" @close="closeContextMenu">
      <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover"
        @click="handleContextAction('sql')">
        <Code2 :size="13" class="text-text-secondary" />
        <span>Open SQL Editor</span>
      </button>
      <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover"
        @click="handleContextAction('refresh')">
        <RefreshCw :size="13" class="text-text-secondary" />
        <span>Refresh Schema</span>
      </button>
      <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover"
        @click="handleContextAction('edit')">
        <Pencil :size="13" class="text-text-secondary" />
        <span>Edit Connection</span>
      </button>
      <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover"
        @click="handleContextAction('duplicate')">
        <Copy :size="13" class="text-text-secondary" />
        <span>Duplicate</span>
      </button>
      <div class="h-px bg-border my-1 w-full" />
      <button class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-danger hover:bg-danger-light"
        @click="handleContextAction('delete')">
        <Trash2 :size="13" />
        <span>Delete Connection</span>
      </button>
    </ContextMenu>

    <!-- Delete Confirmation -->
    <ConfirmDialog v-if="showDeleteConfirm" title="Delete Connection"
      :message="`Are you sure you want to delete '${connectionName}'? This action cannot be undone.`"
      confirm-label="Delete" variant="danger" @confirm="confirmDelete" @cancel="showDeleteConfirm = false" />
  </div>
</template>
