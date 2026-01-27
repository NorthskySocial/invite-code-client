/// <reference types="vitest" />
import {defineConfig} from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  base: './',
  server: {
    host: 'localhost',
    allowedHosts: ['frontend.myapp.local'],
    hmr: {
      protocol: 'ws',
      host: 'localhost',
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
  },
})
