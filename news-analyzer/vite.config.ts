import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  base: './',
  server: {
    port: 1420,
  },
  build: {
    target: 'es2015',
    minify: 'terser',
    sourcemap: false
  }
})
