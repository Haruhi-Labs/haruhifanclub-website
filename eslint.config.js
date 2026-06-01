// 根级 flat ESLint 配置：覆盖所有 apps/* 与 packages/*
import js from '@eslint/js'
import vue from 'eslint-plugin-vue'

export default [
  {
    ignores: ['**/dist/**', '**/node_modules/**', 'backend/**', 'target/**', 'data/**', 'uploads/**'],
  },
  js.configs.recommended,
  ...vue.configs['flat/recommended'],
  {
    languageOptions: {
      ecmaVersion: 2023,
      sourceType: 'module',
    },
    rules: {
      'vue/multi-word-component-names': 'off',
      'no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
    },
  },
]
