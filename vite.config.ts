import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

const execArgv = process.allowedNodeEnvironmentFlags.has('--no-experimental-webstorage')
  ? ['--no-experimental-webstorage']
  : [];

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  base: './',
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom', 'axios', 'date-fns', 'lucide-react'],
        },
      },
    },
  },
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
    execArgv,
  },
});
