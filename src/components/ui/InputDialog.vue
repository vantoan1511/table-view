<script setup lang="ts">
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import { AlertCircle } from 'lucide-vue-next';
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
  <Dialog
    visible
    modal
    :header="title"
    :style="{ width: '26rem' }"
    :closable="true"
    @update:visible="(val) => { if (!val) emit('close'); }"
  >
    <div class="py-2">
      <div class="mb-2">
        <div class="mb-1.5 flex items-center justify-between">
          <label class="text-text-secondary block text-[13px] font-medium">{{ label }}</label>
          <span
            v-if="!inputValue.trim()"
            class="text-danger flex items-center gap-1 text-[11px] font-medium"
          >
            <AlertCircle :size="12" />
            Required
          </span>
        </div>
        <InputText
          v-model.trim="inputValue"
          type="text"
          :placeholder="placeholder"
          class="w-full"
          :invalid="!inputValue.trim()"
          @keyup.enter="handleSubmit"
          autofocus
        />
      </div>
      <p v-if="footerNote" class="text-text-tertiary mt-2 text-[12px]">
        {{ footerNote }}
      </p>
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2 pt-2">
        <Button variant="outlined" severity="secondary" @click="$emit('close')">
          Cancel
        </Button>
        <Button
          severity="primary"
          :disabled="!inputValue.trim()"
          @click="handleSubmit"
        >
          {{ confirmLabel }}
        </Button>
      </div>
    </template>
  </Dialog>
</template>

