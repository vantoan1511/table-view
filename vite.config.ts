import { fileURLToPath, URL } from 'node:url';

import { PrimeVueResolver } from '@primevue/auto-import-resolver';
import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import Components from 'unplugin-vue-components/vite';
import { defineConfig } from 'vite';
import vueDevTools from 'vite-plugin-vue-devtools';

const isProd = process.env.NODE_ENV === 'production';

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    Components({
      resolvers: [PrimeVueResolver()]
    }),
    // Exclude devtools from production bundle
    ...(!isProd ? [vueDevTools()] : [])
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    outDir: 'build',
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Vue core runtime
          if (id.includes('node_modules/vue/') || id.includes('node_modules/@vue/')) {
            return 'vendor-vue';
          }
          // Pinia state management
          if (id.includes('node_modules/pinia')) {
            return 'vendor-vue';
          }
          // CodeMirror editor — isolated, only loads when SQL tab is first opened
          if (
            id.includes('node_modules/codemirror') ||
            id.includes('node_modules/@codemirror') ||
            id.includes('node_modules/@lezer')
          ) {
            return 'vendor-codemirror';
          }
          // PrimeVue UI library + icons + themes
          if (
            id.includes('node_modules/primevue') ||
            id.includes('node_modules/primeicons') ||
            id.includes('node_modules/@primevue') ||
            id.includes('node_modules/@primeuix') ||
            id.includes('node_modules/tailwindcss-primeui')
          ) {
            return 'vendor-primevue';
          }
          // Lucide icon library
          if (id.includes('node_modules/lucide-vue-next')) {
            return 'vendor-lucide';
          }
          // Neutralino desktop bindings
          if (id.includes('node_modules/@neutralinojs')) {
            return 'vendor-neutralino';
          }
        }
      }
    }
  }
});
