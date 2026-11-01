import { useEffect, useMemo, useRef, useState } from 'react'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { DailyUsage } from '../../lib/tauri'
import { buildActivityGrid, parseYmd, type GridCell } from '../../lib/usageStats'

const CELL = 12
const GAP = 3
const LABEL_COL = 22
const MAX_WEEKS = 53

const LEVEL_CLASS = ['bg-grass-0', 'bg-grass-1', 'bg-grass-2', 'bg-grass-3', 'bg-grass-4'] as const

interface Props {
  usage: DailyUsage[]
  today: Date
}

/** GitHub-style contribution grid ("잔디") of daily dictations. */
export function ActivityGrid({ usage, today }: Props) {
  const { t, i18n } = useTranslation()
  const containerRef = useRef<HTMLDivElement | null>(null)
  const [weekCount, setWeekCount] = useState(26)
  const [offset, setOffset] = useState(0)
  const [hover, setHover] = useState<{ cell: GridCell; x: number; y: number } | null>(null)

  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    const measure = () => {
      const width = el.clientWidth - LABEL_COL
      setWeekCount(Math.max(8, Math.min(MAX_WEEKS, Math.floor((width + GAP) / (CELL + GAP)))))
    }
    measure()
    const observer = new ResizeObserver(measure)
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  const grid = useMemo(
    () => buildActivityGrid(usage, today, weekCount, offset),
    [usage, today, weekCount, offset],
  )

  const firstUsage = usage.length ? usage[0].date : null
  const canGoBack = firstUsage !== null && grid.weeks[0][0].date > firstUsage
  const locale = i18n?.language || 'en'
  const weekdayFmt = new Intl.DateTimeFormat(locale, { weekday: 'narrow' })
  const monthFmt = new Intl.DateTimeFormat(locale, { month: 'short' })
  const dateFmt = new Intl.DateTimeFormat(locale, { dateStyle: 'medium' })
  const weekdays = grid.weeks[0].map((cell) => weekdayFmt.format(parseYmd(cell.date)))

  return (
    <div ref={containerRef} className="relative select-none">
      {/* Month labels */}
      <div
        className="relative h-4 text-[10px] text-text-tertiary"
        style={{ marginLeft: LABEL_COL }}
      >
        {grid.monthStarts
          .filter(({ index }) => index <= weekCount - 3)
          .map(({ index, date }) => (
            <span
              key={date}
              className="absolute top-0 whitespace-nowrap"
              style={{ left: index * (CELL + GAP) }}
            >
              {monthFmt.format(parseYmd(date))}
            </span>
          ))}
      </div>

      <div className="flex" style={{ gap: GAP }}>
        {/* Weekday labels */}
        <div
          className="flex flex-col text-[10px] text-text-tertiary"
          style={{ width: LABEL_COL - GAP, gap: GAP }}
        >
          {weekdays.map((label, i) => (
            <span key={i} className="leading-none flex items-center" style={{ height: CELL }}>
              {i % 2 === 1 ? label : ''}
            </span>
          ))}
        </div>

        {grid.weeks.map((week) => (
          <div key={week[0].date} className="flex flex-col" style={{ gap: GAP }}>
            {week.map((cell) => (
              <div
                key={cell.date}
                role="gridcell"
                aria-label={cellLabel(cell)}
                onMouseEnter={(e) => {
                  if (cell.future) return
                  const box = e.currentTarget.getBoundingClientRect()
                  const host = containerRef.current!.getBoundingClientRect()
                  setHover({ cell, x: box.left - host.left + CELL / 2, y: box.top - host.top })
                }}
                onMouseLeave={() => setHover(null)}
                className={`rounded-[3px] ${
                  cell.future
                    ? 'bg-transparent'
                    : cell.level > 0
                      ? LEVEL_CLASS[cell.level]
                      : cell.beforeStart
                        ? 'grass-before'
                        : LEVEL_CLASS[0]
                }`}
                style={{ width: CELL, height: CELL }}
              />
            ))}
          </div>
        ))}
      </div>

      {/* Footer: paging + legend */}
      <div className="mt-3 flex items-center justify-between" style={{ marginLeft: LABEL_COL }}>
        <div className="flex items-center gap-1">
          <PageButton
            label="previous"
            disabled={!canGoBack}
            onClick={() => setOffset((o) => o + Math.max(4, Math.floor(weekCount / 2)))}
          >
            <ChevronLeft size={13} />
          </PageButton>
          <PageButton
            label="next"
            disabled={offset === 0}
            onClick={() =>
              setOffset((o) => Math.max(0, o - Math.max(4, Math.floor(weekCount / 2))))
            }
          >
            <ChevronRight size={13} />
          </PageButton>
        </div>
        <div className="flex items-center gap-1 text-[10.5px] text-text-tertiary">
          <span className="mr-1">{t('home.less')}</span>
          {LEVEL_CLASS.map((cls) => (
            <span key={cls} className={`${cls} rounded-[3px]`} style={{ width: 10, height: 10 }} />
          ))}
          <span className="ml-1">{t('home.more')}</span>
        </div>
      </div>

      {hover && (
        <div
          className="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-full whitespace-nowrap rounded-[7px] bg-text-primary px-2 py-1 text-[11px] font-medium text-bg-primary shadow-md"
          style={{ left: hover.x, top: hover.y - 6 }}
        >
          {cellLabel(hover.cell)}
        </div>
      )}
    </div>
  )

  function cellLabel(cell: GridCell) {
    const date = dateFmt.format(parseYmd(cell.date))
    return cell.entries > 0
      ? t('home.cellTooltip', { date, count: cell.entries, words: cell.words })
      : t('home.cellTooltipEmpty', { date })
  }
}

function PageButton({
  label,
  disabled,
  onClick,
  children,
}: {
  label: string
  disabled: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      type="button"
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      className="flex h-5 w-5 items-center justify-center rounded-[5px] border-none bg-transparent text-text-tertiary cursor-pointer hover:bg-bg-secondary hover:text-text-primary disabled:cursor-default disabled:opacity-30 disabled:hover:bg-transparent"
    >
      {children}
    </button>
  )
}
