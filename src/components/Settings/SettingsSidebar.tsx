import { Settings, Mic, Sparkles, BookOpen, Info, LayoutGrid } from 'lucide-react'
import { useTranslation } from 'react-i18next'

const PANES = [
  { id: 'general', labelKey: 'settings.general', icon: Settings },
  { id: 'stt', labelKey: 'settings.speechRecognition', icon: Mic },
  { id: 'llm', labelKey: 'settings.aiPolish', icon: Sparkles },
  { id: 'dictionary', labelKey: 'settings.dictionary', icon: BookOpen },
  { id: 'scenes', labelKey: 'settings.scenes', icon: LayoutGrid },
  { id: 'about', labelKey: 'settings.about', icon: Info },
] as const

export type PaneId = (typeof PANES)[number]['id']

interface Props {
  activePane: PaneId
  onSelect: (id: PaneId) => void
  /** The main window has a dedicated Dictionary page, so the modal hides this pane. */
  hideDictionary?: boolean
}

export function SettingsSidebar({ activePane, onSelect, hideDictionary = false }: Props) {
  const { t } = useTranslation()
  const panes = hideDictionary ? PANES.filter((p) => p.id !== 'dictionary') : PANES

  return (
    <div className="flex h-full w-[214px] shrink-0 flex-col gap-0.5 bg-bg-sidebar px-3 py-5">
      {panes.map((pane) => {
        const Icon = pane.icon
        const isActive = activePane === pane.id
        return (
          <button
            key={pane.id}
            type="button"
            onClick={() => onSelect(pane.id)}
            aria-current={isActive ? 'page' : undefined}
            className={`flex h-9 w-full items-center gap-2.5 rounded-[8px] border-none px-3 text-left text-[13.5px] cursor-pointer transition-colors ${
              isActive
                ? 'jelly-nav-active font-medium text-text-primary'
                : 'bg-transparent text-text-secondary hover:bg-black/[0.04] hover:text-text-primary dark:hover:bg-white/[0.05]'
            }`}
          >
            <Icon size={16} strokeWidth={isActive ? 2.1 : 1.8} />
            {t(pane.labelKey)}
          </button>
        )
      })}
    </div>
  )
}
