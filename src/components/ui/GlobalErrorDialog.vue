<script setup lang="ts">
import { useErrorStore } from '@/stores/error'
import { AlertCircle, ChevronDown, ChevronRight, X } from 'lucide-vue-next'
import { ref } from 'vue'

const errorStore = useErrorStore()
const showDetails = ref(false)

function handleClose() {
  errorStore.clearError()
  showDetails.value = false
}
</script>

<template>
  <div v-if="errorStore.show"
    class="fixed inset-0 z-9999 flex items-center justify-center bg-black/40 backdrop-blur-sm">
    <div
      class="bg-surface border border-border rounded-xl shadow-2xl w-[550px] flex flex-col max-h-[85vh] overflow-hidden">
      <!-- Header -->
      <div class="flex items-center gap-3 px-5 py-4 border-b border-border bg-danger/5 text-danger">
        <AlertCircle :size="20" class="shrink-0" />
        <h3 class="text-[15px] font-semibold flex-1">Application Error</h3>
        <button @click="handleClose" class="text-danger/70 hover:text-danger transition-colors cursor-pointer">
          <X :size="18" />
        </button>
      </div>

      <!-- Body -->
      <div class="p-5 flex-col flex overflow-y-auto min-h-0">
        <p class="text-[14px] text-text-primary mb-4 leading-relaxed font-medium">
          {{ errorStore.message }}
        </p>

        <div v-if="errorStore.details" class="mt-2">
          <button @click="showDetails = !showDetails"
            class="flex items-center gap-1.5 text-[12px] font-medium text-text-tertiary hover:text-text-secondary transition-colors focus:outline-none cursor-pointer">
            <component :is="showDetails ? ChevronDown : ChevronRight" :size="14" />
            <span>{{ showDetails ? 'Hide details' : 'Show details' }}</span>
          </button>

          <div v-show="showDetails"
            class="mt-3 p-3 bg-muted border border-border rounded-lg overflow-x-auto text-[11px] font-(--font-mono) text-text-secondary whitespace-pre-wrap break-all max-h-[300px] overflow-y-auto">
            {{ errorStore.details }}
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end px-5 py-4 border-t border-border bg-muted/30">
        <button @click="handleClose"
          class="px-5 py-2 text-[13px] font-medium text-white bg-danger hover:bg-danger-hover transition-colors rounded-lg shadow-sm cursor-pointer">
          Close
        </button>
      </div>
    </div>
  </div>
</template>
