import { defineStore } from 'pinia'
import type { LayoutState, Panel, PanelPosition } from '@/types'

export const useLayoutStore = defineStore('layout', {
  state: (): LayoutState => ({
    panels: {
      console: {
        id: 'console',
        title: 'Console',
        position: 'bottom',
        isVisible: true,
        isMinimized: false,
        size: 300,
        activeTabId: 'output',
        tabs: [
          { id: 'output', title: 'Output', component: 'ConsoleOutput' },
          { id: 'timeline', title: 'Timeline', component: 'Timeline' },
        ],
      },
      inspector: {
        id: 'inspector',
        title: 'Inspector',
        position: 'right',
        isVisible: true,
        isMinimized: false,
        size: 350,
        activeTabId: 'properties',
        tabs: [
          { id: 'properties', title: 'Properties', component: 'TableProperties' },
          { id: 'indexes', title: 'Indexes', component: 'TableIndexes' },
        ],
      },
    },
    sidebarWidth: 260,
    isSidebarVisible: true,
  }),

  getters: {
    bottomPanel: (state) => Object.values(state.panels).find(p => p.position === 'bottom'),
    rightPanel: (state) => Object.values(state.panels).find(p => p.position === 'right'),
    visiblePanels: (state) => Object.values(state.panels).filter((p) => p.isVisible && !p.isMinimized),
    minimizedPanels: (state) => Object.values(state.panels).filter((p) => p.isMinimized),
  },

  actions: {
    togglePanel(id: string) {
      const panel = this.panels[id]
      if (panel) {
        panel.isVisible = !panel.isVisible
        if (panel.isVisible) panel.isMinimized = false
      }
    },

    minimizePanel(id: string) {
      const panel = this.panels[id]
      if (panel) {
        panel.isMinimized = true
      }
    },

    restorePanel(id: string) {
      const panel = this.panels[id]
      if (panel) {
        panel.isVisible = true
        panel.isMinimized = false
      }
    },

    movePanel(id: string, position: PanelPosition) {
      const panel = this.panels[id]
      if (!panel || panel.position === position) return

      // Find if another panel is occupying the target position to perform a swap
      const otherPanel = Object.values(this.panels).find(p => p.position === position)
      
      if (otherPanel) {
        // Swap positions and sizes
        const oldPos = panel.position
        const oldSize = panel.size
        
        otherPanel.position = oldPos
        otherPanel.size = oldSize
      }

      panel.position = position
      panel.size = position === 'bottom' ? 300 : 350
    },

    updatePanelSize(id: string, size: number) {
      const panel = this.panels[id]
      if (panel) {
        const minSize = 100
        const maxSize = panel.position === 'bottom' ? window.innerHeight * 0.7 : window.innerWidth * 0.5
        panel.size = Math.max(minSize, Math.min(maxSize, size))
      }
    },

    setSidebarWidth(width: number) {
      this.sidebarWidth = Math.max(150, Math.min(500, width))
    },

    toggleSidebar() {
      this.isSidebarVisible = !this.isSidebarVisible
    },
  },
})
