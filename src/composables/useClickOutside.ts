// src/composables/useClickOutside.ts
import { onMounted, onUnmounted, type Ref } from 'vue';

export function useClickOutside(
  elRef: Ref<HTMLElement | null>,
  callback: (event: MouseEvent) => void,
  excludeSelectors: string[] = []
) {
  const handler = (event: MouseEvent) => {
    if (!elRef.value) return;

    const target = event.target as HTMLElement;

    // Check if clicked inside the element
    if (elRef.value.contains(target)) return;

    // Check if clicked inside any excluded elements
    for (const selector of excludeSelectors) {
      if (target.closest(selector)) return;
    }

    callback(event);
  };

  onMounted(() => {
    window.addEventListener('mousedown', handler);
  });

  onUnmounted(() => {
    window.removeEventListener('mousedown', handler);
  });
}
