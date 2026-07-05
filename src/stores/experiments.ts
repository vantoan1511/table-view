import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useExperimentsStore = defineStore('experiments', () => {
  const enabled = ref(false);

  const toggleEnabled = (val?: boolean) => {
    enabled.value = val !== undefined ? val : !enabled.value;
  };

  return {
    enabled,
    toggleEnabled
  };
});
