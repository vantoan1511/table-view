<script setup lang="ts">
// Components
import Button from '@/components/ui/Button.vue';
import ResizeHandle from '@/components/ui/ResizeHandle.vue';
import ResultsGrid from './ResultsGrid.vue';

// Composables
import { useDebounce } from '@/composables/useDebounce';
import { useSqlEditor } from '@/composables/useSqlEditor';

// Pinia stores
import { useGridStore } from '@/stores/grid';
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';

// Types
import type { Tab } from '@/types';

// External libraries
import { PostgreSQL, sql } from '@codemirror/lang-sql';
import { syntaxHighlighting } from '@codemirror/language';
import { Compartment, EditorState, Prec } from '@codemirror/state';
import { keymap } from '@codemirror/view';
import { EditorView, basicSetup } from 'codemirror';
import { Clock, Download, Play, Save } from 'lucide-vue-next';
import { onMounted, ref, watch } from 'vue';

// Other imports
import { editorTheme, sqlHighlightStyle } from '@/lib/editorConfig';

const props = defineProps<{
  tab: Tab;
}>();

const gridStore = useGridStore();
const schemaStore = useSchemaStore();
const tabsStore = useTabsStore();

const { activeResultTab, executeRun, saveQuery, exportQuery } = useSqlEditor(props);

const editorContainer = ref<HTMLElement>();
const editorWidth = ref(600);

let editorView: EditorView | null = null;
const sqlCompartment = new Compartment();

const buildSqlExtension = () => {
  const schemaMap: Record<string, string[]> = {};
  const connId = props.tab.connectionId;
  const connSchema = connId ? schemaStore.schemasByConnection[connId] : undefined;
  const source = connSchema ?? schemaStore.schema;
  for (const table of source.tables) {
    schemaMap[table.name] = [];
  }
  for (const view of source.views) {
    schemaMap[view.name] = [];
  }
  return sql({ dialect: PostgreSQL, schema: schemaMap });
};

const handleRun = useDebounce(
  () => {
    if (gridStore.isLoading) return;
    executeRun(editorView);
  },
  { delay: 300 }
);

const initEditor = () => {
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }
  if (!editorContainer.value) return;

  const state = EditorState.create({
    doc: props.tab.query || '',
    extensions: [
      basicSetup,
      syntaxHighlighting(sqlHighlightStyle),
      sqlCompartment.of(buildSqlExtension()),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          tabsStore.updateTabQuery(props.tab.id, update.state.doc.toString());
        }
      }),
      Prec.highest(
        keymap.of([
          {
            key: 'Mod-Enter',
            run: () => {
              handleRun();
              return true;
            }
          },
          {
            key: 'Mod-s',
            run: () => {
              saveQuery();
              return true;
            }
          }
        ])
      ),
      editorTheme,
      EditorView.lineWrapping
    ]
  });

  editorView = new EditorView({
    state,
    parent: editorContainer.value
  });
};

onMounted(() => {
  initEditor();

  watch(
    () =>
      props.tab.connectionId
        ? schemaStore.schemasByConnection[props.tab.connectionId]
        : schemaStore.schema,
    () => {
      if (!editorView) return;
      editorView.dispatch({
        effects: sqlCompartment.reconfigure(buildSqlExtension())
      });
    },
    { deep: true }
  );
});
</script>

<template>
  <div class="bg-surface flex min-h-0 flex-1 flex-col">
    <!-- Split Content -->
    <div class="flex min-h-0 flex-1">
      <!-- Editor Pane -->
      <div
        class="border-border flex min-h-0 flex-col border-r"
        :style="{ width: editorWidth + 'px' }"
      >
        <div ref="editorContainer" class="min-h-0 flex-1 overflow-auto"></div>

        <!-- Run Bar -->
        <div class="border-border bg-muted flex items-center gap-2 border-t px-3 py-1.5">
          <Button
            id="btn-run-query"
            variant="primary"
            size="sm"
            :loading="gridStore.isLoading"
            :icon="Play"
            @click="handleRun"
          >
            {{ gridStore.isLoading ? 'Running...' : 'Run' }}
          </Button>

          <Button
            variant="secondary"
            size="sm"
            :icon="Save"
            @click="saveQuery"
            title="Save (Ctrl+S)"
          />

          <Button
            variant="secondary"
            size="sm"
            :icon="Download"
            @click="exportQuery"
            title="Export to .sql file"
          >
            Export
          </Button>

          <div class="text-text-secondary ml-auto flex items-center gap-1 text-[11px]">
            <Clock :size="12" class="text-success" />
            <span>{{ gridStore.sqlExecutionTime }} ms</span>
          </div>
        </div>
      </div>

      <!-- Resize Handle -->
      <ResizeHandle orientation="horizontal" v-model="editorWidth" />

      <!-- Results Pane -->
      <div class="bg-surface flex min-h-0 flex-1 flex-col overflow-hidden">
        <!-- Results / Messages Header -->
        <div
          class="bg-muted border-border flex h-9 shrink-0 items-center justify-between border-b px-3 select-none"
        >
          <div class="flex h-full items-center">
            <button
              class="h-full border-b-2 px-3 text-[12px] font-medium whitespace-nowrap transition-colors"
              :class="
                activeResultTab === 'results'
                  ? 'border-primary text-primary bg-primary/5'
                  : 'text-text-secondary hover:text-text-primary hover:bg-hover/50 border-transparent'
              "
              @click="activeResultTab = 'results'"
            >
              Results
            </button>
            <button
              class="h-full border-b-2 px-3 text-[12px] font-medium whitespace-nowrap transition-colors"
              :class="
                activeResultTab === 'messages'
                  ? 'border-primary text-primary bg-primary/5'
                  : 'text-text-secondary hover:text-text-primary hover:bg-hover/50 border-transparent'
              "
              @click="activeResultTab = 'messages'"
            >
              Messages
            </button>
          </div>

          <!-- Metadata info -->
          <div class="flex items-center gap-3">
            <div class="text-text-tertiary text-[11px] font-medium">
              <span v-if="activeResultTab === 'results' && gridStore.sqlRows.length > 0">
                {{ gridStore.sqlRows.length }} rows
              </span>
            </div>
          </div>
        </div>

        <!-- Content Area -->
        <div class="min-h-0 flex-1 overflow-hidden">
          <div v-if="activeResultTab === 'results'" class="h-full">
            <ResultsGrid />
          </div>
          <div
            v-else
            class="text-text-secondary h-full overflow-auto p-3 text-[12px] font-(--font-mono)"
          >
            <div
              v-for="(msg, i) in gridStore.sqlMessages"
              :key="i"
              class="py-1"
              :class="{
                'text-success': msg.type === 'info',
                'text-danger': msg.type === 'error',
                'text-warning': msg.type === 'warning'
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
  </div>
</template>
