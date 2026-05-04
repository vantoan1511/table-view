<script setup lang="ts">
import ColorPicker from '@/components/ui/ColorPicker.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import { useConnectionsStore } from '@/stores/connections'
import type { Connection, DbType, OracleConnectType, OracleRole } from '@/types'
import {
  CircleHelp,
  Download,
  Eye,
  EyeOff,
  Loader2,
  X,
} from 'lucide-vue-next'
import { reactive, ref, watch } from 'vue'

const connectionsStore = useConnectionsStore()

const activeTab = ref<'general' | 'ssl' | 'advanced'>('general')
const showPassword = ref(false)
const testStatus = ref<'ready' | 'testing' | 'success' | 'error'>('ready')
const importError = ref('')

const dbTypes: { key: DbType; label: string; icon: string }[] = [
  { key: 'postgresql', label: 'PostgreSQL', icon: '🐘' },
  { key: 'mysql', label: 'MySQL', icon: '🐬' },
  { key: 'sqlite', label: 'SQLite', icon: '📦' },
  { key: 'oracle', label: 'Oracle', icon: '⭕' },
  { key: 'sqlserver', label: 'SQL Server', icon: '🔷' },
  { key: 'mariadb', label: 'MariaDB', icon: '🦭' },
  { key: 'redis', label: 'Redis', icon: '🔴' },
]

const form = reactive<Omit<Connection, 'id' | 'isConnected'>>({
  name: '',
  type: 'postgresql',
  host: 'localhost',
  port: 5432,
  database: 'postgres',
  username: 'postgres',
  password: '',
  color: 'indigo',
  environment: 'development',
  connectionTimeout: 30,
  queryTimeout: 60,
  applicationName: 'Table View',
  comment: '',
  savePassword: false,
  displayAllSchemas: false,
  oracleConnectType: 'serviceName',
  oracleRole: 'normal',
})

// Initialize form if editing
watch(
  () => connectionsStore.connectionToEdit,
  (conn) => {
    if (conn) {
      Object.assign(form, {
        ...conn,
        oracleConnectType: conn.oracleConnectType || 'serviceName',
        oracleRole: conn.oracleRole || 'normal',
      })
    } else {
      // Reset to defaults
      Object.assign(form, {
        name: '',
        type: 'postgresql',
        host: 'localhost',
        port: 5432,
        database: 'postgres',
        username: 'postgres',
        password: '',
        color: 'indigo',
        environment: 'development',
        connectionTimeout: 30,
        queryTimeout: 60,
        applicationName: 'Table View',
        comment: '',
        savePassword: false,
        displayAllSchemas: false,
        oracleConnectType: 'serviceName',
        oracleRole: 'normal',
      })
    }
  },
  { immediate: true }
)

const portDefaults: Partial<Record<DbType, number>> = {
  postgresql: 5432,
  mysql: 3306,
  oracle: 1521,
  sqlserver: 1433,
  mariadb: 3306,
  redis: 6379,
}

import * as Neutralino from '@neutralinojs/lib'

const selectDbType = (type: DbType) => {
  form.type = type
  form.port = portDefaults[type] ?? 5432
  if (type === 'oracle') {
    form.database = form.database === 'postgres' ? 'FREEPDB1' : form.database
    form.oracleConnectType = form.oracleConnectType ?? 'serviceName'
    form.oracleRole = form.oracleRole ?? 'normal'
  }
}

// ─── Import Connection ────────────────────────────────────────────────────────
const handleImportConnection = async () => {
  importError.value = ''
  try {
    let fileContent = ''

    if (window.NL_PORT) {
      const filePath = await Neutralino.os.showOpenDialog('Import Connection Profile', {
        filters: [{ name: 'JSON files', extensions: ['json'] }],
        multiSelections: false,
      })
      if (!filePath || filePath.length === 0) return
      if (filePath[0]) {
        fileContent = await Neutralino.filesystem.readFile(filePath[0])
      }
    } else {
      // Browser fallback via <input type="file">
      fileContent = await new Promise((resolve, reject) => {
        const input = document.createElement('input')
        input.type = 'file'
        input.accept = '.json'
        input.onchange = () => {
          const file = input.files?.[0]
          if (!file) return reject(new Error('No file selected'))
          const reader = new FileReader()
          reader.onload = (e) => resolve(e.target?.result as string)
          reader.onerror = () => reject(new Error('Failed to read file'))
          reader.readAsText(file)
        }
        input.click()
      })
    }

    const parsed = JSON.parse(fileContent)
    // Support both single connection object and array (first item)
    const conn = Array.isArray(parsed) ? parsed[0] : parsed

    if (!conn || typeof conn !== 'object') {
      throw new Error('Invalid connection profile format')
    }

    const importedType = (conn.type || 'postgresql') as DbType

    Object.assign(form, {
      name: conn.name || '',
      type: importedType,
      host: conn.host || 'localhost',
      port: conn.port || portDefaults[importedType] || 5432,
      database: conn.database || '',
      username: conn.username || conn.user || '',
      password: conn.password || '',
      color: conn.color || 'indigo',
      environment: conn.environment || 'development',
      connectionTimeout: conn.connectionTimeout || 30,
      queryTimeout: conn.queryTimeout || 60,
      applicationName: conn.applicationName || 'Table View',
      comment: conn.comment || '',
      savePassword: conn.savePassword ?? false,
      displayAllSchemas: conn.displayAllSchemas ?? false,
      oracleConnectType: (conn.oracleConnectType || 'serviceName') as OracleConnectType,
      oracleRole: (conn.oracleRole || 'normal') as OracleRole,
    })
  } catch (err: any) {
    importError.value = err.message || 'Failed to import connection'
    console.error('Import failed:', err)
  }
}

const handleTestConnection = () => {
  testStatus.value = 'testing'

  if (window.NL_PORT) {
    const reqId = Date.now().toString()

    const onTestResult = (evt: any) => {
      const payload = evt.detail
      if (payload.reqId === reqId) {
        if (payload.success) {
          testStatus.value = 'success'
          setTimeout(() => { testStatus.value = 'ready' }, 2000)
        } else {
          testStatus.value = 'error'
          console.error("Connection failed:", payload.error)
        }
        Neutralino.events.off('dbBridge.testConnectionResult', onTestResult)
      }
    }

    Neutralino.events.on('dbBridge.testConnectionResult', onTestResult)
    Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.testConnection', {
      reqId,
      config: form
    })
  } else {
    // Dev fallback
    setTimeout(() => {
      testStatus.value = 'success'
      setTimeout(() => { testStatus.value = 'ready' }, 2000)
    }, 1500)
  }
}

const handleSave = () => {
  if (connectionsStore.connectionToEdit) {
    connectionsStore.updateConnection(connectionsStore.connectionToEdit.id, { ...form })
  } else {
    const conn: Connection = {
      ...form,
      id: connectionsStore.generateId(),
      isConnected: false,
    }
    connectionsStore.addConnection(conn)
  }
  connectionsStore.toggleConnectionModal(false)
}

const handleClose = () => {
  connectionsStore.toggleConnectionModal(false)
}
</script>


<template>
  <!-- Overlay -->
  <Teleport to="body">
    <div v-if="connectionsStore.showNewConnectionModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-sm"
      @click.self="handleClose">
      <!-- Modal -->
      <div class="bg-surface rounded-xl shadow-modal w-[920px] max-h-[90vh] flex flex-col overflow-hidden animate-in">
        <!-- Header -->
        <div class="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <h2 class="text-[15px] font-semibold text-text-primary">
            {{ connectionsStore.connectionToEdit ? 'Edit Connection' : 'New Connection' }}
          </h2>
          <button
            class="flex items-center justify-center w-7 h-7 rounded-md text-text-tertiary hover:text-text-secondary hover:bg-hover cursor-pointer transition-colors"
            @click="handleClose">
            <X :size="16" />
          </button>
        </div>

        <!-- Body -->
        <div class="flex flex-1 min-h-0">
          <!-- Left: DB Type Selector -->
          <div class="w-[180px] bg-sidebar border-r border-border py-3 flex flex-col">
            <div class="px-3 pb-2 text-[10px] font-semibold text-text-tertiary uppercase tracking-wider">
              Connection Type
            </div>
            <div class="flex-1 px-2">
              <button v-for="db in dbTypes" :key="db.key"
                class="flex items-center gap-2.5 w-full px-3 py-2 rounded-lg text-[13px] cursor-pointer transition-all duration-150 mb-0.5"
                :class="form.type === db.key
                  ? 'bg-active text-primary font-medium border border-primary/20'
                  : 'text-text-primary hover:bg-hover border border-transparent'
                  " @click="selectDbType(db.key)">
                <span class="text-base">{{ db.icon }}</span>
                <span>{{ db.label }}</span>
              </button>
            </div>

            <!-- Import Connection -->
            <div class="px-3 pt-2 border-t border-border mt-2">
              <button
                class="flex items-center gap-2 w-full px-3 py-2 text-[12px] text-text-secondary hover:text-text-primary hover:bg-hover rounded-lg transition-colors cursor-pointer"
                @click="handleImportConnection">
                <Download :size="14" />
                <span>Import Connection</span>
              </button>
              <p v-if="importError" class="px-3 mt-1 text-[11px] text-danger">{{ importError }}</p>
            </div>
          </div>

          <!-- Right: Form -->
          <div class="flex-1 flex flex-col min-h-0">
            <!-- Tabs -->
            <div class="flex items-center gap-1 px-5 pt-3 pb-0 border-b border-border">
              <button v-for="tab in (['general', 'ssl', 'advanced'] as const)" :key="tab"
                class="px-3 py-2 text-[13px] font-medium capitalize transition-colors cursor-pointer -mb-px" :class="activeTab === tab
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-text-secondary hover:text-text-primary'
                  " @click="activeTab = tab">
                {{ tab === 'ssl' ? 'SSL' : tab }}
              </button>
            </div>

            <!-- Form Content -->
            <div class="flex-1 overflow-y-auto px-5 py-4">
              <div v-if="activeTab === 'general'" class="grid grid-cols-2 gap-x-6 gap-y-4">
                <!-- Connection Name -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Connection Name</label>
                  <input v-model="form.name" type="text" placeholder="e.g. Local Postgres"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface placeholder-text-tertiary focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                </div>

                <!-- Connection Color -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Connection Color</label>
                  <ColorPicker v-model="form.color" />
                </div>

                <!-- Host -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Host</label>
                  <input v-model="form.host" type="text"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                </div>

                <!-- Environment -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Environment</label>
                  <select v-model="form.environment"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all appearance-none cursor-pointer">
                    <option value="development">Development</option>
                    <option value="staging">Staging</option>
                    <option value="production">Production</option>
                  </select>
                </div>

                <!-- Port -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Port</label>
                  <input v-model.number="form.port" type="number"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                </div>

                <!-- Connection Timeout -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Connection Timeout (s)</label>
                  <input v-model.number="form.connectionTimeout" type="number"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                </div>

                <!-- Database -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">
                    {{ form.type === 'oracle' ? (form.oracleConnectType === 'sid' ? 'SID' : 'Service Name') : 'Database' }}
                  </label>
                  <input v-model="form.database" type="text"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                </div>

                <!-- Query Timeout -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Query Timeout (s)</label>
                  <input v-model.number="form.queryTimeout" type="number"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                </div>

                <!-- Username -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Username</label>
                  <input v-model="form.username" type="text"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                </div>

                <!-- Application Name -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Application Name</label>
                  <input v-model="form.applicationName" type="text"
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                </div>

                <!-- Password -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Password</label>
                  <div class="relative">
                    <input v-model="form.password" :type="showPassword ? 'text' : 'password'"
                      class="w-full px-3 py-2 pr-10 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all" />
                    <button type="button"
                      class="absolute right-2.5 top-1/2 -translate-y-1/2 text-text-tertiary hover:text-text-secondary cursor-pointer"
                      @click="showPassword = !showPassword">
                      <Eye v-if="!showPassword" :size="15" />
                      <EyeOff v-else :size="15" />
                    </button>
                  </div>
                  <div class="flex items-center gap-1.5 mt-2">
                    <ToggleSwitch v-model="form.savePassword" />
                    <span class="text-[12px] text-text-secondary">Save password</span>
                    <CircleHelp :size="13" class="text-text-tertiary" />
                  </div>
                  <div class="flex items-center gap-1.5 mt-2">
                    <ToggleSwitch v-model="form.displayAllSchemas" />
                    <span class="text-[12px] text-text-secondary">Display all schemas</span>
                    <CircleHelp :size="13" class="text-text-tertiary" title="If enabled, loads all schemas instead of just the default public schema." />
                  </div>
                </div>

                <!-- Comment -->
                <div>
                  <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Comment <span
                      class="font-normal text-text-tertiary">(optional)</span></label>
                  <textarea v-model="form.comment" rows="3" placeholder="Add any notes about this connection..."
                    class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface placeholder-text-tertiary focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all resize-y"></textarea>
                </div>
              </div>

              <!-- SSL Tab -->
              <div v-else-if="activeTab === 'ssl'" class="text-[13px] text-text-secondary">
                <div class="flex items-center justify-center h-40">
                  <p class="text-text-tertiary">SSL configuration options will appear here.</p>
                </div>
              </div>

              <!-- Advanced Tab -->
              <div v-else class="text-[13px] text-text-secondary">
                <div v-if="form.type === 'oracle'" class="grid grid-cols-2 gap-x-6 gap-y-4">
                  <div>
                    <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Connect Type</label>
                    <select v-model="form.oracleConnectType"
                      class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all appearance-none cursor-pointer">
                      <option value="serviceName">Service Name</option>
                      <option value="sid">SID</option>
                    </select>
                  </div>

                  <div>
                    <label class="block text-[12px] font-medium text-text-secondary mb-1.5">Role</label>
                    <select v-model="form.oracleRole"
                      class="w-full px-3 py-2 border border-border rounded-lg text-[13px] text-text-primary bg-surface focus:border-primary focus:ring-1 focus:ring-primary/20 outline-none transition-all appearance-none cursor-pointer">
                      <option value="normal">Normal</option>
                      <option value="sysdba">SYSDBA</option>
                      <option value="sysoper">SYSOPER</option>
                    </select>
                  </div>
                </div>
                <div v-else class="flex items-center justify-center h-40">
                  <p class="text-text-tertiary">Advanced settings will appear here.</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-between px-5 py-3 border-t border-border bg-muted">
          <!-- Left: Test Connection -->
          <div class="flex items-center gap-3">
            <button id="btn-test-connection"
              class="flex items-center gap-2 px-4 py-2 border border-border rounded-lg text-[13px] text-text-secondary hover:bg-hover hover:border-border-strong cursor-pointer transition-colors"
              :disabled="testStatus === 'testing'" @click="handleTestConnection">
              <Loader2 v-if="testStatus === 'testing'" :size="14" class="animate-spin" />
              <svg v-else class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path
                  d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z" />
              </svg>
              <span>Test Connection</span>
            </button>
            <span class="flex items-center gap-1.5 text-[12px]">
              <span class="w-2 h-2 rounded-full" :class="{
                'bg-success': testStatus === 'ready' || testStatus === 'success',
                'bg-warning animate-pulse': testStatus === 'testing',
                'bg-danger': testStatus === 'error',
              }"></span>
              <span class="text-text-secondary">
                {{ testStatus === 'ready' ? 'Ready to test' : testStatus === 'testing' ? 'Testing...' : testStatus ===
                  'success' ? 'Connection successful!' : 'Connection failed' }}
              </span>
            </span>
          </div>

          <!-- Right: Cancel / Save -->
          <div class="flex items-center gap-2">
            <button
              class="px-4 py-2 border border-border rounded-lg text-[13px] text-text-secondary hover:bg-hover cursor-pointer transition-colors"
              @click="handleClose">
              Cancel
            </button>
            <button id="btn-save-connection"
              class="px-4 py-2 bg-primary hover:bg-primary-hover text-text-inverse rounded-lg text-[13px] font-medium cursor-pointer transition-colors shadow-sm"
              @click="handleSave">
              Save Connection
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.animate-in {
  animation: modal-in 0.2s ease-out;
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: scale(0.96) translateY(8px);
  }

  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>
