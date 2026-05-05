import './assets/main.css'

import { useErrorStore } from '@/stores/error'
import * as Neutralino from '@neutralinojs/lib'
import { createPinia } from 'pinia'
import { createApp } from 'vue'
import { initLogger, setupConsoleOverride } from '@/utils/logger'

import App from './App.vue'

// Initialize central logging before anything else
initLogger()
setupConsoleOverride()

const app = createApp(App)
const pinia = createPinia()
const errorStore = useErrorStore(pinia)

app.use(pinia)

app.config.errorHandler = (err, instance, info) => {
  console.error('Vue Error:', err, info)
  errorStore.showError('An unexpected error occurred', err)
}

window.addEventListener('error', (event) => {
  console.error('Global Error:', event.error)
  errorStore.showError(event.message || 'An unexpected error occurred', event.error)
})

window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled Rejection:', event.reason)
  errorStore.showError('An unhandled promise rejection occurred', event.reason)
})

// Initialize Neutralino if running in Neutralino environment
if (window.NL_PORT) {
  Neutralino.init()

  // Set window title with version (non-blocking)
  const setWindowTitle = async () => {
    const config = await Neutralino.app.getConfig();
    if (config && config.version) {
      await Neutralino.window.setTitle(`${config.applicationName} v${config.version}`);
    }
  }
  setWindowTitle()
}

app.mount('#app')
