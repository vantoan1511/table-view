<script setup lang="ts">
import { useErrorStore } from '@/stores/error';
import { AlertCircle, ChevronDown, ChevronRight, X } from 'lucide-vue-next';
import { ref } from 'vue';

const errorStore = useErrorStore();
const showDetails = ref(false);

const handleClose = () => {
  errorStore.clearError();
  showDetails.value = false;
};
</script>

<template>
  <div
    v-if="errorStore.show"
    class="modal-backdrop fixed inset-0 z-9999 flex items-center justify-center bg-black/40 backdrop-blur-sm"
  >
    <div
      class="bg-surface border-border modal-container flex max-h-[85vh] w-137.5 flex-col overflow-hidden rounded-xl border shadow-2xl"
    >
      <!-- Header -->
      <div class="border-border bg-danger/5 text-danger flex items-center gap-3 border-b px-5 py-4">
        <AlertCircle :size="20" class="shrink-0" />
        <h3 class="flex-1 text-[15px] font-semibold">Application Error</h3>
        <button
          @click="handleClose"
          class="text-danger/70 hover:text-danger cursor-pointer transition-colors"
        >
          <X :size="18" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex min-h-0 flex-col overflow-y-auto p-5">
        <p class="text-text-primary mb-4 text-[14px] leading-relaxed font-medium">
          {{ errorStore.message }}
        </p>

        <div v-if="errorStore.details" class="mt-2">
          <button
            @click="showDetails = !showDetails"
            class="text-text-tertiary hover:text-text-secondary flex cursor-pointer items-center gap-1.5 text-[12px] font-medium transition-colors focus:outline-none"
          >
            <component :is="showDetails ? ChevronDown : ChevronRight" :size="14" />
            <span>{{ showDetails ? 'Hide details' : 'Show details' }}</span>
          </button>

          <div
            v-show="showDetails"
            class="bg-muted border-border text-text-secondary mt-3 max-h-75 overflow-x-auto overflow-y-auto rounded-lg border p-3 text-[11px] font-(--font-mono) break-all whitespace-pre-wrap"
          >
            {{ errorStore.details }}
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="border-border bg-muted/30 flex items-center justify-end border-t px-5 py-4">
        <button
          @click="handleClose"
          class="bg-danger hover:bg-danger-hover cursor-pointer rounded-lg px-5 py-2 text-[13px] font-medium text-white shadow-sm transition-colors"
        >
          Close
        </button>
      </div>
    </div>
  </div>
</template>
