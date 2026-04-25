import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Connection, ConnectionColor, DbType } from '@/types'
import * as Neutralino from '@neutralinojs/lib'

// ─── Simple Password Obfuscation ─────────────────────────────────────────────
// Uses a fixed key to XOR-encode passwords before storage.
// Not military-grade crypto, but prevents plaintext passwords on disk.
const ENCRYPT_KEY = 'TableView2026!SecretKey'

function xorCipher(input: string, key: string): string {
  let result = ''
  for (let i = 0; i < input.length; i++) {
    result += String.fromCharCode(input.charCodeAt(i) ^ key.charCodeAt(i % key.length))
  }
  return result
}

function encryptPassword(password: string): string {
  if (!password) return ''
  return btoa(xorCipher(password, ENCRYPT_KEY))
}

function decryptPassword(encrypted: string): string {
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

  async function loadConnections() {
    if (window.NL_PORT) {
      try {
        const data = await Neutralino.storage.getData('connections')
        const loaded = JSON.parse(data) as Connection[]
        // Decrypt passwords and reset connection states on boot
        loaded.forEach(c => {
          c.password = decryptPassword(c.password)
          c.isConnected = false
        })
        connections.value = loaded
      } catch (err) {
        // Storage not found, initialize empty
        connections.value = []
      }
    }
  }

  async function saveConnections() {
    if (window.NL_PORT) {
      // Encrypt passwords before persisting
      const toSave = connections.value.map(c => ({
        ...c,
        password: encryptPassword(c.password)
      }))
      await Neutralino.storage.setData('connections', JSON.stringify(toSave))
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
