<script setup lang="ts">
import { useSchemaStore } from '@/stores/schema'
import { useTabsStore } from '@/stores/tabs'
import {
  ChevronRight,
  Database,
  Eye,
  FunctionSquare,
  Table2,
} from 'lucide-vue-next'
import { ref } from 'vue'

const schemaStore = useSchemaStore()
const tabsStore = useTabsStore()

const expandedSections = ref<Record<string, boolean>>({
  tables: true,
  views: false,
  functions: false,
  schemas: false,
})

function toggleSection(key: string) {
  expandedSections.value[key] = !expandedSections.value[key]
}

function handleTableClick(tableName: string) {
  tabsStore.openTable(tableName)
}
</script>

<template>
  <div class="py-1">
    <!-- Tables Section -->
    <div>
      <button
        class="flex items-center gap-1.5 w-full px-3 py-1.5 text-[12px] font-semibold text-text-secondary hover:bg-hover cursor-pointer transition-colors"
        @click="toggleSection('tables')">
        <ChevronRight :size="14" class="shrink-0 transition-transform duration-200"
          :class="expandedSections.tables ? 'rotate-90' : ''" />
        <Table2 :size="14" class="shrink-0 text-text-tertiary" />
        <span class="flex-1 text-left">Tables</span>
        <span class="text-[11px] text-text-tertiary font-normal">({{ schemaStore.filteredTables.length }})</span>
      </button>
      <div v-show="expandedSections.tables" class="ml-3">
        <button v-for="table in schemaStore.filteredTables" :key="table.name"
          class="flex items-center gap-2 w-full pl-5 pr-3 py-1 text-[13px] rounded-md cursor-pointer transition-colors"
          :class="tabsStore.activeTab?.tableName === table.name
            ? 'bg-active text-primary font-medium'
            : 'text-text-primary hover:bg-hover'
            " @click="handleTableClick(table.name)">
          <Table2 :size="13" class="shrink-0 opacity-50" />
          <span class="truncate">{{ table.name }}</span>
        </button>
      </div>
    </div>

    <!-- Views Section -->
    <div class="mt-1">
      <button
        class="flex items-center gap-1.5 w-full px-3 py-1.5 text-[12px] font-semibold text-text-secondary hover:bg-hover cursor-pointer transition-colors"
        @click="toggleSection('views')">
        <ChevronRight :size="14" class="shrink-0 transition-transform duration-200"
          :class="expandedSections.views ? 'rotate-90' : ''" />
        <Eye :size="14" class="shrink-0 text-text-tertiary" />
        <span class="flex-1 text-left">Views</span>
        <span class="text-[11px] text-text-tertiary font-normal">({{ schemaStore.filteredViews.length }})</span>
      </button>
      <div v-show="expandedSections.views" class="ml-3">
        <button v-for="view in schemaStore.filteredViews" :key="view.name"
          class="flex items-center gap-2 w-full pl-5 pr-3 py-1 text-[13px] text-text-primary hover:bg-hover rounded-md cursor-pointer transition-colors">
          <Eye :size="13" class="shrink-0 opacity-50" />
          <span class="truncate">{{ view.name }}</span>
        </button>
      </div>
    </div>

    <!-- Functions Section -->
    <div class="mt-1">
      <button
        class="flex items-center gap-1.5 w-full px-3 py-1.5 text-[12px] font-semibold text-text-secondary hover:bg-hover cursor-pointer transition-colors"
        @click="toggleSection('functions')">
        <ChevronRight :size="14" class="shrink-0 transition-transform duration-200"
          :class="expandedSections.functions ? 'rotate-90' : ''" />
        <FunctionSquare :size="14" class="shrink-0 text-text-tertiary" />
        <span class="flex-1 text-left">Functions</span>
        <span class="text-[11px] text-text-tertiary font-normal">({{ schemaStore.filteredFunctions.length }})</span>
      </button>
      <div v-show="expandedSections.functions" class="ml-3">
        <button v-for="fn in schemaStore.filteredFunctions" :key="fn.name"
          class="flex items-center gap-2 w-full pl-5 pr-3 py-1 text-[13px] text-text-primary hover:bg-hover rounded-md cursor-pointer transition-colors">
          <FunctionSquare :size="13" class="shrink-0 opacity-50" />
          <span class="truncate">{{ fn.name }}</span>
        </button>
      </div>
    </div>

    <!-- Schemas Section -->
    <div class="mt-1">
      <button
        class="flex items-center gap-1.5 w-full px-3 py-1.5 text-[12px] font-semibold text-text-secondary hover:bg-hover cursor-pointer transition-colors"
        @click="toggleSection('schemas')">
        <ChevronRight :size="14" class="shrink-0 transition-transform duration-200"
          :class="expandedSections.schemas ? 'rotate-90' : ''" />
        <Database :size="14" class="shrink-0 text-text-tertiary" />
        <span class="flex-1 text-left">Schemas</span>
        <span class="text-[11px] text-text-tertiary font-normal">({{ schemaStore.schema.schemas.length }})</span>
      </button>
      <div v-show="expandedSections.schemas" class="ml-3">
        <button v-for="s in schemaStore.schema.schemas" :key="s"
          class="flex items-center gap-2 w-full pl-5 pr-3 py-1 text-[13px] text-text-primary hover:bg-hover rounded-md cursor-pointer transition-colors">
          <Database :size="13" class="shrink-0 opacity-50" />
          <span class="truncate">{{ s }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
