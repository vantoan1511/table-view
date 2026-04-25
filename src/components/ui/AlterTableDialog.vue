<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { X, Plus, Trash2, Edit2, Check } from 'lucide-vue-next'
import { useGridStore } from '@/stores/grid'

const props = defineProps<{
  tableName: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'apply', operations: any[]): void
}>()

const gridStore = useGridStore()

interface ColumnDef {
  id: string
  name: string
  dataType: string
  nullable: boolean
  default: string | null
  _originalName?: string
  _isNew?: boolean
  _deleted?: boolean
  _editing?: boolean
}

const columns = ref<ColumnDef[]>([])
const loading = ref(true)
const activeTab = ref('Columns')
const tabs = ['Columns', 'Constraints', 'Indexes', 'Options', 'Comment']

onMounted(async () => {
  try {
    const cols = await gridStore.getTableColumns(props.tableName)
    columns.value = cols.map((c: any) => ({
      id: crypto.randomUUID(),
      name: c.name,
      dataType: c.dataType,
      nullable: c.nullable,
      default: c.default,
      _originalName: c.name,
      _isNew: false,
      _deleted: false,
      _editing: false
    }))
  } catch (err) {
    console.error('Failed to fetch columns:', err)
  } finally {
    loading.value = false
  }
})

const visibleColumns = computed(() => columns.value.filter(c => !c._deleted))

function addColumn() {
  columns.value.push({
    id: crypto.randomUUID(),
    name: 'new_column',
    dataType: 'varchar(255)',
    nullable: true,
    default: null,
    _isNew: true,
    _editing: true
  })
}

function removeColumn(col: ColumnDef) {
  col._deleted = true
}

function editColumn(col: ColumnDef) {
  col._editing = true
}

function saveColumn(col: ColumnDef) {
  col._editing = false
}

async function applyChanges() {
  const operations: any[] = []
  
  for (const col of columns.value) {
    if (col._deleted) {
      if (!col._isNew) {
        operations.push({ type: 'DROP_COLUMN', name: col._originalName })
      }
      continue
    }
    
    if (col._isNew) {
      operations.push({ 
        type: 'ADD_COLUMN', 
        name: col.name, 
        dataType: col.dataType, 
        nullable: col.nullable, 
        default: col.default 
      })
    } else if (col.name !== col._originalName) {
      operations.push({ 
        type: 'RENAME_COLUMN', 
        oldName: col._originalName, 
        newName: col.name 
      })
    }
  }

  emit('apply', operations)
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
    <div class="bg-surface border border-border rounded-xl shadow-2xl w-[700px] flex flex-col max-h-[85vh] overflow-hidden">
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-border">
        <h3 class="text-base font-semibold text-text-primary">Alter Table: {{ tableName }}</h3>
        <button @click="$emit('close')" class="text-text-tertiary hover:text-text-primary transition-colors">
          <X :size="18" />
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex px-5 border-b border-border overflow-x-auto hide-scrollbar">
        <button
          v-for="tab in tabs"
          :key="tab"
          @click="activeTab = tab"
          class="px-4 py-3 text-[13px] font-medium border-b-2 transition-colors whitespace-nowrap"
          :class="activeTab === tab 
            ? 'border-primary text-primary' 
            : 'border-transparent text-text-tertiary hover:text-text-secondary'"
        >
          {{ tab }}
        </button>
      </div>

      <!-- Body -->
      <div class="p-5 flex-1 overflow-y-auto">
        <div v-if="loading" class="flex justify-center py-10">
          <div class="w-6 h-6 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
        </div>

        <div v-else-if="activeTab === 'Columns'">
          <div class="border border-border rounded-lg overflow-hidden">
            <table class="w-full text-left border-collapse">
              <thead>
                <tr class="bg-muted border-b border-border text-[12px] text-text-secondary font-medium">
                  <th class="px-4 py-2 w-[30%]">Column Name</th>
                  <th class="px-4 py-2 w-[25%]">Data Type</th>
                  <th class="px-4 py-2 w-[15%]">Nullable</th>
                  <th class="px-4 py-2 w-[20%]">Default</th>
                  <th class="px-4 py-2 w-[10%] text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="col in visibleColumns" :key="col.id" class="border-b border-border/50 last:border-0 hover:bg-muted/50 text-[13px]">
                  
                  <td class="px-4 py-2">
                    <input v-if="col._editing" v-model="col.name" type="text" class="w-full bg-surface border border-primary/50 focus:border-primary rounded px-2 py-1 outline-none" />
                    <span v-else class="text-text-primary">{{ col.name }}</span>
                  </td>

                  <td class="px-4 py-2">
                    <input v-if="col._editing && col._isNew" v-model="col.dataType" type="text" class="w-full bg-surface border border-primary/50 focus:border-primary rounded px-2 py-1 outline-none" />
                    <span v-else class="text-text-secondary">{{ col.dataType }}</span>
                  </td>

                  <td class="px-4 py-2">
                    <select v-if="col._editing && col._isNew" v-model="col.nullable" class="w-full bg-surface border border-primary/50 focus:border-primary rounded px-2 py-1 outline-none">
                      <option :value="true">YES</option>
                      <option :value="false">NO</option>
                    </select>
                    <span v-else class="text-text-secondary">{{ col.nullable ? 'YES' : 'NO' }}</span>
                  </td>

                  <td class="px-4 py-2">
                    <input v-if="col._editing && col._isNew" v-model="col.default" type="text" placeholder="—" class="w-full bg-surface border border-primary/50 focus:border-primary rounded px-2 py-1 outline-none" />
                    <span v-else class="text-text-secondary">{{ col.default || '—' }}</span>
                  </td>

                  <td class="px-4 py-2 text-right">
                    <div class="flex items-center justify-end gap-2">
                      <button v-if="col._editing" @click="saveColumn(col)" class="text-success hover:text-success/80 transition-colors" title="Save">
                        <Check :size="14" />
                      </button>
                      <button v-else @click="editColumn(col)" class="text-text-tertiary hover:text-primary transition-colors" title="Edit">
                        <Edit2 :size="14" />
                      </button>
                      
                      <button @click="removeColumn(col)" class="text-text-tertiary hover:text-danger transition-colors" title="Delete">
                        <Trash2 :size="14" />
                      </button>
                    </div>
                  </td>

                </tr>
                <tr v-if="visibleColumns.length === 0">
                  <td colspan="5" class="px-4 py-6 text-center text-text-tertiary text-[13px]">
                    No columns found.
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="mt-4">
            <button @click="addColumn" class="flex items-center gap-1.5 px-3 py-1.5 border border-primary/30 text-primary hover:bg-primary/10 rounded-lg text-[13px] font-medium transition-colors">
              <Plus :size="14" />
              <span>Add Column</span>
            </button>
          </div>
        </div>
        
        <div v-else class="flex items-center justify-center h-40 text-text-tertiary text-[13px]">
          {{ activeTab }} configuration is not available yet.
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-5 py-4 border-t border-border bg-muted/30">
        <button 
          @click="$emit('close')" 
          class="px-4 py-2 text-[13px] font-medium text-text-secondary hover:text-text-primary transition-colors border border-border rounded-lg bg-surface hover:bg-hover"
        >
          Cancel
        </button>
        <button 
          @click="applyChanges" 
          class="px-4 py-2 text-[13px] font-medium text-white bg-primary hover:bg-primary-hover transition-colors rounded-lg shadow-sm"
          :disabled="loading"
        >
          Apply Changes
        </button>
      </div>
    </div>
  </div>
</template>
