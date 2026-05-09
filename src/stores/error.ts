import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useErrorStore = defineStore('error', () => {
  const show = ref(false);
  const message = ref('');
  const details = ref('');

  const showError = (msg: string, det?: string | Error | unknown) => {
    message.value = msg;
    if (det instanceof Error) {
      details.value = det.stack || det.message;
    } else if (typeof det === 'string') {
      details.value = det;
    } else if (det) {
      try {
        details.value = JSON.stringify(det, null, 2);
      } catch {
        details.value = String(det);
      }
    } else {
      details.value = '';
    }
    show.value = true;
  };

  const clearError = () => {
    show.value = false;
    message.value = '';
    details.value = '';
  };

  return { show, message, details, showError, clearError };
});
