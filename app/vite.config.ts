import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

// NetTamer frontend build config.
// The Tauri shell (../src-tauri/tauri.conf.json) expects:
//   devUrl       -> http://localhost:5173
//   frontendDist -> ../frontend/dist
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'esnext',
    sourcemap: false,
  },
})
