import { House, History, BookText, CircleUser, Settings, Info } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useRoute, type PageRoute } from '../../lib/router'
import { llmModelLabel, sttEngineLabel } from '../../lib/engineLabels'
import { useAppStore } from '../../stores/appStore'
import appIcon from '../../assets/brand/app-icon.png'
import { AccessibilityBanner } from './AccessibilityBanner'
import { CloudServiceBanner } from './CloudServiceBanner'

const navItems: { id: PageRoute; labelKey: string; icon: typeof House }[] = [
  { id: 'home', labelKey: 'nav.home', icon: House },
  { id: 'history', labelKey: 'nav.history', icon: History },
  { id: 'dictionary', labelKey: 'nav.dictionary', icon: BookText },
]

interface Props {
  children: React.ReactNode
}

export function MainLayout({ children }: Props) {
  const { page, navigate, openSettings } = useRoute()
  const { t } = useTranslation()

  return (
    <div className="w-full h-full flex bg-bg-primary text-text-primary">
      <aside className="w-[208px] shrink-0 flex flex-col bg-bg-sidebar border-r border-border">
        {/* Brand */}
        <div className="flex items-center gap-2 px-4 pt-5 pb-5" data-tauri-drag-region>
          <img src={appIcon} alt="" className="h-[22px] w-[22px] rounded-[6px]" draggable={false} />
          <h1 className="text-[15px] font-semibold tracking-tight">{t('app.name')}</h1>
        </div>

        {/* Main navigation */}
        <nav className="flex-1 px-2.5 space-y-0.5" aria-label={t('nav.mainNavigation')}>
          {navItems.map(({ id, labelKey, icon: Icon }) => {
            const active = page === id
            return (
              <button
                key={id}
                type="button"
                onClick={() => navigate(id)}
                aria-current={active ? 'page' : undefined}
                className={`flex items-center gap-2.5 w-full h-9 px-3 text-[13.5px] rounded-[8px] border-none cursor-pointer text-left transition-colors ${
                  active
                    ? 'jelly-nav-active text-text-primary font-medium'
                    : 'bg-transparent text-text-secondary hover:text-text-primary hover:bg-black/[0.04] dark:hover:bg-white/[0.05]'
                }`}
              >
                <Icon size={16} strokeWidth={active ? 2.1 : 1.8} />
                {t(labelKey)}
              </button>
            )
          })}
        </nav>

        <EngineCard onOpen={() => openSettings('stt')} />

        {/* Footer icons */}
        <div className="flex items-center justify-between px-3 pb-3 pt-1">
          <IconButton
            label={t('nav.account')}
            active={page === 'account'}
            onClick={() => navigate('account')}
          >
            <CircleUser size={16} />
          </IconButton>
          <div className="flex items-center gap-0.5">
            <IconButton label={t('nav.settings')} onClick={() => openSettings()}>
              <Settings size={16} />
            </IconButton>
            <IconButton label={t('sidebar.help')} onClick={() => openSettings('about')}>
              <Info size={16} />
            </IconButton>
          </div>
        </div>
      </aside>

      <main className="flex-1 min-w-0 flex flex-col">
        <AccessibilityBanner />
        <CloudServiceBanner />
        <div className="flex-1 overflow-y-auto">{children}</div>
      </main>
    </div>
  )
}

function IconButton({
  label,
  active = false,
  onClick,
  children,
}: {
  label: string
  active?: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={label}
      title={label}
      className={`flex h-8 w-8 items-center justify-center rounded-[8px] border-none cursor-pointer transition-colors ${
        active
          ? 'jelly-nav-active text-text-primary'
          : 'bg-transparent text-text-secondary hover:text-text-primary hover:bg-black/[0.04] dark:hover:bg-white/[0.05]'
      }`}
    >
      {children}
    </button>
  )
}

/** Replaces the upstream upgrade card: shows which engines this free build is using. */
function EngineCard({ onOpen }: { onOpen: () => void }) {
  const { t } = useTranslation()
  const config = useAppStore((s) => s.config)
  const stt = sttEngineLabel(
    config.stt_provider,
    config.stt_custom_model,
    config.stt_custom_base_url,
  )
  const polish = config.polish_enabled ? llmModelLabel(config.llm_model) : t('sidebar.off')

  return (
    <button
      type="button"
      onClick={onOpen}
      className="mx-2.5 mb-2 rounded-[12px] border border-border bg-bg-primary px-3 py-2.5 text-left cursor-pointer transition-colors hover:border-black/15 dark:hover:border-white/15"
    >
      <p className="text-[11.5px] font-medium text-text-primary">{t('sidebar.engines')}</p>
      <dl className="mt-1.5 space-y-1 text-[11.5px]">
        <div className="flex items-center justify-between gap-2">
          <dt className="text-text-tertiary shrink-0">{t('sidebar.stt')}</dt>
          <dd className="m-0 flex min-w-0 items-center gap-1 text-text-secondary" title={stt.name}>
            <span className="truncate">{stt.short}</span>
            {stt.local && (
              <span className="shrink-0 rounded-[4px] bg-accent-light px-1 text-[10px] font-medium text-accent">
                {t('sidebar.local')}
              </span>
            )}
          </dd>
        </div>
        <div className="flex items-center justify-between gap-2">
          <dt className="text-text-tertiary shrink-0">{t('sidebar.polish')}</dt>
          <dd className="m-0 truncate text-text-secondary">{polish}</dd>
        </div>
      </dl>
      <p className="mt-2 flex items-center gap-1.5 text-[10.5px] text-text-tertiary">
        <span className="h-1.5 w-1.5 rounded-full bg-accent" />
        {t('sidebar.freeNote')}
      </p>
    </button>
  )
}
