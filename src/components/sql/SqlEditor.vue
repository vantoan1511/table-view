<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useGridStore } from '@/stores/grid'
import { useTabsStore } from '@/stores/tabs'
import { EditorView, basicSetup } from 'codemirror'
import { EditorState } from '@codemirror/state'
import { sql, PostgreSQL } from '@codemirror/lang-sql'
import { keymap } from '@codemirror/view'
import ResultsGrid from './ResultsGrid.vue'
import {
  Play,
  ChevronDown,
  Clock,
  X,
  Plus,
} from 'lucide-vue-next'

const gridStore = useGridStore()
const tabsStore = useTabsStore()
const editorContainer = ref<HTMLElement>()
const activeResultTab = ref<'results' | 'messages'>('results')
let editorView: EditorView | null = null

onMounted(() => {
  if (!editorContainer.value) return

  const initialDoc = tabsStore.activeTab?.query ?? ''

  const state = EditorState.create({
    doc: initialDoc,
    extensions: [
      basicSetup,
      sql({ dialect: PostgreSQL }),
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
})

function handleRun() {
  if (!editorView) return
  const query = editorView.state.doc.toString()
  gridStore.runQuery(query)
  activeResultTab.value = 'results'
}
</script>

<template>
  <div class="flex flex-col border-t border-border bg-surface" style="height: 320px">
    <!-- SQL Tab Strip -->
    <div class="flex items-center border-b border-border bg-muted">
      <div class="flex items-center gap-0.5 px-2 py-1">
        <button class="flex items-center gap-1.5 px-2.5 py-1 bg-surface border border-border rounded-t-md text-[12px] font-medium text-text-primary -mb-px">
          SQL Editor
          <X :size="12" class="text-text-tertiary hover:text-text-secondary cursor-pointer" />
        </button>
        <button class="flex items-center justify-center w-6 h-6 rounded text-text-tertiary hover:text-text-secondary hover:bg-hover cursor-pointer">
          <Plus :size="13" />
        </button>
      </div>

      <!-- Results / Messages tabs -->
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

          <div class="flex items-center gap-1.5 text-[11px] text-text-secondary ml-2">
            <span>Limit</span>
            <button class="flex items-center gap-1 px-1.5 py-0.5 border border-border rounded text-[11px] hover:bg-hover cursor-pointer">
              100
              <ChevronDown :size="10" />
            </button>
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
        <div v-else class="p-3 text-[12px] font-[var(--font-mono)] text-text-secondary">
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
