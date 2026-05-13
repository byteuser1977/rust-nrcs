import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN'
import enUS from './en-US'

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS
  }
})

export { i18n }

export async function loadLanguageAsync(lang: string): Promise<void> {
  if (lang === 'zh-CN' || lang === 'en-US') {
    i18n.global.locale.value = lang
  } else {
    console.warn(`Unsupported language: ${lang}`)
  }
}

export { zhCN, enUS }
