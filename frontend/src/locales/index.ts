import { createI18n } from 'vue-i18n'
import type { I18n } from 'vue-i18n'
import zhCN from './zh-CN'
import enUS from './en-US'

// 创建 i18n 实例
const i18n = createI18n<I18n.Composer>({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS
  }
})

export { i18n }

/**
 * 异步加载语言包（用于动态切换）
 */
export async function loadLanguageAsync(lang: string): Promise<void> {
  // 这里可以根据需要动态导入大型语言包
  // 当前已全部导入，所以无需额外操作
  if (lang === 'zh-CN' || lang === 'en-US') {
    i18n.global.locale.value = lang
  } else {
    console.warn(`Unsupported language: ${lang}`)
  }
}

export { zhCN, enUS }
