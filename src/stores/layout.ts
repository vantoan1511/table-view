import { defineStore } from 'pinia'
import type { LayoutState, Panel } from '@/types'
import * as Neutralino from '@neutralinojs/lib'

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
          { id: 'value', title: 'Value', component: 'ValueViewer' },
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
    bottomPanel: (state) => state.panels['console'],
    rightPanel: (state) => state.panels['inspector'],
    isBottomVisible: (state) => state.panels['console']?.isVisible ?? false,
    isRightVisible: (state) => state.panels['inspector']?.isVisible ?? false,
  },

  actions: {
    async init() {
      if (window.NL_PORT) {
        try {
          const saved = await Neutralino.storage.getData('layout')
          const layout = JSON.parse(saved)
          
          if (layout && typeof layout === 'object') {
            if (layout.panels) {
              Object.keys(layout.panels).forEach((id) => {
                const panel = this.panels[id]
                const savedPanel = layout.panels[id]
                if (panel && savedPanel) {
                  if (typeof savedPanel.isVisible === 'boolean') {
                    panel.isVisible = savedPanel.isVisible
                  }
                  if (typeof savedPanel.size === 'number') {
                    panel.size = savedPanel.size
                  }
                }
              })
            }
            if (typeof layout.sidebarWidth === 'number') {
              this.sidebarWidth = layout.sidebarWidth
            }
            if (typeof layout.isSidebarVisible === 'boolean') {
              this.isSidebarVisible = layout.isSidebarVisible
            }
          }
        } catch (err) {
          // Storage item might not exist yet or be invalid JSON
        }
      }
    },

    async save() {
      if (window.NL_PORT) {
        try {
          const panelsToSave: Record<string, { isVisible: boolean, size: number }> = {}
          Object.keys(this.panels).forEach(id => {
            const panel = this.panels[id]
            if (panel) {
              panelsToSave[id] = {
                isVisible: panel.isVisible,
                size: panel.size
              }
            }
          })

          const layout = {
            panels: panelsToSave,
            sidebarWidth: this.sidebarWidth,
            isSidebarVisible: this.isSidebarVisible
          }
          await Neutralino.storage.setData('layout', JSON.stringify(layout))
        } catch (err) {
          console.error('Failed to save layout state:', err)
        }
      }
    },

    async togglePanel(id: string) {
      const panel = this.panels[id]
      if (panel) {
        panel.isVisible = !panel.isVisible
        await this.save()
      }
    },

    async updatePanelSize(id: string, size: number) {
      const panel = this.panels[id]
      if (panel) {
        const minSize = 100
        const maxSize = panel.position === 'bottom' ? window.innerHeight * 0.7 : window.innerWidth * 0.5
        panel.size = Math.max(minSize, Math.min(maxSize, size))
        await this.save()
      }
    },

    async setSidebarWidth(width: number) {
      this.sidebarWidth = Math.max(150, Math.min(500, width))
      await this.save()
    },

    async toggleSidebar() {
      this.isSidebarVisible = !this.isSidebarVisible
      await this.save()
    },
  },
})
