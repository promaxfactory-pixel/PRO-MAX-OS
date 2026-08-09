import tseslint from '@typescript-eslint/eslint-plugin';
import tsParser from '@typescript-eslint/parser';
import reactPlugin from 'eslint-plugin-react';

const browserGlobals = {
  window: 'readonly',
  document: 'readonly',
  navigator: 'readonly',
  console: 'readonly',
  setTimeout: 'readonly',
  clearTimeout: 'readonly',
  setInterval: 'readonly',
  clearInterval: 'readonly',
  fetch: 'readonly',
  localStorage: 'readonly',
  sessionStorage: 'readonly',
  URL: 'readonly',
  URLSearchParams: 'readonly',
  crypto: 'readonly',
  requestAnimationFrame: 'readonly',
  cancelAnimationFrame: 'readonly',
  location: 'readonly',
  history: 'readonly',
  HTMLElement: 'readonly',
  HTMLInputElement: 'readonly',
  HTMLButtonElement: 'readonly',
  HTMLSelectElement: 'readonly',
  HTMLTextAreaElement: 'readonly',
  Event: 'readonly',
  KeyboardEvent: 'readonly',
  MouseEvent: 'readonly',
  CustomEvent: 'readonly',
  Node: 'readonly',
  NodeList: 'readonly',
  MutationObserver: 'readonly',
  FormData: 'readonly',
  File: 'readonly',
  Blob: 'readonly',
  AbortController: 'readonly',
  ResizeObserver: 'readonly',
  IntersectionObserver: 'readonly',
  Image: 'readonly',
};

export default [
  {
    ignores: ['node_modules/**', 'dist/**', 'src-tauri/**', 'scripts/**', '*.config.*'],
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: { jsx: true },
      },
      globals: browserGlobals,
    },
    plugins: {
      '@typescript-eslint': tseslint,
      react: reactPlugin,
    },
    settings: {
      react: {
        version: 'detect',
      },
    },
    rules: {
      ...tseslint.configs.recommended.rules,
      ...reactPlugin.configs.recommended.rules,
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }],
      'react/display-name': 'error',
      'react/jsx-key': 'error',
      'react/react-in-jsx-scope': 'off',
      'react/no-unescaped-entities': 'warn',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
    },
  },
];
