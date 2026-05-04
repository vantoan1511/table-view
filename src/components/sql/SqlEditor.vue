<script setup lang="ts">
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { useGridStore } from '@/stores/grid'
import { useSchemaStore } from '@/stores/schema'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { PostgreSQL, sql } from '@codemirror/lang-sql'
import { Compartment, EditorState, Prec } from '@codemirror/state'
import { keymap } from '@codemirror/view'
import { EditorView, basicSetup } from 'codemirror'
import { tags as t } from '@lezer/highlight'
import {
  Check,
  ChevronDown,
  Clock,
  Play
} from 'lucide-vue-next'
import { useDebounce } from '@/composables/useDebounce'
import { onMounted, ref, watch } from 'vue'
import ResultsGrid from './ResultsGrid.vue'

const gridStore = useGridStore()
const schemaStore = useSchemaStore()
const editorContainer = ref<HTMLElement>()
const activeResultTab = ref<'results' | 'messages'>('results')




let editorView: EditorView | null = null
const sqlCompartment = new Compartment()

const sqlHighlightStyle = HighlightStyle.define([
  { tag: t.keyword, color: '#818CF8', fontWeight: 'bold' },
  { tag: t.string, color: '#34D399' },
  { tag: t.number, color: '#FBBF24' },
  { tag: t.comment, color: '#6C7086', fontStyle: 'italic' },
  { tag: t.operator, color: '#CDD6F4' },
  { tag: t.meta, color: '#CDD6F4' },
  { tag: t.typeName, color: '#818CF8' },
  { tag: t.propertyName, color: '#CDD6F4' },
  { tag: t.className, color: '#CDD6F4' },
  { tag: t.labelName, color: '#CDD6F4' },
  { tag: t.namespace, color: '#CDD6F4' },
  { tag: t.macroName, color: '#CDD6F4' },
  { tag: t.literal, color: '#34D399' },
  { tag: t.bool, color: '#FBBF24' },
  { tag: t.null, color: '#FBBF24' },
  { tag: t.name, color: '#CDD6F4' },
  { tag: t.heading, color: '#CDD6F4', fontWeight: 'bold' },
  { tag: t.invalid, color: '#FB7185' },
])

/**
 * Extract the SQL statement at the current cursor position.
 * Splits the document by semicolons, maps each statement to its
 * character range, and returns the one the cursor falls within.
 */
const getQueryAtCursor = () : string => {
  if (!editorView) return ''
  const doc = editorView.state.doc.toString()
  const cursor = editorView.state.selection.main.head

  // Split into statements by semicolon, tracking each one's range
  const statements: { text: string; start: number; end: number }[] = []
  let offset = 0
  const parts = doc.split(';')

  for (let i = 0; i < parts.length; i++) {
    const raw = parts[i]
    if (!raw) continue;
    const end = offset + raw.length + (i < parts.length - 1 ? 1 : 0) // +1 for the ';'
    const trimmed = raw.trim()
    if (trimmed) {
      statements.push({ text: trimmed, start: offset, end })
    }
    offset = end
  }

  if (statements.length === 0) return ''

  // Find the statement whose range contains the cursor.
  // Use <= for end so that a cursor right after a ';' still matches that statement.
  for (const stmt of statements) {
    if (cursor >= stmt.start && cursor <= stmt.end) {
      return stmt.text
    }
  }

  // Fallback: return the last statement (cursor might be past everything)
  return statements[statements.length - 1]?.text ?? ''
}

const executeRun = () => {
  if (!editorView) return

  const selection = editorView.state.sliceDoc(
    editorView.state.selection.main.from,
    editorView.state.selection.main.to,
  )

  // Priority: selected text > statement at cursor > entire document
  const queryAtCursor = getQueryAtCursor()
  const query = selection.trim() || queryAtCursor || editorView.state.doc.toString()
  if (!query) return

  gridStore.runQuery(query)
  activeResultTab.value = 'results'
}

const handleRun = useDebounce(() => {
  if (gridStore.isLoading) return
  executeRun()
}, { delay: 300 })


onMounted(() => {
  if (!editorContainer.value) return

  const buildSqlExtension = () => {
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
    doc: '',
    extensions: [
      basicSetup,
      syntaxHighlighting(sqlHighlightStyle),
      sqlCompartment.of(buildSqlExtension()),
      Prec.highest(keymap.of([
        {
          key: 'Mod-Enter',
          run: () => {
            handleRun()
            return true
          },
        },
      ])),
      EditorView.theme({
        '&': {
          fontSize: '12px',
          fontFamily: 'var(--font-mono)',
          backgroundColor: 'var(--color-surface)',
          color: 'var(--color-text-primary)',
        },
        '.cm-content': {
          padding: '8px 0',
        },
        '.cm-gutters': {
          backgroundColor: 'var(--color-muted)',
          borderRight: '1px solid var(--color-border)',
          color: 'var(--color-text-tertiary)',
          fontSize: '11px',
        },
        '.cm-activeLine': {
          backgroundColor: 'var(--color-active)',
        },
        '.cm-activeLineGutter': {
          backgroundColor: 'var(--color-active)',
        },
        '&.cm-focused .cm-cursor': {
          borderLeftColor: 'var(--color-primary)',
        },
        '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
          backgroundColor: 'var(--color-primary-light)',
        },
      }, { dark: true }),
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
        <button class="px-3 py-1.5 text-[12px] font-medium transition-colors cursor-pointer"
          :class="activeResultTab === 'results' ? 'text-primary border-b-2 border-primary' : 'text-text-secondary hover:text-text-primary'"
          @click="activeResultTab = 'results'">
          Results
        </button>
        <button class="px-3 py-1.5 text-[12px] font-medium transition-colors cursor-pointer"
          :class="activeResultTab === 'messages' ? 'text-primary border-b-2 border-primary' : 'text-text-secondary hover:text-text-primary'"
          @click="activeResultTab = 'messages'">
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
          <button id="btn-run-query"
            class="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-hover disabled:opacity-70 disabled:cursor-not-allowed text-text-inverse rounded-lg text-[12px] font-medium cursor-pointer transition-colors shadow-sm"
            :disabled="gridStore.isLoading"
            @click="handleRun">
            <Loader2 v-if="gridStore.isLoading" :size="13" class="animate-spin" />
            <Play v-else :size="13" fill="currentColor" />
            {{ gridStore.isLoading ? 'Running...' : 'Run' }}
          </button>
          <button class="flex items-center gap-1 text-[11px] text-primary hover:text-primary-hover cursor-pointer">
            <ChevronDown :size="12" />
          </button>



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
          <div v-for="(msg, i) in gridStore.sqlMessages" :key="i" class="py-1" :class="{
            'text-success': msg.type === 'info',
            'text-danger': msg.type === 'error',
            'text-warning': msg.type === 'warning',
          }">
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
