import type {} from 'vite-plus'
import { defineConfig } from 'vite-plus'
import { svelte } from '@sveltejs/vite-plugin-svelte'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    setupFiles: ['./src/test-setup.ts'],
    server: {
      deps: {
        // Inline svelte so vitest resolves it with browser conditions
        inline: [/svelte/],
      },
    },
  },
  lint: {
    ignorePatterns: ['dist/**', 'src-tauri/**'],
    rules: {
      // Svelte bind:this assigns variables in the template, not in JS
      'no-unassigned-vars': 'off',
    },
  },
  fmt: {
    singleQuote: true,
    semi: false,
    ignorePatterns: ['dist/**', 'src-tauri/**', '**/*.svelte'],
  },
  resolve: {
    conditions: ['browser'],
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
