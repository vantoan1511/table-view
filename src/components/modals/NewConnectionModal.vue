<script setup lang="ts">
import DbIcon from '@/components/icons/DbIcon.vue';
import ColorPicker from '@/components/ui/ColorPicker.vue';
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue';
import Select from 'primevue/select';

import { useConnectionsStore } from '@/stores/connections';
import { useErrorStore } from '@/stores/error';

import { ConnectionColor, DbType, OracleConnectType, OracleRole, type Connection } from '@/types';
import { CircleHelp, Download, Eye, EyeOff, Loader2, Send, X } from 'lucide-vue-next';
import { reactive, ref, watch } from 'vue';

import { DB_TYPES } from '@/lib/dbTypes';
import { BridgeService } from '@/services/bridge';
import { NativeService } from '@/services/nativeService';
import { decryptPassword } from '@/utils/crypto';

const sslModeOptions = [
  { label: 'Prefer (Auto-detect SSL & fallback to plain TCP)', value: 'prefer' },
  { label: 'Require (Mandate SSL connection)', value: 'require' },
  { label: 'Disable (Force unencrypted connection)', value: 'disable' },
  { label: 'Verify CA (Require SSL & verify CA cert)', value: 'verify-ca' },
  { label: 'Verify Full (Require SSL & verify CA and host)', value: 'verify-full' }
];

const oracleConnectTypeOptions = [
  { label: 'Service Name', value: 'serviceName' },
  { label: 'SID', value: 'sid' }
];

const oracleRoleOptions = [
  { label: 'Normal', value: 'normal' },
  { label: 'SYSDBA', value: 'sysdba' },
  { label: 'SYSOPER', value: 'sysoper' }
];

const connectionsStore = useConnectionsStore();
const errorStore = useErrorStore();

const activeTab = ref<'general' | 'ssl' | 'advanced'>('general');
const showPassword = ref(false);
const testStatus = ref<'ready' | 'testing' | 'success' | 'error'>('ready');
const importError = ref('');
const showImportConfirm = ref(false);
const pendingImportData = ref<any>(null);

interface ConnectionForm extends Omit<Connection, 'id' | 'isConnected'> {
  ssl: boolean;
  sslMode: string;
}

const form = reactive<ConnectionForm>({
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
  ssl: false,
  sslMode: 'prefer',
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
        savePassword: conn.savePassword ?? false,
        displayAllDatabases: conn.displayAllDatabases ?? false,
        ssl: conn.ssl ?? false,
        sslMode: conn.sslMode || 'prefer',
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
        ssl: false,
        sslMode: 'prefer',
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
const confirmImport = () => {
  if (!pendingImportData.value) return;

  const conn = pendingImportData.value;
  const importedType = (conn.type || 'postgresql') as DbType;

  Object.assign(form, {
    name: conn.name || '',
    type: importedType,
    host: conn.host || 'localhost',
    port: conn.port || portDefaults[importedType] || 5432,
    database: conn.database || '',
    username: conn.username || conn.user || '',
    password: conn.password ? decryptPassword(conn.password) : '',
    color: conn.color || 'indigo',
    tags: conn.tags || '',
    savePassword: conn.savePassword ?? false,
    displayAllDatabases: conn.displayAllDatabases ?? false,
    ssl: conn.ssl ?? false,
    sslMode: conn.sslMode || 'prefer',
    oracleConnectType: (conn.oracleConnectType || 'serviceName') as OracleConnectType,
    oracleRole: (conn.oracleRole || 'normal') as OracleRole
  });

  showImportConfirm.value = false;
  pendingImportData.value = null;
};

const cancelImport = () => {
  showImportConfirm.value = false;
  pendingImportData.value = null;
};

const handleImportConnection = async () => {
  importError.value = '';
  try {
    let fileContent = '';

    if (window.NL_PORT) {
      const filePaths = await NativeService.os.showOpenDialog('Import Connection Profile', {
        filters: [{ name: 'JSON files', extensions: ['json'] }],
        multiSelections: false
      });
      if (!filePaths || filePaths.length === 0 || !filePaths[0]) return;
      const read = await NativeService.fs.readFile(filePaths[0]);
      if (!read) return;
      fileContent = read;
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

    pendingImportData.value = conn;
    showImportConfirm.value = true;
  } catch (err: any) {
    importError.value = err.message || 'Failed to import connection';
    console.error('Import failed:', err);
  }
};

const handleTestConnection = async () => {
  testStatus.value = 'testing';

  if (window.NL_PORT) {
    try {
      await BridgeService.request('dbBridge.testConnection', 'dbBridge.testConnectionResult', {
        config: form
      });
      testStatus.value = 'success';
      setTimeout(() => {
        testStatus.value = 'ready';
      }, 2000);
    } catch (err: any) {
      testStatus.value = 'error';
      errorStore.showError('Connection Test Failed', err.message || String(err));
      console.error('Connection failed:', err);
    }
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
          <Button rounded variant="text" severity="secondary" @click="handleClose">
            <template #icon>
              <X class="h-4 w-4" />
            </template>
          </Button>
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
              <Button
                v-for="db in DB_TYPES"
                :key="db.key"
                fluid
                class="mb-0.5 w-full justify-start!"
                size="small"
                :variant="form.type === db.key ? 'link' : 'text'"
                :severity="form.type === db.key ? 'primary' : 'secondary'"
                @click="selectDbType(db.key)"
              >
                <DbIcon :type="db.key" size="18" />
                <span>{{ db.label }}</span>
              </Button>
            </div>

            <!-- Import Connection -->
            <div class="border-border mt-2 border-t px-3 pt-2">
              <Button
                variant="text"
                severity="secondary"
                size="small"
                class="w-full justify-start!"
                @click="handleImportConnection"
              >
                <Download class="h-4 w-4" />
                <span>Import</span>
              </Button>
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
                class="-mb-px cursor-pointer border-none bg-transparent px-3 py-2 text-[13px] font-medium capitalize transition-colors"
                :class="
                  activeTab === tab
                    ? 'text-primary border-primary rounded-none border-b-2'
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
                      class="text-text-tertiary hover:text-text-secondary absolute top-1/2 right-1 flex -translate-y-1/2 cursor-pointer items-center justify-center border-none bg-transparent p-1"
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
                    <span class="inline-flex shrink-0">
                      <CircleHelp
                        v-tooltip.top="
                          'If enabled, loads all databases from the server. Otherwise, only loads the configured database. Schemas are always fully loaded.'
                        "
                        :size="13"
                        class="text-text-tertiary cursor-help"
                      />
                    </span>
                  </div>
                </div>
              </div>

              <!-- SSL Tab -->
              <div v-else-if="activeTab === 'ssl'" class="grid grid-cols-2 gap-x-6 gap-y-4">
                <div
                  class="border-border bg-surface col-span-2 flex items-center justify-between rounded-lg border p-3.5"
                >
                  <div>
                    <label class="text-text-primary block text-[13px] font-medium"
                      >Enable SSL / TLS Encryption</label
                    >
                    <p class="text-text-tertiary mt-0.5 text-[11px]">
                      Encrypt connection between Table View and your database server.
                    </p>
                  </div>
                  <ToggleSwitch v-model="form.ssl" />
                </div>

                <div v-if="form.type === 'postgresql'" class="col-span-2">
                  <label class="text-text-secondary mb-1.5 block text-[12px] font-medium">
                    SSL Mode (PostgreSQL)
                  </label>
                  <Select
                    v-model="form.sslMode"
                    :options="sslModeOptions"
                    optionLabel="label"
                    optionValue="value"
                    class="w-full text-[13px]"
                  />
                  <p class="text-text-tertiary mt-1.5 text-[11px]">
                    "Prefer" automatically negotiates SSL for cloud databases (Supabase, Neon, AWS
                    RDS, PgBouncer) while working with local databases.
                  </p>
                </div>
              </div>

              <!-- Advanced Tab -->
              <div v-else class="text-text-secondary text-[13px]">
                <div v-if="form.type === 'oracle'" class="grid grid-cols-2 gap-x-6 gap-y-4">
                  <div>
                    <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                      >Connect Type</label
                    >
                    <Select
                      v-model="form.oracleConnectType"
                      :options="oracleConnectTypeOptions"
                      optionLabel="label"
                      optionValue="value"
                      class="w-full text-[13px]"
                    />
                  </div>

                  <div>
                    <label class="text-text-secondary mb-1.5 block text-[12px] font-medium"
                      >Role</label
                    >
                    <Select
                      v-model="form.oracleRole"
                      :options="oracleRoleOptions"
                      optionLabel="label"
                      optionValue="value"
                      class="w-full text-[13px]"
                    />
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
            <Button
              id="btn-test-connection"
              variant="text"
              severity="secondary"
              size="small"
              :loading="testStatus === 'testing'"
              @click="handleTestConnection"
            >
              <Send v-if="testStatus !== 'testing'" :size="13" />
              <Loader2 v-else :size="13" class="text-text-tertiary shrink-0 animate-spin" />
              <span>{{ testStatus === 'testing' ? 'Testing...' : 'Test Connection' }}</span>
            </Button>
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
            <Button variant="text" severity="secondary" size="small" @click="handleClose">
              Cancel
            </Button>
            <Button id="btn-save-connection" size="small" @click="handleSave">
              Save Connection
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- Confirm dialog -->
    <ConfirmDialog
      v-if="showImportConfirm"
      title="Import Connection"
      message="This will overwrite your current connection settings. Do you want to proceed?"
      confirm-label="Overwrite"
      cancel-label="Cancel"
      variant="warning"
      @confirm="confirmImport"
      @cancel="cancelImport"
    />
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
