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

// 引入动态常量 Store（启动时拉取 getConstants）
import { useConstantsStore } from '@/stores/modules'

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

// 5. 启动时拉取服务端动态常量（getConstants）。
//    非阻塞：失败不阻断渲染，Store 内部记录 loadError 并保持 loaded=false，
//    依赖常量的判定函数在未加载时会按参考语义隐式放行。
//    对标参考 nrs.js 初始化阶段的 NRS.loadServerConstants() 调用。
useConstantsStore().loadServerConstants().catch((err) => {
  console.error('[bootstrap] loadServerConstants failed:', err)
})

// 6. 设置通知系统集成（对标 nrs.js 中 $.growl 的全局调用）。
//    注册 account.store 资产变化回调 + node.store 分叉/连接错误 watcher。
//    必须在 Pinia 安装后调用（app.mount 之前）。
import { setupNotificationIntegration } from '@/composables/useNotificationIntegration'
setupNotificationIntegration()

// 7. 初始化远程节点管理器（对标 nrs.remote.nodes.js:35 的 NRS.initRemoteNodesMgr）。
//    仅在移动端模拟或 API 代理场景启用 bootstrap/updateRemoteNodes；
//    Web 端直连本地节点时无操作。confirmResponse 钩子在此注册，
//    但仅当 requestNeedsConfirmation 返回 true 时才触发远程验证。
//    非阻塞：失败不阻断渲染。
import { initRemoteNodesMgr } from '@/utils/remote-nodes'
import { getMobileSettings } from '@/utils/remote-nodes'
import { getFeatureContext } from '@/utils/feature-detection'
const isTestnet = getMobileSettings().is_testnet || !!getFeatureContext().isTestNet
initRemoteNodesMgr(isTestnet).catch((err) => {
  console.warn('[bootstrap] initRemoteNodesMgr failed:', err)
})

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
