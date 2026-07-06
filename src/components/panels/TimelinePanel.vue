<script setup lang="ts">
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';

import { useGridStore } from '@/stores/grid';
import { useHistoryStore } from '@/stores/history';
import { useTabsStore } from '@/stores/tabs';
import { useToastStore } from '@/stores/toast';

import type { HistoryEntry } from '@/types';

import {
  Code2,
  Copy,
  Database,
  ExternalLink,
  History,
  Play,
  Search,
  Trash2
} from 'lucide-vue-next';
import { computed, onMounted, ref } from 'vue';

const historyStore = useHistoryStore();
const tabsStore = useTabsStore();
const gridStore = useGridStore();
const toastStore = useToastStore();

const searchQuery = ref('');
const expandedEntryIds = ref<Record<string, boolean>>({});
const showClearConfirm = ref(false);

onMounted(async () => {
  await historyStore.loadHistory();
});

const filteredHistory = computed(() => {
  const query = searchQuery.value.toLowerCase().trim();
  if (!query) return historyStore.history;

  return historyStore.history.filter((entry) => {
    if (entry.type === 'sql' && entry.query) {
      return entry.query.toLowerCase().includes(query);
    }
    if (entry.type === 'table' && entry.tableName) {
      return (
        entry.tableName.toLowerCase().includes(query) ||
        (entry.schema && entry.schema.toLowerCase().includes(query))
      );
    }
    return false;
  });
});

const formatTime = (timeStr: string) => {
  const date = new Date(timeStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;

  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h ago`;

  return date.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
};

const toggleExpand = (id: string) => {
  expandedEntryIds.value[id] = !expandedEntryIds.value[id];
};

const handleCopy = (text: string) => {
  navigator.clipboard.writeText(text);
  toastStore.addToast({
    severity: 'success',
    title: 'Copied',
    message: 'Copied to clipboard successfully.',
    ttl: 2000
  });
};

const openInEditor = (entry: HistoryEntry) => {
  if (entry.type === 'sql' && entry.query) {
    tabsStore.openSqlEditor(entry.connectionId, entry.query, true, true, entry.dbName);
  }
};

const reRunQuery = async (entry: HistoryEntry) => {
  if (entry.type === 'sql' && entry.query) {
    try {
      // Find the console panel inside layout and switch tab to output/messages
      const { useLayoutStore } = await import('@/stores/layout');
      const layoutStore = useLayoutStore();
      const consolePanel = layoutStore.panels['console'];
      if (consolePanel) {
        consolePanel.activeTabId = 'output';
      }

      await gridStore.runQuery(entry.query, undefined, entry.connectionId, entry.dbName);
    } catch (err: any) {
      console.error('Failed to re-run query:', err);
    }
  }
};

const handleClearHistory = () => {
  showClearConfirm.value = true;
};

const confirmClearHistory = async () => {
  showClearConfirm.value = false;
  await historyStore.clearHistory();
  toastStore.addToast({
    severity: 'info',
    title: 'History Cleared',
    message: 'Timeline activity history has been cleared.'
  });
};

const colorMap: Record<string, string> = {
  indigo: 'bg-conn-indigo',
  blue: 'bg-conn-blue',
  teal: 'bg-conn-teal',
  green: 'bg-conn-green',
  amber: 'bg-conn-amber',
  orange: 'bg-conn-orange',
  pink: 'bg-conn-pink',
  gray: 'bg-conn-gray',
  purple: 'bg-conn-purple',
  rose: 'bg-conn-rose',
  emerald: 'bg-conn-emerald',
  cyan: 'bg-conn-cyan',
  violet: 'bg-conn-violet',
  red: 'bg-conn-red'
};

const getFlagColorClass = (colorName?: string) => {
  if (!colorName) return 'bg-gray-500';
  return colorMap[colorName] || 'bg-conn-gray';
};
</script>

<template>
  <div class="bg-surface flex flex-1 flex-col overflow-hidden text-[12px]">
    <!-- Toolbar/Search Area -->
    <div
      class="border-border bg-sidebar/30 flex shrink-0 items-center justify-between gap-3 border-b px-3 py-2"
    >
      <div
        class="bg-surface border-border focus-within:border-primary/50 flex flex-1 items-center gap-2 rounded-lg border px-2.5 py-1 transition-colors"
      >
        <Search :size="12" class="text-text-tertiary shrink-0" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search history..."
          class="text-text-primary placeholder-text-tertiary flex-1 border-none bg-transparent py-0.5 text-[12px] outline-none"
        />
        <button
          v-if="searchQuery"
          class="text-text-tertiary hover:text-text-primary shrink-0 cursor-pointer"
          @click="searchQuery = ''"
        >
          &times;
        </button>
      </div>

      <Button
        v-if="historyStore.history.length > 0"
        variant="text"
        severity="secondary"
        size="small"
        class="text-text-tertiary hover:text-danger h-7"
        @click="handleClearHistory"
      >
        <Trash2 class="h-4 w-4" />
        Clear
      </Button>
    </div>

    <!-- History Timeline List -->
    <div class="flex-1 overflow-y-auto px-4 py-3">
      <!-- Empty state -->
      <div
        v-if="filteredHistory.length === 0"
        class="text-text-tertiary flex flex-col items-center justify-center gap-3 py-16 text-center select-none"
      >
        <History :size="28" class="opacity-30" />
        <div class="italic">
          {{
            searchQuery ? 'No matching history entries found.' : 'No activity history recorded yet.'
          }}
        </div>
      </div>

      <!-- Real timeline stream -->
      <div v-else class="border-border relative ml-2 flex flex-col gap-4 border-l pl-4">
        <div
          v-for="entry in filteredHistory"
          :key="entry.id"
          class="group hover:bg-hover/20 hover:border-border relative flex flex-col gap-1.5 rounded-lg border border-transparent p-2.5 transition-all duration-200"
        >
          <!-- Timeline point indicator -->
          <div
            class="border-surface absolute top-4 -left-5.25 h-2.5 w-2.5 rounded-full border"
            :class="[entry.type === 'sql' ? 'bg-primary' : 'bg-success']"
          />

          <!-- Header / Title -->
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-2">
              <span
                v-if="entry.type === 'sql'"
                class="bg-primary/10 text-primary flex items-center gap-1 rounded px-1.5 py-0.5 text-[9px] font-semibold uppercase"
              >
                <Code2 :size="10" /> SQL
              </span>
              <span
                v-else
                class="bg-success/10 text-success flex items-center gap-1 rounded px-1.5 py-0.5 text-[9px] font-semibold uppercase"
              >
                <Database :size="10" /> TABLE
              </span>

              <!-- Time -->
              <span class="text-text-tertiary text-[10px]" :title="entry.timestamp">
                {{ formatTime(entry.timestamp) }}
              </span>

              <span class="text-text-tertiary opacity-30 select-none">│</span>

              <!-- Connection indicator pill -->
              <span class="text-text-secondary inline-flex items-center gap-1 text-[10px]">
                <span
                  class="h-1.5 w-1.5 shrink-0 rounded-full"
                  :class="getFlagColorClass(entry.connectionColor)"
                />
                <span class="max-w-20 truncate font-medium">{{ entry.connectionName }}</span>
                <span v-if="entry.dbName" class="text-text-tertiary text-[9px]"
                  >/ {{ entry.dbName }}</span
                >
              </span>
            </div>

            <!-- Actions Row (displayed on hover/focus) -->
            <div
              class="flex shrink-0 items-center gap-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
            >
              <template v-if="entry.type === 'sql' && entry.query">
                <button
                  v-tooltip.top="'Copy SQL'"
                  class="text-text-tertiary hover:text-text-primary hover:bg-hover rounded p-1"
                  @click.stop="handleCopy(entry.query)"
                >
                  <Copy :size="12" />
                </button>
                <button
                  v-tooltip.top="'Open in SQL Editor'"
                  class="text-text-tertiary hover:text-text-primary hover:bg-hover rounded p-1"
                  @click.stop="openInEditor(entry)"
                >
                  <ExternalLink :size="12" />
                </button>
                <button
                  v-tooltip.top="'Re-run Query'"
                  class="text-primary hover:text-primary-hover hover:bg-hover rounded p-1"
                  @click.stop="reRunQuery(entry)"
                >
                  <Play :size="12" />
                </button>
              </template>
              <template v-else-if="entry.type === 'table' && entry.tableName">
                <button
                  v-tooltip.top="'Copy Table Name'"
                  class="text-text-tertiary hover:text-text-primary hover:bg-hover rounded p-1"
                  @click.stop="handleCopy(entry.tableName)"
                >
                  <Copy :size="12" />
                </button>
              </template>
            </div>
          </div>

          <!-- Content Details -->
          <div class="min-w-0 flex-1">
            <!-- SQL query details -->
            <div v-if="entry.type === 'sql'" class="flex flex-col gap-1">
              <div
                class="bg-muted/40 border-border/20 hover:border-border max-w-full cursor-pointer overflow-x-auto rounded border p-2 font-mono text-[11.5px] transition-colors select-text"
                :class="[
                  expandedEntryIds[entry.id] ? 'overflow-x-auto whitespace-pre' : 'truncate'
                ]"
                @click="toggleExpand(entry.id)"
                title="Click to expand/collapse full SQL"
              >
                {{ entry.query }}
              </div>
              <div class="text-text-tertiary mt-0.5 flex items-center gap-2.5 text-[10px]">
                <span v-if="!entry.success" class="text-danger font-medium">
                  Failed: {{ entry.error }}
                </span>
                <template v-else>
                  <span v-if="entry.rowCount !== undefined" class="text-text-secondary font-medium">
                    {{ entry.rowCount.toLocaleString() }} rows affected
                  </span>
                  <span v-if="entry.executionTime !== undefined">
                    Execution: {{ entry.executionTime }}ms
                  </span>
                </template>
              </div>
            </div>

            <!-- Table loaded details -->
            <div v-else class="text-text-primary">
              Loaded table
              <span
                class="text-text-secondary bg-muted/50 border-border/20 rounded border px-1.5 py-0.5 font-mono text-[11px] font-bold"
              >
                {{ entry.schema ? `${entry.schema}.${entry.tableName}` : entry.tableName }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Clear History Confirmation Dialog -->
  <ConfirmDialog
    v-if="showClearConfirm"
    title="Clear History"
    message="Are you sure you want to clear your query and action history?"
    variant="danger"
    confirm-label="Clear"
    @confirm="confirmClearHistory"
    @cancel="showClearConfirm = false"
  />
</template>
