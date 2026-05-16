// src/composables/useContextMenu.ts
import { ref } from 'vue';

export interface ContextMenuState {
  show: boolean;
  x: number;
  y: number;
  data: any;
}

export function useContextMenu<T = any>() {
  const contextMenu = ref<ContextMenuState>({
    show: false,
    x: 0,
    y: 0,
    data: null as any
  });

  const openContextMenu = (
    event: MouseEvent,
    data?: T,
    options: { width?: number; height?: number } = {}
  ) => {
    event.preventDefault();
    event.stopPropagation();

    const menuWidth = options.width || 192;
    const menuHeight = options.height || 160;

    let x = event.clientX;
    let y = event.clientY;

    // Adjust if near window boundaries
    if (x + menuWidth > window.innerWidth) {
      x -= menuWidth;
    }
    if (y + menuHeight > window.innerHeight) {
      y -= menuHeight;
    }

    contextMenu.value = {
      show: true,
      x,
      y,
      data: data as any
    };
  };

  const closeContextMenu = () => {
    contextMenu.value.show = false;
  };

  return {
    contextMenu,
    openContextMenu,
    closeContextMenu
  };
}
