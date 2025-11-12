import { mergeConfig, defineConfig } from 'vitest/config'
import viteReact from '@vitejs/plugin-react'
import { TanStackRouterVite } from '@tanstack/router-plugin/vite'
import { resolve } from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import { RoutesJsonPlugin } from './vite.axum'
import baseConfig from '../../../vitest.config.base'


// https://vitejs.dev/config/
export default mergeConfig(baseConfig, defineConfig({
  plugins: [
    TanStackRouterVite({ autoCodeSplitting: true }),
    viteReact(),
    tailwindcss(),
    RoutesJsonPlugin(),
  ],
  test: {
    environment: 'jsdom',
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  server: {
    // port: 3000,
    // host: '127.0.0.1',
    hmr: {
      port: 21012
      // clientPort: process.env.HMR_PORT ? parseInt(process.env.HMR_PORT) : 21013,
    },
    allowedHosts: ['localhost', '127.0.0.1'],
  },
  build: {
    target: 'esnext',
  },
}))

