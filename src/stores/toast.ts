import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ToastSeverity = 'success' | 'error' | 'warning' | 'info';
export type ToastPosition =
  | 'top-right'
  | 'top-left'
  | 'bottom-right'
  | 'bottom-left'
  | 'top-center'
  | 'bottom-center';
export type ToastVariation = 'filled' | 'outlined' | 'subtle';

export interface ToastOptions {
  id?: string;
  title?: string;
  message: string;
  severity?: ToastSeverity;
  position?: ToastPosition;
  variation?: ToastVariation;
  ttl?: number; // time to live in ms, 0 means manual close
  icon?: any; // component
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<ToastOptions[]>([]);

  const addToast = (options: Omit<ToastOptions, 'id'>) => {
    const id = crypto.randomUUID();
    const toast: ToastOptions = {
      id,
      severity: 'info',
      position: 'bottom-right',
      variation: 'subtle',
      ttl: 5000,
      ...options
    };
    toasts.value.push(toast);

    if (toast.ttl && toast.ttl > 0) {
      setTimeout(() => {
        removeToast(id);
      }, toast.ttl);
    }
    return id;
  };

  const removeToast = (id: string) => {
    const idx = toasts.value.findIndex((t) => t.id === id);
    if (idx !== -1) {
      toasts.value.splice(idx, 1);
    }
  };

  const clearAll = () => {
    toasts.value = [];
  };

  return { toasts, addToast, removeToast, clearAll };
});
