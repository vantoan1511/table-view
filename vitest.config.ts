import { fileURLToPath, URL } from 'node:url';
import { mergeConfig, defineConfig, configDefaults } from 'vitest/config';
import viteConfig from './vite.config';

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: 'jsdom',
      exclude: [...configDefaults.exclude, 'e2e/**'],
      root: fileURLToPath(new URL('./', import.meta.url)),
      coverage: {
        provider: 'v8',
        reporter: ['text', 'json', 'html'],
        exclude: [
          ...configDefaults.exclude,
          '**/node_modules/**',
          '**/dist/**',
          '**/build/**',
          '**/test/**',
          '**/*.config.ts',
          '**/*.d.ts'
        ]
      }
    }
  })
);
