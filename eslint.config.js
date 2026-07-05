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
      '.storage/**'
    ]
  },

  ...pluginVue.configs['flat/essential'],
  ...vueTsConfig(),
  prettierConfig,

  {
    rules: {
      'linebreak-style': ['warn', 'windows'],
      'vue/multi-word-component-names': 'off',
      'no-console': process.env.NODE_ENV === 'production' ? 'error' : 'warn',
      'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'warn',
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_'
        }
      ],
      'vue/no-mutating-props': 'warn',
      '@typescript-eslint/ban-ts-comment': 'warn'
    }
  }
];
