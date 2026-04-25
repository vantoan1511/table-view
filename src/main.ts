import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import * as Neutralino from '@neutralinojs/lib'

import App from './App.vue'

const app = createApp(App)

app.use(createPinia())

app.mount('#app')

// Initialize Neutralino if running in Neutralino environment
if (window.NL_PORT) {
  Neutralino.init()

  Neutralino.events.on('extensionReady', (evt) => {
    console.log('Extension ready:', evt.detail)
  })
}
