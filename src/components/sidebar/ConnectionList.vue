<script setup lang="ts">

import { useConnectionsStore } from '@/stores/connections';
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';
import { useToastStore } from '@/stores/toast';
import {
  Code2,
  Copy,
  MoreVertical,
  Pencil,
  RefreshCw,
  Trash2
} from 'lucide-vue-next';
import { computed, ref } from 'vue';
import ConfirmDialog from '../ui/ConfirmDialog.vue';
import ContextMenu from '../ui/ContextMenu.vue';

const connectionsStore = useConnectionsStore()
const toastStore = useToastStore()
const schemaStore = useSchemaStore()
const tabsStore = useTabsStore()

const contextMenu = ref({ show: false, x: 0, y: 0, connId: null as string | null })
const showDeleteConfirm = ref(false)
const idToDelete = ref<string | null>(null)

const connectionName = computed(() => {
  return connectionsStore.connections.find(c => c.id === idToDelete.value)?.name || ''
})

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

const handleSelectConnection = (id: string) => {
  connectionsStore.setActiveConnection(id).catch((err: any) => {
    toastStore.addToast({
      title: 'Connection Failed',
      message: err.message,
      severity: 'error',
      variation: 'filled',
      position: 'bottom-center'
    })
  })
}

const closeContextMenu = () => {
  if (contextMenu.value.show) {
    contextMenu.value.show = false
    contextMenu.value.connId = null
  }
}

const handleContextAction = (action: string) => {
  const id = contextMenu.value.connId
  closeContextMenu()
  if (!id) return

  if (action === 'sql') {
    tabsStore.openSqlEditor()
  } else if (action === 'edit') {
    const conn = connectionsStore.connections.find(c => c.id === id)
    if (conn) {
      connectionsStore.toggleConnectionModal(true, conn)
    }
  } else if (action === 'duplicate') {
    const conn = connectionsStore.connections.find(c => c.id === id)
    if (conn) {
      const newConn = { ...conn, id: crypto.randomUUID(), name: `${conn.name} (Copy)`, isConnected: false }
      connectionsStore.connections.push(newConn)
      connectionsStore.saveConnections()
    }
  } else if (action === 'delete') {
    idToDelete.value = id
    showDeleteConfirm.value = true
  } else if (action === 'refresh') {
    if (connectionsStore.activeConnectionId === id) {
      schemaStore.loadSchema(schemaStore.loadedAllSchemas, id, schemaStore.selectedSchema)
    } else {
      handleSelectConnection(id)
    }
  }
}

const confirmDelete = () => {
  if (!idToDelete.value) return
  const id = idToDelete.value
  const idx = connectionsStore.connections.findIndex(c => c.id === id)
  if (idx !== -1) {
    connectionsStore.connections.splice(idx, 1)
    if (connectionsStore.activeConnectionId === id) {
      connectionsStore.activeConnectionId = null
    }
    connectionsStore.saveConnections()
    toastStore.addToast({
      message: 'Connection deleted successfully',
      severity: 'success',
      variation: 'subtle'
    })
  }
  showDeleteConfirm.value = false
  idToDelete.value = null
}

const onContextMenu = (event: MouseEvent, id: string) => {
  event.preventDefault()
  event.stopPropagation() // Prevent immediate close from document contextmenu listener

  // Keep the menu fully on-screen
  let x = event.clientX
  let y = event.clientY
  const menuWidth = 200
  const menuHeight = 200

  if (x + menuWidth > window.innerWidth) x -= menuWidth
  if (y + menuHeight > window.innerHeight) y -= menuHeight

  contextMenu.value = { show: true, x, y, connId: id }
}
</script>

<template>
  <div class="px-2 pb-1">
    <button v-for="conn in connectionsStore.connections" :key="conn.id"
      class="group flex items-center gap-2.5 w-full px-2.5 py-2 rounded-lg cursor-pointer text-left transition-all duration-150"
      :class="connectionsStore.activeConnectionId === conn.id
        ? 'bg-active border border-primary/20'
        : 'hover:bg-hover border border-transparent'
        " @click="handleSelectConnection(conn.id)" @contextmenu.prevent.stop="onContextMenu($event, conn.id)">
      <!-- Color dot / connected indicator -->
      <span class="w-2.5 h-2.5 rounded-full shrink-0"
        :class="conn.isConnected ? 'bg-success' : colorMap[conn.color] ?? 'bg-conn-gray'"></span>

      <!-- Info -->
      <div class="flex-1 min-w-0">
        <div class="text-[13px] font-medium truncate"
          :class="connectionsStore.activeConnectionId === conn.id ? 'text-primary' : 'text-text-primary'">
          {{ conn.name }}
        </div>
        <div class="text-[11px] text-text-tertiary truncate">
          {{ conn.username }}@{{ conn.host }}:{{ conn.port }}
        </div>
      </div>

      <!-- More button -->
      <span @click.stop="onContextMenu($event, conn.id)"
        class="flex items-center justify-center w-5 h-5 rounded opacity-0 group-hover:opacity-100 text-text-tertiary hover:text-text-secondary hover:bg-border transition-opacity cursor-pointer">
        <MoreVertical :size="13" />
      </span>
    </button>

    <!-- Context Menu -->
    <ContextMenu :show="contextMenu.show" :x="contextMenu.x" :y="contextMenu.y" @close="closeContextMenu">
      <button @click="handleContextAction('sql')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <Code2 :size="14" class="text-text-secondary" /> <span>Open SQL Editor</span>
      </button>
      <button @click="handleContextAction('edit')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <Pencil :size="14" class="text-text-secondary" /> <span>Edit Connection</span>
      </button>
      <button @click="handleContextAction('duplicate')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <Copy :size="14" class="text-text-secondary" /> <span>Duplicate</span>
      </button>
      <div class="h-px bg-border my-1 w-full"></div>
      <button @click="handleContextAction('refresh')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-text-primary hover:bg-hover">
        <RefreshCw :size="14" class="text-text-secondary" /> <span>Refresh</span>
      </button>
      <button @click="handleContextAction('delete')"
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-danger hover:bg-danger-light">
        <Trash2 :size="14" /> <span>Delete Connection</span>
      </button>
    </ContextMenu>

    <!-- Delete Confirmation -->
    <ConfirmDialog v-if="showDeleteConfirm" title="Delete Connection"
      :message="`Are you sure you want to delete the connection '${connectionName}'? This action cannot be undone.`"
      confirm-label="Delete" variant="danger" @confirm="confirmDelete" @cancel="showDeleteConfirm = false" />
  </div>
</template>
