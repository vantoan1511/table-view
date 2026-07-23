import './assets/main.css';
import 'primeicons/primeicons.css';

import { useErrorStore } from '@/stores/error';
import { useUpdaterStore } from '@/stores/updater';
import { initLogger, setupConsoleOverride } from '@/utils/logger';
import * as Neutralino from '@neutralinojs/lib';
import { TableViewTheme } from '@/theme/TableViewTheme';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import Tooltip from 'primevue/tooltip';
import { createApp } from 'vue';
import App from './App.vue';

// Initialize central logging before anything else
initLogger();
setupConsoleOverride();

const app = createApp(App);
const pinia = createPinia();
const errorStore = useErrorStore(pinia);

app.use(pinia);

app.use(PrimeVue, {
  ripple: true,
  theme: {
    preset: TableViewTheme,
    options: {
      darkModeSelector: '.dark',
      cssLayer: {
        name: 'primevue',
        order: 'theme, base, primevue'
      }
    }
  }
});

app.directive('tooltip', Tooltip);

app.config.errorHandler = (err, instance, info) => {
  console.error('Vue Error:', err, info);
  errorStore.showError('An unexpected error occurred', err);
};

window.addEventListener('error', (event) => {
  console.error('Global Error:', event.error);
  errorStore.showError(event.message || 'An unexpected error occurred', event.error);
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled Rejection:', event.reason);
  errorStore.showError('An unhandled promise rejection occurred', event.reason);
});

// Initialize Neutralino if running in Neutralino environment
if (window.NL_PORT) {
  Neutralino.init();

  // Set window title with active version (non-blocking)
  const setWindowTitle = async () => {
    try {
      const config = await Neutralino.app.getConfig();
      const updaterStore = useUpdaterStore(pinia);
      await updaterStore.init();
      const version = updaterStore.getCurrentAppVersion();
      const appName = config?.applicationName || 'Table View';
      await Neutralino.window.setTitle(`${appName} v${version}`);
    } catch (err) {
      console.warn('Failed to set window title with version:', err);
    }
  };
  setWindowTitle();
}

app.mount('#app');
