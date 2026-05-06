<script setup lang="ts">
import ResizeHandle from '@/components/ui/ResizeHandle.vue'
import { useDebounce } from '@/composables/useDebounce'
import { useGridStore } from '@/stores/grid'
import { useSchemaStore } from '@/stores/schema'
import { useTabsStore } from '@/stores/tabs'
import type { Tab } from '@/types'
import { PostgreSQL, sql } from '@codemirror/lang-sql'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Compartment, EditorState, Prec } from '@codemirror/state'
import { keymap } from '@codemirror/view'
import { tags as t } from '@lezer/highlight'
import { EditorView, basicSetup } from 'codemirror'
import {
  Clock,
  Download,
  Loader2,
  Play,
  Save
} from 'lucide-vue-next'
import { onMounted, ref, watch } from 'vue'
import ResultsGrid from './ResultsGrid.vue'

const props = defineProps<{
  tab: Tab
}>()

const gridStore = useGridStore()
const schemaStore = useSchemaStore()
const tabsStore = useTabsStore()
const editorContainer = ref<HTMLElement>()
const activeResultTab = ref<'results' | 'messages'>('results')
const editorWidth = ref(600) // Initial width for editor pane

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

const buildSqlExtension = () => {
  const schemaMap: Record<string, string[]> = {}
  const connId = props.tab.connectionId
  const connSchema = connId ? schemaStore.schemasByConnection[connId] : undefined
  const source = connSchema ?? schemaStore.schema
  for (const table of source.tables) {
    schemaMap[table.name] = []
  }
  for (const view of source.views) {
    schemaMap[view.name] = []
  }
  return sql({ dialect: PostgreSQL, schema: schemaMap })
}

/**
 * Extract the SQL statement at the current cursor position.
 */
const getQueryAtCursor = (): string => {
  if (!editorView) return ''
  const doc = editorView.state.doc.toString()
  const cursor = editorView.state.selection.main.head

  const statements: { text: string; start: number; end: number }[] = []
  let offset = 0
  const parts = doc.split(';')

  for (let i = 0; i < parts.length; i++) {
    const raw = parts[i]
    if (raw === undefined) continue

    const end = offset + raw.length + (i < parts.length - 1 ? 1 : 0)
    const trimmed = raw.trim()
    if (trimmed) {
      statements.push({ text: trimmed, start: offset, end })
    }
    offset = end
  }

  if (statements.length === 0) return ''

  for (const stmt of statements) {
    if (cursor >= stmt.start && cursor <= stmt.end) {
      return stmt.text
    }
  }

  return statements[statements.length - 1]?.text ?? ''
}

const executeRun = () => {
  if (!editorView) return

  const selection = editorView.state.sliceDoc(
    editorView.state.selection.main.from,
    editorView.state.selection.main.to,
  )

  const queryAtCursor = getQueryAtCursor()
  const query = selection.trim() || queryAtCursor || editorView.state.doc.toString()
  if (!query) return

  gridStore.runQuery(query, gridStore.sqlLimit, props.tab.connectionId)
  activeResultTab.value = 'results'
}

const handleRun = useDebounce(() => {
  if (gridStore.isLoading) return
  executeRun()
}, { delay: 300 })


const initEditor = () => {
  if (editorView) {
    editorView.destroy()
    editorView = null
  }
  if (!editorContainer.value) return

  const state = EditorState.create({
    doc: props.tab.query || '',
    extensions: [
      basicSetup,
      syntaxHighlighting(sqlHighlightStyle),
      sqlCompartment.of(buildSqlExtension()),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          tabsStore.updateTabQuery(props.tab.id, update.state.doc.toString())
        }
      }),
      Prec.highest(keymap.of([
        {
          key: 'Mod-Enter',
          run: () => {
            handleRun()
            return true
          },
        },
        {
          key: 'Mod-s',
          run: () => {
            tabsStore.saveSqlTab(props.tab.id)
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
}

onMounted(() => {
  initEditor()

  // Re-configure SQL extension when schema for this connection loads
  watch(
    () => props.tab.connectionId ? schemaStore.schemasByConnection[props.tab.connectionId] : schemaStore.schema,
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
      <div class="flex flex-col border-r border-border min-h-0" :style="{ width: editorWidth + 'px' }">
        <div ref="editorContainer" class="flex-1 overflow-auto min-h-0"></div>

        <!-- Run Bar -->
        <div class="flex items-center gap-3 px-3 py-1.5 border-t border-border bg-muted">
          <button id="btn-run-query"
            class="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-hover disabled:opacity-70 disabled:cursor-not-allowed text-text-inverse rounded-lg text-[12px] font-medium cursor-pointer transition-colors shadow-sm"
            :disabled="gridStore.isLoading" @click="handleRun">
            <Loader2 v-if="gridStore.isLoading" :size="13" class="animate-spin" />
            <Play v-else :size="13" fill="currentColor" />
            {{ gridStore.isLoading ? 'Running...' : 'Run' }}
          </button>

          <button
            class="flex items-center gap-1.5 px-2.5 py-1.5 bg-surface border border-border hover:border-border-strong text-text-secondary rounded-lg text-[12px] cursor-pointer transition-colors"
            @click="tabsStore.saveSqlTab(tab.id)" title="Save (Ctrl+S)">
            <Save :size="13" />
          </button>

          <button
            class="flex items-center gap-1.5 px-2.5 py-1.5 bg-surface border border-border hover:border-border-strong text-text-secondary rounded-lg text-[12px] cursor-pointer transition-colors"
            @click="tabsStore.exportSqlTab(tab.id)" title="Export to .sql file">
            <Download :size="13" />
            <span>Export</span>
          </button>

          <div class="flex items-center gap-1 border-l border-border pl-3 ml-1 h-5">
            <span class="text-[11px] text-text-tertiary">Limit:</span>
            <select v-model="gridStore.sqlLimit"
              class="bg-transparent border-none outline-none text-[11px] text-text-primary cursor-pointer hover:text-primary">
              <option :value="100">100</option>
              <option :value="200">200</option>
              <option :value="500">500</option>
              <option :value="1000">1000</option>
              <option :value="5000">5000</option>
            </select>
          </div>

          <div class="flex items-center gap-1 text-[11px] text-text-secondary ml-auto">
            <Clock :size="12" class="text-success" />
            <span>{{ gridStore.sqlExecutionTime }} ms</span>
          </div>
        </div>
      </div>

      <!-- Resize Handle -->
      <ResizeHandle orientation="horizontal" v-model="editorWidth" />

      <!-- Results Pane -->
      <div class="flex-1 overflow-auto min-h-0">
        <div v-if="activeResultTab === 'results'" class="h-full">
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
