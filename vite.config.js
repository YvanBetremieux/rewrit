import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { resolve } from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        preview: resolve(__dirname, 'src/preview.html'),
        quickselect: resolve(__dirname, 'src/quickselect.html'),
      },
    },
  },
})
