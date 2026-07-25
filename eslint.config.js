import pluginVue from 'eslint-plugin-vue';
import vueTsConfig from '@vue/eslint-config-typescript';
import prettierConfig from '@vue/eslint-config-prettier';

export default [
  {
    name: 'app/files-to-lint',
    files: ['**/*.{js,mjs,cjs,ts,mts,vue}']
  },

  {
    name: 'app/files-to-ignore',
    ignores: [
      '**/dist/**',
      '**/dist-ssr/**',
      '**/coverage/**',
      'extensions/db-bridge/**',
      'bin/**',
      'build/**',
      '.tmp/**',
      '.storage/**',
      'test/**',
      '**/*.test.ts',
      '**/*.spec.ts',
      '**/__tests__/**'
    ]
  },

  ...pluginVue.configs['flat/essential'],
  ...vueTsConfig(),
  prettierConfig,

  {
    rules: {
      'linebreak-style': ['warn', 'windows'],
      'vue/multi-word-component-names': 'off',
      'no-console': 'off',
      'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'warn',
      '@typescript-eslint/no-explicit-any': 'off',
      // Disable base rule — TS-aware version handles all cases including imports
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          vars: 'all',
          args: 'after-used',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
          ignoreRestSiblings: true
        }
      ],
      '@typescript-eslint/no-unused-expressions': [
        'error',
        {
          allowShortCircuit: true,
          allowTernary: true,
          allowTaggedTemplates: true
        }
      ],
      'vue/no-mutating-props': 'warn',
      '@typescript-eslint/ban-ts-comment': 'warn'
    }
  }
];
