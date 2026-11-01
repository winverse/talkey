import { useEffect, useMemo, useState } from 'react'
import { ArrowUpRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { getVersion } from '@tauri-apps/api/app'
import { useAppStore } from '../../stores/appStore'
import { useRoute } from '../../lib/router'
import { getDailyUsage, type DailyUsage } from '../../lib/tauri'
import { bindingKeys, llmModelLabel, sttEngineLabel } from '../../lib/engineLabels'
import { formatCompact, splitDuration, summarizeUsage } from '../../lib/usageStats'
import { ActivityGrid } from './ActivityGrid'

export function HomePage() {
  const { t } = useTranslation()
  const history = useAppStore((s) => s.history)
  const [usage, setUsage] = useState<DailyUsage[]>([])
  const [version, setVersion] = useState('')
  // Re-aggregate whenever a new dictation lands in the history list.
  const latestId = history[0]?.id ?? 0
  const today = useMemo(() => new Date(), [latestId]) // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => {
    getDailyUsage()
      .then((days) => setUsage(days ?? []))
      .catch((e) => console.error('Failed to load daily usage:', e))
  }, [latestId])

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(''))
  }, [])

  const summary = useMemo(() => summarizeUsage(usage, today), [usage, today])
  const saved = splitDuration(summary.savedMs)
  const total = splitDuration(summary.durationMs)

  return (
    <div className="relative min-h-full">
      <div className="dot-canvas pointer-events-none absolute inset-x-0 top-0 h-[560px]" />

      <div className="relative mx-auto max-w-[1040px] px-8 pt-9 pb-8">
        <h1 className="m-0 text-[30px] font-bold tracking-[-0.03em]">{t('home.heroTitle')}</h1>

        <div className="mt-7 grid gap-6 min-[1100px]:grid-cols-[minmax(0,1fr)_236px]">
          <section className="min-w-0 space-y-3">
            <h2 className="m-0 px-0.5 text-[17px] font-medium text-text-secondary">
              {t('home.insights')}
            </h2>

            <div className="grid grid-cols-2 gap-3">
              <StatCard label={t('home.wordsDictated')}>
                <Num>{formatCompact(summary.words)}</Num>
                <Unit>{t('home.unitWords')}</Unit>
              </StatCard>
              <StatCard label={t('home.timeSaved')} hint={t('home.timeSavedHint')}>
                <Duration hours={saved.hours} minutes={saved.minutes} />
              </StatCard>
              <StatCard label={t('home.avgSpeed')}>
                <Num>{summary.wpm}</Num>
                <Unit>WPM</Unit>
              </StatCard>
              <StatCard label={t('home.totalDictation')}>
                <Duration hours={total.hours} minutes={total.minutes} />
              </StatCard>
            </div>

            <div className="jelly-card p-5">
              <div className="grid grid-cols-3 gap-3 pb-4">
                <Streak value={summary.activeDays} label={t('home.activeDays')} />
                <Streak value={summary.currentStreak} label={t('home.currentStreak')} />
                <Streak value={summary.longestStreak} label={t('home.longestStreak')} />
              </div>
              <ActivityGrid usage={usage} today={today} />
            </div>

            <RecentCard />
          </section>

          <aside className="space-y-3">
            <ShortcutsCard />
            <EnginesCard />
          </aside>
        </div>

        {version && (
          <p className="mt-8 text-right text-[11px] text-text-tertiary">
            {t('home.version', { version: `v${version}` })}
          </p>
        )}
      </div>
    </div>
  )
}

function StatCard({
  label,
  hint,
  children,
}: {
  label: string
  hint?: string
  children: React.ReactNode
}) {
  return (
    <div className="jelly-card px-5 py-4" title={hint}>
      <div className="flex items-baseline gap-1">{children}</div>
      <p className="m-0 mt-1 text-[12.5px] text-text-secondary">{label}</p>
    </div>
  )
}

function Num({ children }: { children: React.ReactNode }) {
  return (
    <span className="text-[26px] font-semibold leading-none tracking-[-0.02em] tabular-nums">
      {children}
    </span>
  )
}

function Unit({ children }: { children: React.ReactNode }) {
  return <span className="mr-1 text-[12px] font-medium text-text-secondary">{children}</span>
}

function Duration({ hours, minutes }: { hours: number; minutes: number }) {
  const { t } = useTranslation()
  return (
    <>
      {hours > 0 && (
        <>
          <Num>{hours}</Num>
          <Unit>{t('home.unitHr')}</Unit>
        </>
      )}
      <Num>{minutes}</Num>
      <Unit>{t('home.unitMin')}</Unit>
    </>
  )
}

function Streak({ value, label }: { value: number; label: string }) {
  const { t } = useTranslation()
  return (
    <div>
      <div className="flex items-baseline gap-1">
        <span className="text-[20px] font-semibold leading-none tabular-nums">{value}</span>
        <Unit>{t('home.unitDays')}</Unit>
      </div>
      <p className="m-0 mt-1 text-[12px] text-text-secondary">{label}</p>
    </div>
  )
}

function RecentCard() {
  const { t } = useTranslation()
  const history = useAppStore((s) => s.history)
  const { navigate } = useRoute()
  const recent = history.filter((h) => h.polished_text.trim()).slice(0, 4)

  return (
    <div className="jelly-card px-5 py-4">
      <div className="flex items-center justify-between">
        <h3 className="m-0 text-[13px] font-medium">{t('home.recent')}</h3>
        <button
          type="button"
          onClick={() => navigate('history')}
          className="flex items-center gap-0.5 border-none bg-transparent p-0 text-[12px] text-text-secondary cursor-pointer hover:text-text-primary"
        >
          {t('home.viewAll')}
          <ArrowUpRight size={12} />
        </button>
      </div>
      {recent.length === 0 ? (
        <p className="m-0 mt-3 text-[12.5px] text-text-tertiary">{t('home.noRecent')}</p>
      ) : (
        <ul className="m-0 mt-2 list-none divide-y divide-border p-0">
          {recent.map((entry) => (
            <li key={entry.id} className="flex gap-3 py-2">
              <span className="w-10 shrink-0 pt-px text-[11.5px] tabular-nums text-text-tertiary">
                {entry.created_at.split('T')[1]?.slice(0, 5)}
              </span>
              <span className="min-w-0 flex-1 truncate text-[13px]">{entry.polished_text}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}

function ShortcutsCard() {
  const { t } = useTranslation()
  const hotkeys = useAppStore((s) => s.config.hotkeys)
  const rows = [
    {
      label: t('home.shortcutDictate'),
      bindings: hotkeys.dictationBindings?.length ? hotkeys.dictationBindings : [hotkeys.dictation],
    },
    {
      label: t('home.shortcutTranslate'),
      bindings: hotkeys.translateBindings?.length ? hotkeys.translateBindings : [hotkeys.translate],
    },
    {
      label: t('home.shortcutAsk'),
      bindings: hotkeys.askBindings?.length ? hotkeys.askBindings : [hotkeys.ask],
    },
  ]

  return (
    <div className="rounded-[14px] bg-bg-secondary px-4 py-3.5">
      <h3 className="m-0 text-[12px] font-medium text-text-secondary">{t('home.shortcuts')}</h3>
      <div className="mt-2.5 space-y-3">
        {rows.map(({ label, bindings }) => {
          const keys = bindingKeys(bindings[0])
          return (
            <div key={label}>
              <p className="m-0 text-[11.5px] text-text-secondary">{label}</p>
              <div className="mt-1 flex flex-wrap gap-1">
                {keys.length ? (
                  keys.map((k, i) => (
                    <kbd key={`${k}-${i}`} className="key-chip font-sans">
                      {k}
                    </kbd>
                  ))
                ) : (
                  <span className="text-[11.5px] text-text-tertiary">{t('home.notSet')}</span>
                )}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

function EnginesCard() {
  const { t } = useTranslation()
  const config = useAppStore((s) => s.config)
  const { openSettings } = useRoute()
  const stt = sttEngineLabel(
    config.stt_provider,
    config.stt_custom_model,
    config.stt_custom_base_url,
  )
  const styleKey: Record<string, string> = {
    minimal: 'settings.polishStyleMinimal',
    clean: 'settings.polishStyleClean',
    structured: 'settings.polishStyleStructured',
    professional: 'settings.polishStyleProfessional',
  }
  const rows: [string, string, 'stt' | 'llm'][] = [
    [t('sidebar.stt'), stt.local ? `${stt.name} (${t('sidebar.local')})` : stt.name, 'stt'],
    [
      t('sidebar.polish'),
      config.polish_enabled ? llmModelLabel(config.llm_model) : t('sidebar.off'),
      'llm',
    ],
    [
      t('home.polishStyle'),
      styleKey[config.polish_style] ? t(styleKey[config.polish_style]) : config.polish_style,
      'llm',
    ],
  ]

  return (
    <div className="rounded-[14px] bg-bg-secondary px-4 py-3.5">
      <h3 className="m-0 text-[12px] font-medium text-text-secondary">{t('home.currentConfig')}</h3>
      <div className="mt-1.5 space-y-0.5">
        {rows.map(([label, value, pane]) => (
          <button
            key={label}
            type="button"
            title={value}
            onClick={() => openSettings(pane)}
            className="-mx-1.5 block w-[calc(100%+12px)] rounded-[7px] border-none bg-transparent px-1.5 py-1.5 text-left cursor-pointer hover:bg-black/[0.04] dark:hover:bg-white/[0.05]"
          >
            <span className="block text-[11.5px] text-text-secondary">{label}</span>
            <span className="mt-0.5 block truncate text-[12.5px] font-medium text-text-primary">
              {value}
            </span>
          </button>
        ))}
      </div>
    </div>
  )
}
