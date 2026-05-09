<script setup lang="ts">
import DbIcon from '@/components/icons/DbIcon.vue';
import ColorPicker from '@/components/ui/ColorPicker.vue';
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue';
import { DB_TYPES } from '@/lib/dbTypes';
import { useConnectionsStore } from '@/stores/connections';
import { useErrorStore } from '@/stores/error';
import { ConnectionColor, DbType, OracleConnectType, OracleRole, type Connection } from '@/types';
import * as Neutralino from '@neutralinojs/lib';
import { CircleHelp, Download, Eye, EyeOff, Loader2, X } from 'lucide-vue-next';
import { reactive, ref, watch } from 'vue';

const connectionsStore = useConnectionsStore();
const errorStore = useErrorStore();

const activeTab = ref<'general' | 'ssl' | 'advanced'>('general');
const showPassword = ref(false);
const testStatus = ref<'ready' | 'testing' | 'success' | 'error'>('ready');
const importError = ref('');

const form = reactive<Omit<Connection, 'id' | 'isConnected'>>({
  name: 'New connection',
  type: DbType.POSTGRESQL,
  host: 'localhost',
  port: 5432,
  database: 'postgres',
  username: 'postgres',
  password: '',
  color: ConnectionColor.INDIGO,
  tags: '',
  savePassword: false,
  displayAllDatabases: false,
  oracleConnectType: OracleConnectType.SERVICE_NAME,
  oracleRole: OracleRole.NORMAL
});

// Initialize form if editing
watch(
  () => connectionsStore.connectionToEdit,
  (conn) => {
    if (conn) {
      Object.assign(form, {
        ...conn,
        oracleConnectType: conn.oracleConnectType || 'serviceName',
        oracleRole: conn.oracleRole || 'normal'
      });
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
        tags: '',
        savePassword: false,
        displayAllDatabases: false,
        oracleConnectType: 'serviceName',
        oracleRole: 'normal'
      });
    }
  },
  { immediate: true }
);

const portDefaults: Partial<Record<DbType, number>> = Object.fromEntries(
  DB_TYPES.filter((d) => d.defaultPort > 0).map((d) => [d.key, d.defaultPort])
) as Partial<Record<DbType, number>>;

// TODO: Handle default value properly and should not override manual input
const selectDbType = (type: DbType) => {
  form.type = type;
  form.port = portDefaults[type] ?? 5432;
  if (type === 'oracle') {
    form.database = form.database === 'postgres' ? 'FREEPDB1' : form.database;
    form.oracleConnectType = form.oracleConnectType ?? OracleConnectType.SERVICE_NAME;
    form.oracleRole = form.oracleRole ?? OracleRole.NORMAL;
  }
};

// ─── Import Connection ────────────────────────────────────────────────────────
const handleImportConnection = async () => {
  importError.value = '';
  try {
    let fileContent = '';

    if (window.NL_PORT) {
      const filePath = await Neutralino.os.showOpenDialog('Import Connection Profile', {
        filters: [{ name: 'JSON files', extensions: ['json'] }],
        multiSelections: false
      });
      if (!filePath || filePath.length === 0) return;
      if (filePath[0]) {
        fileContent = await Neutralino.filesystem.readFile(filePath[0]);
      }
    } else {
      // Browser fallback via <input type="file">
      fileContent = await new Promise((resolve, reject) => {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.json';
        input.onchange = () => {
          const file = input.files?.[0];
          if (!file) return reject(new Error('No file selected'));
          const reader = new FileReader();
          reader.onload = (e) => resolve(e.target?.result as string);
          reader.onerror = () => reject(new Error('Failed to read file'));
          reader.readAsText(file);
        };
        input.click();
      });
    }

    const parsed = JSON.parse(fileContent);
    // Support both single connection object and array (first item)
    const conn = Array.isArray(parsed) ? parsed[0] : parsed;

    if (!conn || typeof conn !== 'object') {
      throw new Error('Invalid connection profile format');
    }

    const importedType = (conn.type || 'postgresql') as DbType;

    Object.assign(form, {
      name: conn.name || '',
      type: importedType,
      host: conn.host || 'localhost',
      port: conn.port || portDefaults[importedType] || 5432,
      database: conn.database || '',
      username: conn.username || conn.user || '',
      password: conn.password || '',
      color: conn.color || 'indigo',
      tags: conn.tags || '',
      savePassword: conn.savePassword ?? false,
      displayAllDatabases: conn.displayAllDatabases ?? false,
      oracleConnectType: (conn.oracleConnectType || 'serviceName') as OracleConnectType,
      oracleRole: (conn.oracleRole || 'normal') as OracleRole
    });
  } catch (err: any) {
    importError.value = err.message || 'Failed to import connection';
    console.error('Import failed:', err);
  }
};

const handleTestConnection = () => {
  testStatus.value = 'testing';

  if (window.NL_PORT) {
    const reqId = Date.now().toString();

    const onTestResult = (evt: any) => {
      const payload = evt.detail;
      if (payload.reqId === reqId) {
        if (payload.success) {
          testStatus.value = 'success';
          setTimeout(() => {
            testStatus.value = 'ready';
          }, 2000);
        } else {
          testStatus.value = 'error';
          errorStore.showError('Connection Test Failed', payload.error);
          console.error('Connection failed:', payload.error);
        }
        Neutralino.events.off('dbBridge.testConnectionResult', onTestResult);
      }
    };

    Neutralino.events.on('dbBridge.testConnectionResult', onTestResult);
    Neutralino.extensions.dispatch(
      'com.github.vantoan1511.table-view.db-bridge',
      'dbBridge.testConnection',
      {
        reqId,
        config: form
      }
    );
  } else {
    // Dev fallback
    setTimeout(() => {
      testStatus.value = 'success';
      setTimeout(() => {
        testStatus.value = 'ready';
      }, 2000);
    }, 1500);
  }
};

const handleSave = () => {
  if (connectionsStore.connectionToEdit) {
    connectionsStore.updateConnection(connectionsStore.connectionToEdit.id, { ...form });
  } else {
    const conn: Connection = {
      ...form,
      id: connectionsStore.generateId(),
      isConnected: false
    };
    connectionsStore.addConnection(conn);
  }
  connectionsStore.toggleConnectionModal(false);
};

const handleClose = () => {
  connectionsStore.toggleConnectionModal(false);
};
</script>

<template>
  <!-- Overlay -->
  <Teleport to="body">
    <div
      v-if="connectionsStore.showNewConnectionModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-sm"
      @click.self="handleClose"
    >
      <!-- Modal -->
      <div
        id="new-connection-modal"
        class="bg-surface shadow-modal animate-in modal-container flex max-h-[90vh] w-230 flex-col overflow-hidden rounded-xl"
      >
        <!-- Header -->
        <div class="border-border flex items-center justify-between border-b px-5 py-3.5">
          <h2 class="text-text-primary text-[15px] font-semibold">
            {{ connectionsStore.connectionToEdit ? 'Edit Connection' : 'New Connection' }}
          </h2>
          <button
            class="text-text-tertiary hover:text-text-secondary hover:bg-hover flex h-7 w-7 cursor-pointer items-center justify-center rounded-md transition-colors"
            @click="handleClose"
          >
            <X :size="16" />
          </button>
        </div>

        <!-- Body -->
        <div class="flex min-h-0 flex-1">
          <!-- Left: DB Type Selector -->
          <div class="bg-sidebar border-border flex w-45 flex-col border-r py-3">
            <div
              class="text-text-tertiary px-3 pb-2 text-[10px] font-semibold tracking-wider uppercase"
            >
              Connection Type
            </div>
            <div class="flex-1 px-2">
              <button
                v-for="db in DB_TYPES"
                :key="db.key"
                class="mb-0.5 flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2 text-[13px] transition-all duration-150"
                :class="
                  form.type === db.key
                    ? 'bg-active text-primary border-primary/20 border font-medium'
                    : 'text-text-primary hover:bg-hover border border-transparent'
                "
                @click="selectDbType(db.key)"
              >
                <DbIcon :type="db.key" size="18" />
                <span>{{ db.label }}</span>
              </button>
            </div>

            <!-- Import Connection -->
            <div class="border-border mt-2 border-t px-3 pt-2">
              <button
                class="text-text-secondary hover:text-text-primary hover:bg-hover flex w-full cursor-pointer items-center gap-2 rounded-lg px-3 py-2 text-[12px] transition-colors"
                @click="handleImportConnection"
              >
                <Download :size="14" />
                <span>Import Connection</span>
              </button>
              <p v-if="importError" class="text-danger mt-1 px-3 text-[11px]">{{ importError }}</p>
            </div>
          </div>

          <!-- Right: Form -->
          <div class="flex min-h-0 flex-1 flex-col">
            <!-- Tabs -->
            <div class="border-border flex items-center gap-1 border-b px-5 pt-3 pb-0">
              <button
                v-for="tab in ['general', 'ssl', 'advanced'] as const"
                :key="tab"
                class="-mb-px cursor-pointer px-3 py-2 text-[13px] font-medium capitalize transition-colors"
                :class="
                  activeTab === tab
                    ? 'text-primary border-primary border-b-2'
                    : 'text-text-secondary hover:text-text-primary'
                "
                @click="activeTab = tab"
              >
                {{ tab === 'ssl' ? 'SSL' : tab }}
              </button>
            </div>

            <!-- Form Content -->
            <div class="flex-1 overflow-y-auto px-5 py-4">
              <div v-if="activeTab === 'general'" class="grid grid-cols-2 gap-x-6 gap-y-4">
                <!-- Connection Name -->
                <div class="relative">
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                    >Connection Name</label
                  >
                  <div class="relative">
                    <!-- Flag Preview -->
                    <div
                      class="absolute top-0 bottom-0 left-0 w-1 rounded-l-lg transition-colors duration-200"
                      :class="form.color ? `bg-conn-${form.color}` : 'bg-conn-gray'"
                    />
                    <input
                      v-model="form.name"
                      type="text"
                      placeholder="e.g. Local Postgres"
                      class="border-border text-text-primary bg-surface placeholder-text-tertiary focus:border-primary focus:ring-primary/20 w-full rounded-lg border py-2 pr-3 pl-4 text-[13px] transition-all outline-none focus:ring-1"
                    />
                  </div>
                </div>

                <!-- Connection Color -->
                <div>
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                    >Connection Color</label
                  >
                  <ColorPicker v-model="form.color" />
                </div>

                <!-- Host -->
                <div>
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                    >Host</label
                  >
                  <input
                    v-model="form.host"
                    type="text"
                    class="border-border text-text-primary bg-surface focus:border-primary focus:ring-primary/20 w-full rounded-lg border px-3 py-2 text-[13px] transition-all outline-none focus:ring-1"
                  />
                </div>

                <!-- Port -->
                <div>
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                    >Port</label
                  >
                  <input
                    v-model.number="form.port"
                    type="number"
                    class="border-border text-text-primary bg-surface focus:border-primary focus:ring-primary/20 w-full rounded-lg border px-3 py-2 text-[13px] transition-all outline-none focus:ring-1"
                  />
                </div>

                <!-- Database -->
                <div>
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium">
                    {{
                      form.type === 'oracle'
                        ? form.oracleConnectType === 'sid'
                          ? 'SID'
                          : 'Service Name'
                        : 'Database'
                    }}
                  </label>
                  <input
                    v-model="form.database"
                    type="text"
                    class="border-border text-text-primary bg-surface focus:border-primary focus:ring-primary/20 w-full rounded-lg border px-3 py-2 text-[13px] transition-all outline-none focus:ring-1"
                  />
                </div>

                <!-- Username -->
                <div>
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                    >Username</label
                  >
                  <input
                    v-model="form.username"
                    type="text"
                    class="border-border text-text-primary bg-surface focus:border-primary focus:ring-primary/20 w-full rounded-lg border px-3 py-2 text-[13px] transition-all outline-none focus:ring-1"
                  />
                </div>

                <!-- Tags -->
                <div>
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                    >Tags
                    <span class="text-text-tertiary font-normal"
                      >(max 6 chars per tag, comma separated)</span
                    ></label
                  >
                  <input
                    v-model="form.tags"
                    type="text"
                    placeholder="e.g. PROD, DEV"
                    class="border-border text-text-primary bg-surface placeholder-text-tertiary focus:border-primary focus:ring-primary/20 w-full rounded-lg border px-3 py-2 text-[13px] transition-all outline-none focus:ring-1"
                  />
                </div>

                <!-- Password -->
                <div>
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                    >Password</label
                  >
                  <div class="relative">
                    <input
                      v-model="form.password"
                      :type="showPassword ? 'text' : 'password'"
                      class="border-border text-text-primary bg-surface focus:border-primary focus:ring-primary/20 w-full rounded-lg border px-3 py-2 pr-10 text-[13px] transition-all outline-none focus:ring-1"
                    />
                    <button
                      type="button"
                      class="text-text-tertiary hover:text-text-secondary absolute top-1/2 right-2.5 -translate-y-1/2 cursor-pointer"
                      @click="showPassword = !showPassword"
                    >
                      <Eye v-if="!showPassword" :size="15" />
                      <EyeOff v-else :size="15" />
                    </button>
                  </div>
                  <div class="mt-2 flex items-center gap-1.5">
                    <ToggleSwitch v-model="form.savePassword" />
                    <span class="text-text-secondary text-[12px]">Save password</span>
                    <CircleHelp :size="13" class="text-text-tertiary" />
                  </div>
                  <div class="mt-2 flex items-center gap-1.5">
                    <ToggleSwitch v-model="form.displayAllDatabases" />
                    <span class="text-text-secondary text-[12px]">Display all databases</span>
                    <CircleHelp
                      :size="13"
                      class="text-text-tertiary"
                      title="If enabled, loads all databases from the server. Otherwise, only loads the configured database. Schemas are always fully loaded."
                    />
                  </div>
                </div>
              </div>

              <!-- SSL Tab -->
              <div v-else-if="activeTab === 'ssl'" class="text-text-secondary text-[13px]">
                <div class="flex h-40 items-center justify-center">
                  <p class="text-text-tertiary">SSL configuration options will appear here.</p>
                </div>
              </div>

              <!-- Advanced Tab -->
              <div v-else class="text-text-secondary text-[13px]">
                <div v-if="form.type === 'oracle'" class="grid grid-cols-2 gap-x-6 gap-y-4">
                  <div>
                    <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                      >Connect Type</label
                    >
                    <select
                      v-model="form.oracleConnectType"
                      class="border-border text-text-primary bg-surface focus:border-primary focus:ring-primary/20 w-full cursor-pointer appearance-none rounded-lg border px-3 py-2 text-[13px] transition-all outline-none focus:ring-1"
                    >
                      <option value="serviceName">Service Name</option>
                      <option value="sid">SID</option>
                    </select>
                  </div>

                  <div>
                    <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                      >Role</label
                    >
                    <select
                      v-model="form.oracleRole"
                      class="border-border text-text-primary bg-surface focus:border-primary focus:ring-primary/20 w-full cursor-pointer appearance-none rounded-lg border px-3 py-2 text-[13px] transition-all outline-none focus:ring-1"
                    >
                      <option value="normal">Normal</option>
                      <option value="sysdba">SYSDBA</option>
                      <option value="sysoper">SYSOPER</option>
                    </select>
                  </div>
                </div>
                <div v-else class="flex h-40 items-center justify-center">
                  <p class="text-text-tertiary">Advanced settings will appear here.</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="border-border bg-muted flex items-center justify-between border-t px-5 py-3">
          <!-- Left: Test Connection -->
          <div class="flex items-center gap-3">
            <button
              id="btn-test-connection"
              class="border-border text-text-secondary hover:bg-hover hover:border-border-strong flex cursor-pointer items-center gap-2 rounded-lg border px-4 py-2 text-[13px] transition-colors"
              :disabled="testStatus === 'testing'"
              @click="handleTestConnection"
            >
              <Loader2 v-if="testStatus === 'testing'" :size="14" class="animate-spin" />
              <svg
                v-else
                class="h-3.5 w-3.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path
                  d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z"
                />
              </svg>
              <span>Test Connection</span>
            </button>
            <span class="flex items-center gap-1.5 text-[12px]">
              <span
                class="h-2 w-2 rounded-full"
                :class="{
                  'bg-success': testStatus === 'success',
                  'bg-text-tertiary': testStatus === 'ready',
                  'bg-warning animate-pulse': testStatus === 'testing',
                  'bg-danger': testStatus === 'error'
                }"
              ></span>
              <span class="text-text-secondary">
                {{
                  testStatus === 'ready'
                    ? 'Ready to test'
                    : testStatus === 'testing'
                      ? 'Testing...'
                      : testStatus === 'success'
                        ? 'Connection successful!'
                        : 'Connection failed'
                }}
              </span>
            </span>
          </div>

          <!-- Right: Cancel / Save -->
          <div class="flex items-center gap-2">
            <button
              class="border-border text-text-secondary hover:bg-hover cursor-pointer rounded-lg border px-4 py-2 text-[13px] transition-colors"
              @click="handleClose"
            >
              Cancel
            </button>
            <button
              id="btn-save-connection"
              class="bg-primary hover:bg-primary-hover text-text-inverse cursor-pointer rounded-lg px-4 py-2 text-[13px] font-medium shadow-sm transition-colors"
              @click="handleSave"
            >
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
