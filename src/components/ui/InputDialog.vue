<script setup lang="ts">
import { AlertCircle, X } from 'lucide-vue-next';
import { ref } from 'vue';

withDefaults(
  defineProps<{
    title: string;
    label: string;
    placeholder?: string;
    confirmLabel?: string;
    footerNote?: string;
  }>(),
  {
    placeholder: '',
    confirmLabel: 'Confirm',
    footerNote: ''
  }
);

const emit = defineEmits<{
  (e: 'submit', value: string): void;
  (e: 'close'): void;
}>();

const inputValue = ref('');

const handleSubmit = () => {
  if (!inputValue.value.trim()) return;
  emit('submit', inputValue.value.trim());
};
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
    <div
      class="bg-surface border-border flex w-100 animate-[scale-in_0.15s_ease-out] flex-col overflow-hidden rounded-xl border shadow-2xl"
    >
      <!-- Header -->
      <div class="border-border flex items-center justify-between border-b px-5 py-4">
        <h3 class="text-text-primary text-base font-semibold">{{ title }}</h3>
        <button
          @click="$emit('close')"
          class="text-text-tertiary hover:text-text-primary transition-colors"
        >
          <X :size="18" />
        </button>
      </div>

      <!-- Body -->
      <div class="p-5">
        <div class="mb-2">
          <div class="mb-1.5 flex items-center justify-between">
            <label class="text-text-secondary block text-[13px] font-medium">{{ label }}</label>
            <span
              v-if="!inputValue.trim()"
              class="text-danger animate-fade-in-scale flex items-center gap-1 text-[11px] font-medium"
            >
              <AlertCircle :size="12" />
              Required
            </span>
          </div>
          <input
            v-model.trim="inputValue"
            type="text"
            :placeholder="placeholder"
            class="bg-muted border-border text-text-primary focus:border-primary w-full rounded-lg border px-4 py-2 text-[14px] transition-colors outline-none"
            :class="{ 'border-danger! !focus:border-danger': !inputValue.trim() }"
            @keyup.enter="handleSubmit"
            autofocus
          />
        </div>
        <p v-if="footerNote" class="text-text-tertiary mt-2 text-[12px]">
          {{ footerNote }}
        </p>
      </div>

      <!-- Footer -->
      <div class="border-border bg-muted/30 flex items-center justify-end gap-3 border-t px-5 py-4">
        <button
          @click="$emit('close')"
          class="text-text-secondary hover:text-text-primary border-border bg-surface hover:bg-hover rounded-lg border px-4 py-2 text-[13px] font-medium transition-colors"
        >
          Cancel
        </button>
        <button
          @click="handleSubmit"
          class="bg-primary hover:bg-primary-hover rounded-lg px-4 py-2 text-[13px] font-medium text-white shadow-sm transition-colors"
          :disabled="!inputValue.trim()"
        >
          {{ confirmLabel }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes scale-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
