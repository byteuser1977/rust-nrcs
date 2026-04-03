import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    // 自动导入 Vue 相关函数（ref, computed, etc.）
    AutoImport({
      resolvers: [ElementPlusResolver()],
      imports: ['vue', 'vue-router', 'pinia'],
      dts: 'src/auto-imports.d.ts'
    }),
    // 自动导入 Element Plus 组件
    Components({
      resolvers: [ElementPlusResolver()],
      dts: 'src/components.d.ts'
    })
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@assets': resolve(__dirname, 'src/assets'),
      '@components': resolve(__dirname, 'src/components'),
      '@composables': resolve(__dirname, 'src/composables'),
      '@stores': resolve(__dirname, 'src/stores'),
      '@api': resolve(__dirname, 'src/api'),
      '@types': resolve(__dirname, 'src/types'),
      '@utils': resolve(__dirname, 'src/utils'),
      '@locales': resolve(__dirname, 'src/locales')
    }
  },
  server: {
    port: 3000,
    open: true,
    proxy: {
      // 开发环境代理到后端 API
      '/api': {
        target: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '/api/v1')
      },
      // WebSocket 代理（如果使用）
      '/ws': {
        target: import.meta.env.VITE_WS_URL || 'ws://localhost:8081',
        changeOrigin: true,
        ws: true
      }
    }
  },
  build: {
    target: 'es2015',
    outDir: 'dist',
    sourcemap: true,  // 生产环境 source map（便于调试）
    rollupOptions: {
      output: {
        manualChunks: {
          // 将大型库拆分为独立 chunk
          vendor: ['vue', 'vue-router', 'pinia'],
          'element-plus': ['element-plus', '@element-plus/icons-vue'],
          'charts': ['echarts', 'vue-echarts']
        }
      }
    },
    // 代码分割配置
    chunkSizeWarningLimit: 1000, // 1MB
    // 压缩配置
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: import.meta.env.PROD, // 生产环境移除 console
        drop_debugger: import.meta.env.PROD
      }
    }
  },
  // CSS 预处理器配置
  css: {
    preprocessorOptions: {
      scss: {
        // 全局 SCSS 变量
        additionalData: `@use "@/assets/styles/variables.scss" as *;`,
        // 支持 Element Plus 主题定制
        api: 'modern-compiler'
      }
    }
  },
  // 环境变量
  envPrefix: 'VITE_'
})
