<script setup lang="ts">
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { useGridStore } from '@/stores/grid'
import { useSchemaStore } from '@/stores/schema'
import { PostgreSQL, sql } from '@codemirror/lang-sql'
import { Compartment, EditorState } from '@codemirror/state'
import { keymap } from '@codemirror/view'
import { EditorView, basicSetup } from 'codemirror'
import {
  Check,
  ChevronDown,
  Clock,
  Play
} from 'lucide-vue-next'
import { onMounted, ref, watch } from 'vue'
import ResultsGrid from './ResultsGrid.vue'

const gridStore = useGridStore()
const schemaStore = useSchemaStore()
const editorContainer = ref<HTMLElement>()
const activeResultTab = ref<'results' | 'messages'>('results')

const showLimitMenu = ref(false)
const limitMenuPos = ref({ x: 0, y: 0 })

function toggleLimitMenu(e: MouseEvent) {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  limitMenuPos.value = { x: rect.left, y: rect.bottom + 5 }
  showLimitMenu.value = !showLimitMenu.value
}

function setSqlLimit(limit: number) {
  gridStore.sqlLimit = limit
  showLimitMenu.value = false
}
let editorView: EditorView | null = null
const sqlCompartment = new Compartment()

const initialQuery = `SELECT u.id,
       u.name,
       u.email,
       u.created_at,
       r.name AS role_name
FROM users u
LEFT JOIN roles r ON r.id = u.role_id
WHERE u.status = 'active'
ORDER BY u.created_at DESC
LIMIT 100;`

function handleRun() {
  if (!editorView) return
  const query = editorView.state.doc.toString()
  gridStore.runQuery(query)
  activeResultTab.value = 'results'
}

onMounted(() => {
  if (!editorContainer.value) return

  function buildSqlExtension() {
    // Build a schema map from current store data: { tableName: [col, col, ...] }
    const schemaMap: Record<string, string[]> = {}
    for (const table of schemaStore.schema.tables) {
      schemaMap[table.name] = [] // column names can be added when we fetch them later
    }
    for (const view of schemaStore.schema.views) {
      schemaMap[view.name] = []
    }
    return sql({ dialect: PostgreSQL, schema: schemaMap })
  }

  const state = EditorState.create({
    doc: initialQuery,
    extensions: [
      basicSetup,
      sqlCompartment.of(buildSqlExtension()),
      keymap.of([
        {
          key: 'Mod-Enter',
          run: () => {
            handleRun()
            return true
          },
        },
      ]),
      EditorView.theme({
        '&': {
          fontSize: '12px',
          fontFamily: 'var(--font-mono)',
        },
        '.cm-content': {
          padding: '8px 0',
        },
        '.cm-gutters': {
          backgroundColor: '#F9FAFB',
          borderRight: '1px solid #E5E7EB',
          color: '#9CA3AF',
          fontSize: '11px',
        },
        '.cm-activeLine': {
          backgroundColor: '#F0F4FF',
        },
        '.cm-activeLineGutter': {
          backgroundColor: '#EEF2FF',
        },
        '&.cm-focused .cm-cursor': {
          borderLeftColor: '#4F46E5',
        },
        '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
          backgroundColor: '#E0E7FF',
        },
      }),
      EditorView.lineWrapping,
    ],
  })

  editorView = new EditorView({
    state,
    parent: editorContainer.value,
  })

  // Re-configure SQL extension when schema loads
  watch(
    () => schemaStore.schema,
    () => {
      if (!editorView) return
      editorView.dispatch({
        effects: sqlCompartment.reconfigure(buildSqlExtension()),
      })
    },
    { deep: true }
  )
})
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0 bg-surface">
    <!-- Results / Messages tabs toggle -->
    <div class="flex items-center border-b border-border bg-muted shrink-0">
      <div class="px-4 py-1.5 text-[12px] font-semibold text-text-secondary uppercase tracking-wider">
        Query Results
      </div>
      <div class="flex items-center ml-auto border-l border-border">
        <button
          class="px-3 py-1.5 text-[12px] font-medium transition-colors cursor-pointer"
          :class="activeResultTab === 'results' ? 'text-primary border-b-2 border-primary' : 'text-text-secondary hover:text-text-primary'"
          @click="activeResultTab = 'results'"
        >
          Results
        </button>
        <button
          class="px-3 py-1.5 text-[12px] font-medium transition-colors cursor-pointer"
          :class="activeResultTab === 'messages' ? 'text-primary border-b-2 border-primary' : 'text-text-secondary hover:text-text-primary'"
          @click="activeResultTab = 'messages'"
        >
          Messages
        </button>
      </div>
    </div>

    <!-- Split Content -->
    <div class="flex flex-1 min-h-0">
      <!-- Editor Pane -->
      <div class="flex flex-col w-1/2 border-r border-border min-h-0">
        <div ref="editorContainer" class="flex-1 overflow-auto min-h-0"></div>

        <!-- Run Bar -->
        <div class="flex items-center gap-3 px-3 py-1.5 border-t border-border bg-muted">
          <button
            id="btn-run-query"
            class="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-hover text-text-inverse rounded-lg text-[12px] font-medium cursor-pointer transition-colors shadow-sm"
            @click="handleRun"
          >
            <Play :size="13" fill="currentColor" />
            Run
          </button>
          <button class="flex items-center gap-1 text-[11px] text-primary hover:text-primary-hover cursor-pointer">
            <ChevronDown :size="12" />
          </button>

          <div class="flex items-center gap-1.5 text-[11px] text-text-secondary ml-2 relative">
            <span>Limit</span>
            <button 
              class="flex items-center gap-1 px-1.5 py-0.5 border border-border rounded text-[11px] hover:bg-hover cursor-pointer"
              @click="toggleLimitMenu"
            >
              {{ gridStore.sqlLimit }}
              <ChevronDown :size="10" />
            </button>
            <ContextMenu :show="showLimitMenu" :x="limitMenuPos.x" :y="limitMenuPos.y" @close="showLimitMenu = false">
              <button v-for="limit in [100, 500, 1000, 5000, 0]" :key="limit"
                class="w-full flex items-center justify-between px-3 py-1.5 hover:bg-hover text-[12px]"
                @click="setSqlLimit(limit)">
                <span>{{ limit === 0 ? 'No Limit' : limit }}</span>
                <Check v-if="gridStore.sqlLimit === limit" :size="14" class="text-primary" />
              </button>
            </ContextMenu>
          </div>

          <div class="flex items-center gap-1 text-[11px] text-text-secondary ml-auto">
            <Clock :size="12" class="text-success" />
            <span>{{ gridStore.sqlExecutionTime }} ms</span>
          </div>
        </div>
      </div>

      <!-- Results Pane -->
      <div class="flex-1 overflow-auto min-h-0">
        <div v-if="activeResultTab === 'results'">
          <ResultsGrid />
        </div>
        <div v-else class="p-3 text-[12px] font-(--font-mono) text-text-secondary">
          <div
            v-for="(msg, i) in gridStore.sqlMessages"
            :key="i"
            class="py-1"
            :class="{
              'text-success': msg.type === 'info',
              'text-danger': msg.type === 'error',
              'text-warning': msg.type === 'warning',
            }"
          >
            {{ msg.text }}
          </div>
          <div v-if="gridStore.sqlMessages.length === 0" class="text-text-tertiary italic">
            No messages yet. Run a query to see output.
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
