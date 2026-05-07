import type { Connection } from '@/types'
import * as Neutralino from '@neutralinojs/lib'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

// ─── Simple Password Obfuscation ─────────────────────────────────────────────
// Uses a fixed key to XOR-encode passwords before storage.
// Not military-grade crypto, but prevents plaintext passwords on disk.
const ENCRYPT_KEY = 'TableView2026!SecretKey'

const xorCipher = (input: string, key: string) : string => {
  let result = ''
  for (let i = 0; i < input.length; i++) {
    result += String.fromCharCode(input.charCodeAt(i) ^ key.charCodeAt(i % key.length))
  }
  return result
}

const encryptPassword = (password: string) : string => {
  if (!password) return ''
  return btoa(xorCipher(password, ENCRYPT_KEY))
}

const decryptPassword = (encrypted: string) : string => {
  if (!encrypted) return ''
  try {
    return xorCipher(atob(encrypted), ENCRYPT_KEY)
  } catch {
    return encrypted // If it fails, assume it's already plaintext (migration)
  }
}


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

  const loadConnections = async () => {
    if (window.NL_PORT) {
      try {
        const data = await Neutralino.storage.getData('connections')
        const loaded = JSON.parse(data) as Connection[]
        // Decrypt passwords and reset connection states on boot
        loaded.forEach(c => {
          c.password = decryptPassword(c.password)
          c.isConnected = false
          c.oracleConnectType = c.oracleConnectType || 'serviceName'
          c.oracleRole = c.oracleRole || 'normal'
        })
        connections.value = loaded
      } catch (err) {
        // Storage not found, initialize empty
        connections.value = []
      }
    }
  }

  const saveConnections = async () => {
    if (window.NL_PORT) {
      // Encrypt passwords before persisting
      const toSave = connections.value.map(c => ({
        ...c,
        password: encryptPassword(c.password)
      }))
      await Neutralino.storage.setData('connections', JSON.stringify(toSave))
    }
  }

  const setActiveConnection = async (id: string) : Promise<any> => {
    // 1. Update active ID immediately so UI reflects intended state
    const previousActiveConnectionId = activeConnectionId.value
    activeConnectionId.value = id
    
    const conn = connections.value.find(c => c.id === id)
    if (!conn) return

    // 2. If already marked connected, we might still want to ensure bridge has it (stateless extension)
    // but we can skip the wait if we're confident. 
    // Actually, always dispatching connect is safest for the bridge's pool.

    if (window.NL_PORT) {
      return new Promise((resolve, reject) => {
        const reqId = Date.now().toString()

        const onConnectResult = async (evt: any) => {
          const payload = evt.detail
          if (payload.reqId === reqId) {
            Neutralino.events.off('dbBridge.connectResult', onConnectResult)
            if (payload.success) {
              conn.isConnected = true
              // Import dynamically to avoid circular dependency
              const { useSchemaStore } = await import('./schema')
              const schemaStore = useSchemaStore()
              schemaStore.setSelectedSchema('')
              schemaStore.loadSchema(conn.displayAllDatabases, id)
              resolve(true)
            } else {
              conn.isConnected = false
              // Rollback if this specific connection attempt failed
              activeConnectionId.value = previousActiveConnectionId
              console.error("Failed to connect:", payload.error)
              reject(new Error(payload.error || `Failed to connect to database: ${conn.name}`))
            }
          }
        }

        Neutralino.events.on('dbBridge.connectResult', onConnectResult)
        Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.connect', {
          reqId,
          connectionId: id,
          config: conn
        })
      })
    }
  }

  const addConnection = async (conn: Connection) => {
    connections.value.push(conn)
    await saveConnections()
  }

  const removeConnection = async (id: string) => {
    connections.value = connections.value.filter((c) => c.id !== id)
    if (activeConnectionId.value === id) {
      activeConnectionId.value = connections.value[0]?.id ?? null
    }
    await saveConnections()
  }

  const connectionToEdit = ref<Connection | null>(null)

  const toggleConnectionModal = (show?: boolean, conn?: Connection) => {
    showNewConnectionModal.value = show ?? !showNewConnectionModal.value
    connectionToEdit.value = conn ?? null
  }

  const generateId = () : string => {
    return `conn-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  }

  const updateConnection = async (id: string, updates: Partial<Connection>) => {
    const conn = connections.value.find(c => c.id === id)
    if (conn) {
      Object.assign(conn, updates)
      await saveConnections()
    }
  }

  return {
    connections,
    activeConnectionId,
    activeConnection,
    connectedConnections,
    showNewConnectionModal,
    connectionToEdit,
    loadConnections,
    saveConnections,
    setActiveConnection,
    addConnection,
    removeConnection,
    updateConnection,
    toggleConnectionModal,
    generateId,
  }
})
