import { config } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { createApp } from 'vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import { createI18n } from 'vue-i18n'
import en from '@/locales/en.ts'
import zhCn from '@/locales/zh-cn.ts'

// Mock global fetch
global.fetch = async () => {
  return {
    ok: true,
    json: async () => ({})
  } as Response
}

config.global.mocks = {
  // 可以在这里添加 Vue 生态的全局 mock
}

// 创建 i18n 实例
const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'en',
  messages: { 'zh-CN': zhCn, en }
})

// 创建路由器
const routes = [
  {
    path: '/',
    name: 'home',
    component: { template: '<div>Home</div>' }
  }
]
const router = createRouter({
  history: createWebHashHistory(),
  routes
})

// 导出测试辅助函数
export function createTestContext() {
  const pinia = createPinia()
  
  return {
    pinia,
    router,
    i18n
  }
}