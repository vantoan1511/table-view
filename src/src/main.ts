import './assets/main.css'

import { createApp } from 'vue'
import { init } from "@neutralinojs/lib"
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')

init();
