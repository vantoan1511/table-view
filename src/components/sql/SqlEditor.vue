<script setup lang="ts">
import ResizeHandle from '@/components/ui/ResizeHandle.vue';
import DropdownMenu, { type DropdownValue } from '@/components/ui/DropdownMenu.vue';
import { useDebounce } from '@/composables/useDebounce';
import { useSqlEditor } from '@/composables/useSqlEditor';
import { editorTheme, sqlHighlightStyle } from '@/lib/editorConfig';
import { useGridStore } from '@/stores/grid';
import { useSchemaStore } from '@/stores/schema';
import { useTabsStore } from '@/stores/tabs';
import type { Tab } from '@/types';
import { PostgreSQL, sql } from '@codemirror/lang-sql';
import { syntaxHighlighting } from '@codemirror/language';
import { Compartment, EditorState, Prec } from '@codemirror/state';
import { keymap } from '@codemirror/view';
import { EditorView, basicSetup } from 'codemirror';
import { Clock, Download, Loader2, Play, Save } from 'lucide-vue-next';
import { onMounted, ref, watch } from 'vue';
import ResultsGrid from './ResultsGrid.vue';

const props = defineProps<{
  tab: Tab;
}>();

const gridStore = useGridStore();
const schemaStore = useSchemaStore();
const tabsStore = useTabsStore();

const { activeResultTab, executeRun, saveQuery, exportQuery } = useSqlEditor(props);

const editorContainer = ref<HTMLElement>();
const editorWidth = ref(600);
const sqlLimitOptions = [100, 200, 500, 1000, 5000].map((value) => ({
  label: String(value),
  value
}));

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

const setSqlLimit = (limit: DropdownValue) => {
  gridStore.sqlLimit = Number(limit);
};

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
    <!-- Results / Messages tabs toggle -->
    <div class="border-border bg-muted flex shrink-0 items-center border-b">
      <div
        class="text-text-secondary px-4 py-1.5 text-[12px] font-semibold tracking-wider uppercase"
      >
        Query Results
      </div>
      <div class="border-border ml-auto flex items-center border-l">
        <button
          class="cursor-pointer px-3 py-1.5 text-[12px] font-medium transition-colors"
          :class="
            activeResultTab === 'results'
              ? 'text-primary border-primary border-b-2'
              : 'text-text-secondary hover:text-text-primary'
          "
          @click="activeResultTab = 'results'"
        >
          Results
        </button>
        <button
          class="cursor-pointer px-3 py-1.5 text-[12px] font-medium transition-colors"
          :class="
            activeResultTab === 'messages'
              ? 'text-primary border-primary border-b-2'
              : 'text-text-secondary hover:text-text-primary'
          "
          @click="activeResultTab = 'messages'"
        >
          Messages
        </button>
      </div>
    </div>

    <!-- Split Content -->
    <div class="flex min-h-0 flex-1">
      <!-- Editor Pane -->
      <div
        class="border-border flex min-h-0 flex-col border-r"
        :style="{ width: editorWidth + 'px' }"
      >
        <div ref="editorContainer" class="min-h-0 flex-1 overflow-auto"></div>

        <!-- Run Bar -->
        <div class="border-border bg-muted flex items-center gap-3 border-t px-3 py-1.5">
          <button
            id="btn-run-query"
            class="bg-primary hover:bg-primary-hover text-text-inverse flex cursor-pointer items-center gap-1.5 rounded-lg px-3 py-1.5 text-[12px] font-medium shadow-sm transition-colors disabled:cursor-not-allowed disabled:opacity-70"
            :disabled="gridStore.isLoading"
            @click="handleRun"
          >
            <Loader2 v-if="gridStore.isLoading" :size="13" class="animate-spin" />
            <Play v-else :size="13" fill="currentColor" />
            {{ gridStore.isLoading ? 'Running...' : 'Run' }}
          </button>

          <button
            class="bg-surface border-border hover:border-border-strong text-text-secondary flex cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] transition-colors"
            @click="saveQuery"
            title="Save (Ctrl+S)"
          >
            <Save :size="13" />
          </button>

          <button
            class="bg-surface border-border hover:border-border-strong text-text-secondary flex cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] transition-colors"
            @click="exportQuery"
            title="Export to .sql file"
          >
            <Download :size="13" />
            <span>Export</span>
          </button>

          <div class="border-border ml-1 flex h-5 items-center gap-1 border-l pl-3">
            <span class="text-text-tertiary text-[11px]">Limit:</span>
            <DropdownMenu
              :model-value="gridStore.sqlLimit"
              :options="sqlLimitOptions"
              placement="top"
              aria-label="SQL result limit"
              button-class="border-none bg-transparent px-1 py-0 text-[11px] text-text-primary hover:bg-transparent hover:text-primary"
              menu-class="min-w-18"
              @update:model-value="setSqlLimit"
            />
          </div>

          <div class="text-text-secondary ml-auto flex items-center gap-1 text-[11px]">
            <Clock :size="12" class="text-success" />
            <span>{{ gridStore.sqlExecutionTime }} ms</span>
          </div>
        </div>
      </div>

      <!-- Resize Handle -->
      <ResizeHandle orientation="horizontal" v-model="editorWidth" />

      <!-- Results Pane -->
      <div class="min-h-0 flex-1 overflow-auto">
        <div v-if="activeResultTab === 'results'" class="h-full">
          <ResultsGrid />
        </div>
        <div v-else class="text-text-secondary p-3 text-[12px] font-(--font-mono)">
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
</template>
