<script setup lang="ts">
import { useSchemaStore } from '@/stores/schema'
import { useTabsStore } from '@/stores/tabs'
import {
  ChevronRight,
  Table2,
} from 'lucide-vue-next'
import { ref } from 'vue'

const schemaStore = useSchemaStore()
const tabsStore = useTabsStore()

const expandedSections = ref<Record<string, boolean>>({
  tables: true,
})

const toggleSection = (key: string) => {
  expandedSections.value[key] = !expandedSections.value[key]
}

const handleTableClick = (tableName: string) => {
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
        <div v-if="schemaStore.filteredTables.length === 0"
          class="pl-5 pr-3 py-2 text-[12px] text-text-tertiary italic">
          No tables found
        </div>
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

    <!-- Views & Functions are temporarily hidden (future features) -->
  </div>
</template>
