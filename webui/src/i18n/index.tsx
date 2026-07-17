import { createSignal, createContext, useContext, onMount, type ParentComponent } from 'solid-js'
import { zh, type TranslationKey } from './locales/zh'
import { en } from './locales/en'

type Locale = 'zh' | 'en'

const translations: Record<Locale, Record<TranslationKey, string>> = { zh, en }

function detectLocale(): Locale {
  const lang = navigator.language || 'en'
  return lang.startsWith('zh') ? 'zh' : 'en'
}

const I18nContext = createContext<{
  t: (key: TranslationKey) => string
  locale: () => Locale
}>()

export const I18nProvider: ParentComponent = (props) => {
  const [locale] = createSignal<Locale>(detectLocale())
  const t = (key: TranslationKey): string => translations[locale()][key] ?? zh[key]

  onMount(() => {
    document.documentElement.lang = locale() === 'zh' ? 'zh-CN' : 'en'
    document.title = locale() === 'zh' ? '战网账号切换' : 'BattleNetManager'
  })

  return (
    <I18nContext.Provider value={{ t, locale }}>
      {props.children}
    </I18nContext.Provider>
  )
}

export function useI18n() {
  const ctx = useContext(I18nContext)
  if (!ctx) throw new Error('useI18n must be used within I18nProvider')
  return ctx
}
