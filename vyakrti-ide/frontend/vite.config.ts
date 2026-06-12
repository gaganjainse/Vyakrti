import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/compile': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        timeout: 15000,
        proxyTimeout: 15000,
      },
      '/ws': {
        target: 'ws://127.0.0.1:8080',
        ws: true,
        timeout: 15000,
      },
    },
  },
})
