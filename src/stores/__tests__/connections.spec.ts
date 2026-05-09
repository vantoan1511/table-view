import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useConnectionsStore } from '../connections'
import * as Neutralino from '@neutralinojs/lib'

// Mock Neutralino
vi.mock('@neutralinojs/lib', () => ({
  storage: {
    getData: vi.fn(),
    setData: vi.fn()
  }
}))

describe('Connections Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    // Reset window.NL_PORT for testing to enable Neutralino calls
    // @ts-ignore
    window.NL_PORT = 1234
  })

  it('generates a unique id', () => {
    const store = useConnectionsStore()
    const id1 = store.generateId()
    const id2 = store.generateId()
    expect(id1).not.toBe(id2)
    expect(id1).toMatch(/^conn-/)
  })

  it('toggles the new connection modal', () => {
    const store = useConnectionsStore()
    expect(store.showNewConnectionModal).toBe(false)

    store.toggleConnectionModal(true)
    expect(store.showNewConnectionModal).toBe(true)

    store.toggleConnectionModal(false)
    expect(store.showNewConnectionModal).toBe(false)
  })

  it('adds a connection and saves it', async () => {
    const store = useConnectionsStore()
    const conn = {
      id: 'test-1',
      name: 'Test',
      type: 'mysql',
      host: 'localhost',
      port: 3306,
      username: 'root',
      password: 'password123',
      database: '',
      isConnected: false
    }

    await store.addConnection(conn)
    expect(store.connections).toContainEqual(conn)
    expect(Neutralino.storage.setData).toHaveBeenCalled()

    // Verify password was encrypted in storage
    const lastCall = vi.mocked(Neutralino.storage.setData).mock.calls[0]
    const savedData = JSON.parse(lastCall[1])
    expect(savedData[0].password).not.toBe('password123')
  })

  it('loads connections and decrypts passwords', async () => {
    const store = useConnectionsStore()
    const encryptedData = JSON.stringify([{
      id: 'test-1',
      name: 'Test',
      type: 'mysql',
      host: 'localhost',
      port: 3306,
      username: 'root',
      // This is "password123" XORed and base64 encoded manually for the test
      password: 'JAARHxI5GwFGAAM=',
      database: '',
      isConnected: true // Should be reset to false
    }])

    vi.mocked(Neutralino.storage.getData).mockResolvedValue(encryptedData)

    await store.loadConnections()

    expect(store.connections.length).toBe(1)
    expect(store.connections[0].password).toBe('password123')
    expect(store.connections[0].isConnected).toBe(false)
  })

  it('removes a connection', async () => {
    const store = useConnectionsStore()
    const conn = {
      id: 'test-1',
      name: 'Test',
      type: 'mysql',
      host: 'localhost',
      port: 3306,
      username: 'root',
      password: '',
      database: '',
      isConnected: false
    }

    store.connections.push(conn)
    await store.removeConnection('test-1')
    expect(store.connections).not.toContainEqual(conn)
    expect(Neutralino.storage.setData).toHaveBeenCalled()
  })

  it('sets active connection id', async () => {
    const store = useConnectionsStore()
    const id = 'test-id'
    store.connections.push({
      id,
      name: 'Test',
      type: 'mysql',
      host: 'localhost',
      port: 3306,
      username: 'root',
      password: '',
      database: '',
      isConnected: false
    })

    // We don't test the full setActiveConnection because it has complex dynamic imports 
    // and BridgeService calls that would need extensive mocking.
    // But we can check if it updates the ref.
    store.activeConnectionId = id
    expect(store.activeConnectionId).toBe(id)
    expect(store.activeConnection?.id).toBe(id)
  })
})
