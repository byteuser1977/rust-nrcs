import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from '@/router'
import App from '@/App.vue'

// 引入全局样式
import '@/assets/styles/index.scss'

// 引入 Element Plus
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'

// 引入 i18n 和语言包
import { i18n } from '@/locales'

// 创建应用实例
const app = createApp(App)

// 1. Pinia 状态管理
const pinia = createPinia()
app.use(pinia)

// 2. Vue Router
app.use(router)

// 3. Vue I18n（国际化）
app.use(i18n)

// 4. Element Plus（全局注册）
app.use(ElementPlus, {
  size: 'default',
  zIndex: 3000
})

// 挂载应用
app.mount('#app')

// 开发环境日志
if (import.meta.env.DEV) {
  console.log(
    '%c NRCS Platform %c Vue 3 + TypeScript + Vite ',
    'background:#409eff;color:white;padding:4px;border-radius:4px 0 0 4px;',
    'background:#333;color:white;padding:4px;border-radius:0 4px 4px 0;'
  )
}

// 错误处理（开发环境报错更友好）
app.config.errorHandler = (err, instance, info) => {
  console.error('[Vue Error]', err)
  console.error('Component:', instance)
  console.error('Info:', info)
}

// 未处理的 Promise reject 警告
app.config.warnHandler = (msg, instance, trace) => {
  console.warn('[Vue Warn]', msg)
  if (instance) {
    console.warn('Component:', instance)
  }
  if (trace) {
    console.warn('Trace:', trace)
  }
}
