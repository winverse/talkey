import { useState, useEffect, useRef } from 'react'
import { motion } from 'framer-motion'
import { X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useAppStore } from '../../stores/appStore'
import { SettingsSidebar, type PaneId } from './SettingsSidebar'
import { GeneralPane } from './GeneralPane'
import { SttPane } from './SttPane'
import { LlmPane } from './LlmPane'
import { DictionaryPane } from './DictionaryPane'
import { ScenesPane } from './ScenesPane'
import { AboutPane } from './AboutPane'
import { useAutoSaveConfig } from './shared/useAutoSaveConfig'

const paneTitleKeys: Record<PaneId, string> = {
  general: 'settings.general',
  stt: 'settings.speechRecognition',
  llm: 'settings.aiPolish',
  dictionary: 'settings.dictionary',
  scenes: 'settings.scenes',
  about: 'settings.about',
}

function paneFromHash(): PaneId | null {
  const query = window.location.hash.split('?')[1]
  if (!query) return null
  const requested = new URLSearchParams(query).get('pane')
  return requested && requested in paneTitleKeys ? (requested as PaneId) : null
}

interface Props {
  /** When set, settings render as a modal dialog over the current page. */
  onClose?: () => void
}

export function Settings({ onClose }: Props) {
  const [activePane, setActivePane] = useState<PaneId>(() => paneFromHash() ?? 'general')
  const contentRef = useRef<HTMLDivElement | null>(null)
  const config = useAppStore((s) => s.config)
  const setSavedConfig = useAppStore((s) => s.setSavedConfig)
  const flushSave = useAutoSaveConfig()
  const { t } = useTranslation()

  // First-run onboarding may enter Settings before MainApp has established backend truth.
  useEffect(() => {
    if (useAppStore.getState().savedConfig === null) setSavedConfig(config)
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const syncRequestedPane = () => {
      const requested = paneFromHash()
      if (requested) setActivePane(requested)
    }
    window.addEventListener('hashchange', syncRequestedPane)
    return () => window.removeEventListener('hashchange', syncRequestedPane)
  }, [])

  useEffect(() => {
    contentRef.current?.scrollTo?.({ top: 0 })
  }, [activePane])

  // Changes save automatically; closing saves anything still pending first.
  const requestClose = () => {
    if (!onClose) return
    void flushSave().finally(onClose)
  }

  useEffect(() => {
    if (!onClose) return
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || event.defaultPrevented) return
      // A nested dialog (import, app mapping, menus) owns Escape while it is open.
      if (document.querySelectorAll('[role="dialog"], [role="menu"]').length > 1) return
      requestClose()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  })

  const body = (
    <div className="flex h-full min-h-0 flex-col bg-bg-primary text-text-primary">
      <div className="flex min-h-0 flex-1">
        <SettingsSidebar
          activePane={activePane}
          onSelect={setActivePane}
          hideDictionary={Boolean(onClose)}
        />

        <div className="relative flex min-w-0 flex-1 flex-col">
          {onClose && (
            <button
              type="button"
              onClick={requestClose}
              aria-label={t('settings.close')}
              className="absolute right-4 top-4 z-10 flex h-7 w-7 items-center justify-center rounded-[7px] border-none bg-transparent text-text-secondary cursor-pointer hover:bg-bg-secondary hover:text-text-primary"
            >
              <X size={16} />
            </button>
          )}

          <div ref={contentRef} className="flex-1 overflow-y-auto overflow-x-hidden px-9 pb-8 pt-8">
            <h2 className="m-0 mb-6 text-[22px] font-semibold tracking-[-0.02em]">
              {t(paneTitleKeys[activePane])}
            </h2>
            <motion.div
              key={activePane}
              className="w-full"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.1, ease: 'easeOut' }}
            >
              {activePane === 'general' && <GeneralPane />}
              {activePane === 'stt' && <SttPane />}
              {activePane === 'llm' && <LlmPane />}
              {activePane === 'dictionary' && <DictionaryPane />}
              {activePane === 'scenes' && <ScenesPane />}
              {activePane === 'about' && <AboutPane />}
            </motion.div>
          </div>
        </div>
      </div>
    </div>
  )

  if (!onClose) return <div className="h-full w-full">{body}</div>

  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-6"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.12 }}
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) requestClose()
      }}
    >
      <motion.div
        role="dialog"
        aria-modal="true"
        aria-label={t('settings.title')}
        className="h-[min(800px,100%)] w-[min(1080px,100%)] overflow-hidden rounded-[16px] shadow-float"
        initial={{ opacity: 0, scale: 0.98, y: 6 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        transition={{ duration: 0.16, ease: 'easeOut' }}
      >
        {body}
      </motion.div>
    </motion.div>
  )
}
