import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Connection, ConnectionColor, DbType } from '@/types'

// ─── Mock Data ──────────────────────────────────────────────────────────────

const mockConnections: Connection[] = [
  {
    id: 'conn-1',
    name: 'Local Postgres',
    type: 'postgresql',
    host: 'localhost',
    port: 5432,
    database: 'postgres',
    username: 'postgres',
    password: 'postgres',
    color: 'indigo',
    environment: 'development',
    connectionTimeout: 30,
    queryTimeout: 60,
    applicationName: 'Table View',
    comment: '',
    savePassword: true,
    isConnected: true,
  },
  {
    id: 'conn-2',
    name: 'Analytics DB',
    type: 'postgresql',
    host: 'analytics',
    port: 5432,
    database: 'analytics',
    username: 'analytics',
    password: '',
    color: 'teal',
    environment: 'production',
    connectionTimeout: 30,
    queryTimeout: 60,
    applicationName: 'Table View',
    comment: '',
    savePassword: false,
    isConnected: false,
  },
]

import * as Neutralino from '@neutralinojs/lib'

export const useConnectionsStore = defineStore('connections', () => {
  const connections = ref<Connection[]>([])
  const activeConnectionId = ref<string | null>(null)
  const showNewConnectionModal = ref(false)

  const activeConnection = computed(() =>
    connections.value.find((c) => c.id === activeConnectionId.value) ?? null,
  )

  const connectedConnections = computed(() =>
    connections.value.filter((c) => c.isConnected),
  )

  async function loadConnections() {
    if (window.NL_PORT) {
      try {
        const data = await Neutralino.storage.getData('connections')
        connections.value = JSON.parse(data)
        // Reset all isConnected states on boot
        connections.value.forEach(c => c.isConnected = false)
      } catch (err) {
        // Storage not found, initialize empty
        connections.value = []
      }
    }
  }

  async function saveConnections() {
    if (window.NL_PORT) {
      await Neutralino.storage.setData('connections', JSON.stringify(connections.value))
    }
  }

  async function setActiveConnection(id: string) {
    activeConnectionId.value = id
    // Connect to the actual database using the db-bridge
    const conn = connections.value.find(c => c.id === id)
    if (conn && window.NL_PORT) {
      const reqId = Date.now().toString()
      
      const onConnectResult = async (evt: any) => {
        const payload = evt.detail
        if (payload.reqId === reqId) {
          if (payload.success) {
            conn.isConnected = true
            // Import dynamically to avoid circular dependency
            const { useSchemaStore } = await import('./schema')
            const schemaStore = useSchemaStore()
            schemaStore.loadSchema()
          } else {
            conn.isConnected = false
            console.error("Failed to connect:", payload.error)
          }
          Neutralino.events.off('dbBridge.connectResult', onConnectResult)
        }
      }
      
      Neutralino.events.on('dbBridge.connectResult', onConnectResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.connect', {
        reqId,
        config: conn
      })
    }
  }

  async function addConnection(conn: Connection) {
    connections.value.push(conn)
    await saveConnections()
  }

  async function removeConnection(id: string) {
    connections.value = connections.value.filter((c) => c.id !== id)
    if (activeConnectionId.value === id) {
      activeConnectionId.value = connections.value[0]?.id ?? null
    }
    await saveConnections()
  }

  function toggleConnectionModal(show?: boolean) {
    showNewConnectionModal.value = show ?? !showNewConnectionModal.value
  }

  function generateId(): string {
    return `conn-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  }

  return {
    connections,
    activeConnectionId,
    activeConnection,
    connectedConnections,
    showNewConnectionModal,
    loadConnections,
    saveConnections,
    setActiveConnection,
    addConnection,
    removeConnection,
    toggleConnectionModal,
    generateId,
  }
})
