<script setup lang="ts">
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import { useErrorStore } from '@/stores/error';
import { AlertCircle, ChevronDown, ChevronRight } from 'lucide-vue-next';
import { ref } from 'vue';

const errorStore = useErrorStore();
const showDetails = ref(false);

const handleClose = () => {
  errorStore.clearError();
  showDetails.value = false;
};
</script>

<template>
  <Dialog
    :visible="errorStore.show"
    modal
    :closable="true"
    :style="{ width: '34rem' }"
    @update:visible="
      (val) => {
        if (!val) handleClose();
      }
    "
  >
    <template #header>
      <div class="text-danger flex items-center gap-2">
        <AlertCircle :size="18" class="shrink-0" />
        <span class="text-[15px] font-semibold">Application Error</span>
      </div>
    </template>

    <div class="py-2">
      <p class="text-text-primary mb-4 text-[14px] leading-relaxed font-medium">
        {{ errorStore.message }}
      </p>

      <div v-if="errorStore.details" class="mt-2">
        <Button
          variant="text"
          severity="secondary"
          size="small"
          @click="showDetails = !showDetails"
          class="text-text-tertiary! hover:text-text-secondary! p-0!"
        >
          <ChevronDown v-if="showDetails" :size="16" />
          <ChevronRight v-else :size="16" />
          <span>{{ showDetails ? 'Hide details' : 'Show details' }}</span>
        </Button>

        <div
          v-show="showDetails"
          class="bg-muted border-border text-text-secondary mt-3 max-h-75 overflow-x-auto overflow-y-auto rounded-lg border p-3 text-[11px] font-(--font-mono) break-all whitespace-pre-wrap"
        >
          {{ errorStore.details }}
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-end pt-2">
        <Button severity="danger" size="small" @click="handleClose"> Close </Button>
      </div>
    </template>
  </Dialog>
</template>
