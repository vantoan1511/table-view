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

// ─── Store ──────────────────────────────────────────────────────────────────

export const useConnectionsStore = defineStore('connections', () => {
  const connections = ref<Connection[]>(mockConnections)
  const activeConnectionId = ref<string | null>('conn-1')
  const showNewConnectionModal = ref(false)

  const activeConnection = computed(() =>
    connections.value.find((c) => c.id === activeConnectionId.value) ?? null,
  )

  const connectedConnections = computed(() =>
    connections.value.filter((c) => c.isConnected),
  )

  function setActiveConnection(id: string) {
    activeConnectionId.value = id
  }

  function addConnection(conn: Connection) {
    connections.value.push(conn)
  }

  function removeConnection(id: string) {
    connections.value = connections.value.filter((c) => c.id !== id)
    if (activeConnectionId.value === id) {
      activeConnectionId.value = connections.value[0]?.id ?? null
    }
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
    setActiveConnection,
    addConnection,
    removeConnection,
    toggleConnectionModal,
    generateId,
  }
})
