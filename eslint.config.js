// 根级 flat ESLint 配置：覆盖所有 apps/* 与 packages/*（含 TS）。
import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
import vueParser from 'vue-eslint-parser'
import tseslint from 'typescript-eslint'
import globals from 'globals'

export default tseslint.config(
  {
    // 忽略：构建产物、依赖、后端、运行时数据、本地工作目录，以及各 app public/ 下的
    // 第三方 / 生成 / 压缩脚本（Live2D 模型、PixiJS 等，非本仓源码，不应 lint）。
    ignores: [
      '**/dist/**',
      '**/node_modules/**',
      'backend/**',
      'target/**',
      'target-linux/**',
      'data/**',
      'uploads/**',
      '.claude/**',
      '**/public/**',
      '**/*.min.js',
    ],
  },
  js.configs.recommended,
  ...vue.configs['flat/recommended'],
  // typescript-eslint 推荐规则（非类型检查版，无需 tsconfig project），作用于 TS / Vue。
  {
    files: ['**/*.ts', '**/*.tsx', '**/*.vue'],
    extends: [tseslint.configs.recommended],
  },
  {
    // .vue 必须用 vue-eslint-parser 作顶层解析器（解析模板），<script lang="ts"> 委托给 TS 解析器。
    // 放在 tseslint 之后，覆盖其对 .vue 设置的解析器，避免模板被当 TS 解析（'>' expected）。
    files: ['**/*.vue'],
    languageOptions: {
      parser: vueParser,
      parserOptions: { parser: tseslint.parser, ecmaVersion: 2023, sourceType: 'module' },
    },
  },
  {
    languageOptions: {
      ecmaVersion: 2023,
      sourceType: 'module',
      // 前端代码跑在浏览器、构建脚本跑在 node：声明二者全局，避免 no-undef 误报。
      globals: { ...globals.browser, ...globals.node },
    },
    rules: {
      'vue/multi-word-component-names': 'off',
      // 未用变量降为 warning（不阻断 CI；lint:js 仅 error 失败）。
      'no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      // 旧代码中存量的问题降为 warning：仍在 CI 输出/编辑器/CodeRabbit 中可见，但不阻断，
      // 避免为存量债大改工作正常的组件（新代码请尽量不要触发）。
      'prefer-const': 'warn',
      'no-empty': 'warn',
      'no-self-assign': 'warn',
      'no-irregular-whitespace': 'warn',
      'vue/no-mutating-props': 'warn',
      'vue/no-unused-vars': 'warn',
    },
  },
  {
    files: ['**/*.ts', '**/*.tsx', '**/*.vue'],
    rules: {
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-empty-object-type': 'off',
      '@typescript-eslint/ban-ts-comment': 'warn',
      '@typescript-eslint/no-unsafe-function-type': 'warn',
    },
  },
)
