import { useTranslation } from 'react-i18next'
import { DictionaryPane } from '../Settings/DictionaryPane'

/** Top-level Dictionary page (the pane used to live inside Settings). */
export function DictionaryPage() {
  const { t } = useTranslation()
  return (
    <div className="mx-auto max-w-[880px] px-8 pt-9 pb-10">
      <h1 className="m-0 mb-6 text-[28px] font-bold tracking-[-0.03em]">
        {t('settings.dictionary')}
      </h1>
      <DictionaryPane />
    </div>
  )
}
