<script setup lang="ts">
import { computed, ref } from 'vue';
import Button from 'primevue/button';
import ContextMenu from 'primevue/contextmenu';
import InputText from 'primevue/inputtext';
import { AppWindow, Code2, LayoutGrid, Minus, Plus, Trash2, X } from 'lucide-vue-next';

import { useConnectionsStore } from '@/stores/connections';
import { usePreferencesStore } from '@/stores/preferences';
import { useTabsStore } from '@/stores/tabs';
import { useToastStore } from '@/stores/toast';
import { launchNativeUi } from '@/services/nativeService';
import { TabType, type Tab } from '@/types';

const tabsStore = useTabsStore();
const connectionsStore = useConnectionsStore();
const preferencesStore = usePreferencesStore();
const toastStore = useToastStore();

const isLaunchingNativeUi = ref(false);

const onSwitchToNativeUi = async () => {
  if (isLaunchingNativeUi.value) return;
  isLaunchingNativeUi.value = true;
  try {
    const result = await launchNativeUi();
    if (result.launched) {
      toastStore.addToast({
        title: 'Native UI launched',
        message: 'Opened experimental GPUI desktop window.',
        severity: 'info',
        ttl: 3000
      });
    } else {
      toastStore.addToast({
        title: 'Native UI unavailable',
        message: result.message || 'Unable to launch experimental GPUI window.',
        severity: result.errorCode === 'SPAWN_FAILED' ? 'error' : 'warning',
        ttl: 5000
      });
    }
  } finally {
    isLaunchingNativeUi.value = false;
  }
};

const getTabColorClass = (connectionId?: string) => {
  if (!connectionId) return '';
  const conn = connectionsStore.connections.find((c) => c.id === connectionId);
  if (!conn) return '';
  return `bg-conn-${conn.color}`;
};

// ─── Tab Context Menu ────────────────────────────────────────────────────────
const menuRef = ref();
const menuTargetTabId = ref<string | null>(null);
const targetTab = ref<Tab | null | undefined>(null);

const onContextMenu = (e: MouseEvent, tabId: string) => {
  e.preventDefault();
  menuTargetTabId.value = tabId;
  targetTab.value = tabsStore.getById(tabId);
  menuRef.value?.show(e);
};

const closeTab = () => {
  if (menuTargetTabId.value) tabsStore.closeTab(menuTargetTabId.value);
};

const closeOthers = () => {
  if (menuTargetTabId.value) {
    const toClose = tabsStore.tabs
      .filter((t: Tab) => t.id !== menuTargetTabId.value)
      .map((t: Tab) => t.id);
    toClose.forEach((id: string) => tabsStore.closeTab(id));
  }
};

const closeAll = () => {
  const toClose = tabsStore.tabs.map((t: Tab) => t.id);
  toClose.forEach((id: string) => tabsStore.closeTab(id));
};

const openNewQueryConsole = () => {
  tabsStore.openSqlEditor(undefined, '', true, true);
};

const minimizeTab = () => {
  if (menuTargetTabId.value) tabsStore.minimizeTab(menuTargetTabId.value);
};

const deleteTab = () => {
  if (menuTargetTabId.value) {
    tabsStore.deleteTab(menuTargetTabId.value);
  }
};

const menuItems = computed(() => {
  const items: any[] = [
    {
      label: 'Close Tab',
      lucideIcon: X,
      command: () => closeTab()
    },
    {
      label: 'Minimize',
      lucideIcon: Minus,
      command: () => minimizeTab()
    },
    { separator: true },
    {
      label: 'Close Others',
      command: () => closeOthers()
    },
    {
      label: 'Close All',
      command: () => closeAll()
    }
  ];

  if (targetTab.value?.type === TabType.SQL) {
    items.push(
      { separator: true },
      {
        label: 'Delete',
        lucideIcon: Trash2,
        class: 'text-danger',
        command: () => deleteTab()
      }
    );
  }

  return items;
});

const onDragStart = (event: DragEvent, tabId: string) => {
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', tabId);
  }
  tabsStore.draggingTabId = tabId;
};

const onDragEnd = () => {
  tabsStore.draggingTabId = null;
};

const onDragOver = (event: DragEvent) => {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
};

const onDrop = (event: DragEvent, targetTabId: string) => {
  event.preventDefault();
  const draggingId = event.dataTransfer?.getData('text/plain');
  if (draggingId && draggingId !== targetTabId) {
    tabsStore.reorderTab(draggingId, targetTabId);
  }
};

// ─── Renaming Logic ──────────────────────────────────────────────────────────
const renamingTabId = ref<string | null>(null);
const tempTitle = ref('');

const startRenaming = (tabId: string, currentTitle: string) => {
  renamingTabId.value = tabId;
  tempTitle.value = currentTitle;
};

const finishRenaming = () => {
  if (renamingTabId.value && tempTitle.value.trim()) {
    tabsStore.renameTab(renamingTabId.value, tempTitle.value.trim());
  }
  renamingTabId.value = null;
};
</script>

<template>
  <header
    class="bg-muted border-border flex h-(--titlebar-height) shrink-0 items-center border-b pr-3 pl-2"
  >
    <!-- Tab Strip -->
    <nav
      class="scrollbar-none flex min-w-0 flex-1 items-end overflow-x-auto overflow-y-hidden"
      id="tab-strip"
    >
      <div
        v-for="tab in tabsStore.mainTabs"
        :key="tab.id"
        role="tab"
        tabindex="0"
        :aria-selected="tabsStore.activeTabId === tab.id"
        draggable="true"
        class="group relative flex h-[calc(var(--titlebar-height)-.75rem)] max-w-50 min-w-30 shrink-0 cursor-pointer items-center gap-1.5 rounded-t-lg border border-b-0 border-transparent px-3 text-[13px] whitespace-nowrap transition-colors"
        :class="
          tabsStore.activeTabId === tab.id
            ? 'bg-surface border-border text-text-primary after:bg-surface z-10 font-medium after:absolute after:right-0 after:-bottom-px after:left-0 after:h-px'
            : 'text-text-secondary hover:bg-hover/50 hover:text-text-primary'
        "
        @click="tabsStore.setActiveTab(tab.id)"
        @keydown.enter="tabsStore.setActiveTab(tab.id)"
        @contextmenu.prevent="onContextMenu($event, tab.id)"
        @dragstart="onDragStart($event, tab.id)"
        @dragend="onDragEnd"
        @dragover="onDragOver"
        @drop="onDrop($event, tab.id)"
      >
        <!-- Connection color indicator -->
        <div
          v-if="tab.connectionId"
          class="absolute top-0 right-0 left-0 h-0.5 transition-all duration-200"
          :class="getTabColorClass(tab.connectionId)"
        />

        <component
          :is="tab.type === 'sql' ? Code2 : LayoutGrid"
          :size="14"
          class="shrink-0"
          :class="tabsStore.activeTabId === tab.id ? 'text-primary' : 'text-text-tertiary'"
        />

        <span
          v-if="renamingTabId !== tab.id"
          class="flex-1 truncate text-left"
          @dblclick="startRenaming(tab.id, tab.title)"
        >
          {{ tab.title }}
        </span>
        <InputText
          v-else
          v-model="tempTitle"
          size="small"
          class="h-6! flex-1 px-1! text-[13px]!"
          @blur="finishRenaming"
          @keydown.enter="finishRenaming"
          @keydown.esc="renamingTabId = null"
          autofocus
        />

        <div v-if="tab.isDirty" class="bg-primary h-1.5 w-1.5 shrink-0 rounded-full" />
        <span
          class="hover:bg-border flex h-4 w-4 items-center justify-center rounded opacity-0 transition-opacity group-hover:opacity-100"
          @click.stop="tabsStore.closeTab(tab.id)"
        >
          <X :size="12" />
        </span>
      </div>

      <Button
        variant="text"
        severity="secondary"
        size="small"
        class="mb-px! h-[calc(var(--titlebar-height)-8px)]! w-8! shrink-0! rounded-t-lg! rounded-b-none! p-0!"
        v-tooltip="'New Query Console'"
        @click="openNewQueryConsole"
      >
        <template #icon>
          <Plus :size="14" />
        </template>
      </Button>
    </nav>

    <Button
      v-if="preferencesStore.settings.experimentalFeatures"
      data-testid="btn-switch-native-ui"
      label="New UI"
      variant="outlined"
      severity="secondary"
      size="small"
      :loading="isLaunchingNativeUi"
      class="ml-2 h-7 shrink-0 text-xs"
      v-tooltip.bottom="'Launch experimental native GPUI window'"
      @click="onSwitchToNativeUi"
    >
      <template #icon>
        <AppWindow :size="13" class="mr-1" />
      </template>
    </Button>

    <!-- Context Menu -->
    <ContextMenu ref="menuRef" :model="menuItems">
      <template #item="{ item, props }">
        <a
          v-bind="props.action"
          class="flex items-center gap-2 px-3 py-1.5 text-xs"
          :class="item.class || 'text-text-primary'"
        >
          <component :is="item.lucideIcon" v-if="item.lucideIcon" :size="13" class="shrink-0" />
          <span>{{ item.label }}</span>
        </a>
      </template>
    </ContextMenu>
  </header>
</template>
