<script setup lang="ts">
// Components
import Button from '@/components/ui/Button.vue';
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';
import ResizeHandle from '@/components/ui/ResizeHandle.vue';
import Tooltip from '@/components/ui/Tooltip.vue';
import ResultsGrid from './ResultsGrid.vue';

// Composables
import { useDebounce } from '@/composables/useDebounce';
import { useSqlEditor } from '@/composables/useSqlEditor';

import { useConnectionsStore } from '@/stores/connections';
import { useDiagramStore } from '@/stores/diagram';
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
import { Clock, Download, Play, Save, Check, RotateCcw } from 'lucide-vue-next';
import { onMounted, ref, watch } from 'vue';

// Other imports
import { editorTheme, sqlHighlightStyle } from '@/lib/editorConfig';
import { formatSql } from '@/utils/sqlFormatter';

const props = defineProps<{
  tab: Tab;
}>();

const connectionsStore = useConnectionsStore();
const gridStore = useGridStore();
const schemaStore = useSchemaStore();
const tabsStore = useTabsStore();
const diagramStore = useDiagramStore();

const {
  activeResultTab,
  autoCommit,
  showDestructiveConfirm,
  showTransactionConfirm,
  hasActiveTransaction,
  executeRun,
  confirmDestructive,
  cancelDestructive,
  commitTx,
  rollbackTx,
  saveQuery,
  exportQuery
} = useSqlEditor(props);

const editorContainer = ref<HTMLElement>();
const editorWidth = ref(600);

let editorView: EditorView | null = null;
const sqlCompartment = new Compartment();

const buildSqlExtension = () => {
  const schemaMap: Record<string, string[]> = {};
  const connId = props.tab.connectionId || connectionsStore.activeConnectionId;
  const dbName = props.tab.dbName;
  const schemaName = props.tab.schema || schemaStore.selectedSchema;
  const source =
    (connId ? schemaStore.schemasByConnection[connId] : undefined) ?? schemaStore.schema;

  const cachedDetails =
    connId && schemaName
      ? diagramStore.diagrams[diagramStore.getCacheKey(connId, schemaName, dbName)]
      : undefined;

  if (cachedDetails) {
    for (const table of cachedDetails.tables) {
      schemaMap[table.name] = table.columns.map((c) => c.name);
    }
  } else {
    for (const table of source.tables) {
      schemaMap[table.name] = [];
    }
    for (const view of source.views) {
      schemaMap[view.name] = [];
    }
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

const handleFormat = () => {
  if (!editorView) return;
  const { from, to } = editorView.state.selection.main;
  const hasSelection = from !== to;

  const textToFormat = hasSelection
    ? editorView.state.sliceDoc(from, to)
    : editorView.state.doc.toString();

  if (!textToFormat || textToFormat.trim() === '') return;

  const connId = props.tab.connectionId || connectionsStore.activeConnectionId;
  const connection = connId ? connectionsStore.connections.find((c) => c.id === connId) : undefined;
  const dbType = connection?.type;

  const formatted = formatSql(textToFormat, dbType);

  if (formatted && formatted !== textToFormat) {
    editorView.dispatch({
      changes: hasSelection
        ? { from, to, insert: formatted }
        : { from: 0, to: editorView.state.doc.length, insert: formatted }
    });
  }
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
          },
          {
            key: 'Mod-Shift-f',
            run: () => {
              handleFormat();
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

const loadDetails = async () => {
  const connId = props.tab.connectionId || connectionsStore.activeConnectionId;
  const dbName = props.tab.dbName;
  const schemaName = props.tab.schema || schemaStore.selectedSchema;
  if (connId && schemaName) {
    await diagramStore.fetchSchemaDetails(connId, schemaName, dbName);
    if (editorView) {
      editorView.dispatch({
        effects: sqlCompartment.reconfigure(buildSqlExtension())
      });
    }
  }
};

onMounted(() => {
  initEditor();
  loadDetails();

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
      loadDetails();
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

          <template v-if="hasActiveTransaction">
            <Button variant="success" size="sm" :icon="Check" @click="commitTx"> Commit </Button>
            <Button variant="danger" size="sm" :icon="RotateCcw" @click="rollbackTx">
              Rollback
            </Button>
          </template>

          <span class="relative">
            <Button variant="secondary" size="sm" :icon="Save" @click="saveQuery" />
            <Tooltip text="Save (Ctrl+S)" position="top" />
          </span>

          <span class="relative">
            <Button variant="secondary" size="sm" :icon="Download" @click="exportQuery">
              Export
            </Button>
            <Tooltip text="Export to .sql file" position="top" />
          </span>

          <label
            class="hover:bg-hover text-text-secondary border-border bg-surface ml-1 flex cursor-pointer items-center gap-1.5 rounded border px-2 py-1 text-[11px] font-medium transition-colors select-none"
          >
            <input
              type="checkbox"
              v-model="autoCommit"
              class="form-checkbox border-border bg-muted text-primary focus:ring-primary/20 accent-primary h-3 w-3 rounded"
            />
            <span>Auto-Commit</span>
          </label>

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

    <!-- Confirm Dialog for Destructive Query Warning -->
    <ConfirmDialog
      v-if="showDestructiveConfirm"
      title="Dangerous Query Warning"
      message="You are about to execute a potentially destructive query (DROP, TRUNCATE, or UPDATE/DELETE without a WHERE clause). Are you sure you want to proceed?"
      variant="danger"
      confirm-label="Execute"
      cancel-label="Cancel"
      @confirm="confirmDestructive"
      @cancel="cancelDestructive"
    />

    <!-- Confirm Dialog for Transaction Preview -->
    <ConfirmDialog
      v-if="showTransactionConfirm"
      title="Transaction Preview"
      :message="`Query executed successfully. ${gridStore.sqlRowCount} rows affected. Would you like to commit these changes or rollback?`"
      variant="info"
      confirm-label="Commit"
      cancel-label="Rollback"
      @confirm="commitTx"
      @cancel="rollbackTx"
    />
  </div>
</template>
