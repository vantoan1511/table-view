import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useAboutStore = defineStore('about', () => {
  const isOpen = ref(false);

  const toggle = (val?: boolean) => {
    isOpen.value = val !== undefined ? val : !isOpen.value;
  };

  const open = () => toggle(true);
  const close = () => toggle(false);

  return {
    isOpen,
    toggle,
    open,
    close
  };
});
