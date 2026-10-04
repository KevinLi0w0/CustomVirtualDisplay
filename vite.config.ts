import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'node:path'

const tauriDevHost = process.env.TAURI_DEV_HOST

export default defineConfig({
  // 用相对路径(base:'./')产物, 让打包后的 index.html 在 asset://localhost 等自定义
  // 协议下也能正确解析 ./assets/* (绝对路径 /assets/* 在部分 WebView2 环境下会失败)。
  base: './',
  root: resolve(__dirname, 'src/renderer'),
  plugins: [tailwindcss(), react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src/renderer')
    }
  },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: tauriDevHost ?? '127.0.0.1',
    hmr: tauriDevHost
      ? { protocol: 'ws', host: tauriDevHost, port: 1421 }
      : undefined,
    watch: { ignored: ['**/src-tauri/**', '**/native/**'] }
  },
  build: {
    outDir: resolve(__dirname, 'dist/renderer'),
    emptyOutDir: true,
    target: 'esnext',
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    minify: process.env.TAURI_ENV_DEBUG ? false : 'esbuild'
  }
})
