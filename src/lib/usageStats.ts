import type { DailyUsage } from './tauri'

/** Typing speed used to estimate time saved by dictation. */
export const TYPING_WPM = 40

export interface UsageSummary {
  words: number
  entries: number
  durationMs: number
  activeDays: number
  currentStreak: number
  longestStreak: number
  /** Words per minute of recorded speech; 0 when no duration was recorded. */
  wpm: number
  /** Estimated typing time minus dictation time, never negative. */
  savedMs: number
}

export interface GridCell {
  date: string
  entries: number
  words: number
  /** 0 = none, 1–4 = relative activity */
  level: 0 | 1 | 2 | 3 | 4
  future: boolean
  beforeStart: boolean
}

export interface ActivityGrid {
  /** Columns of 7 cells (Sunday → Saturday), oldest first. */
  weeks: GridCell[][]
  /** Column index where a new month starts, with that month's first day. */
  monthStarts: { index: number; date: string }[]
}

const DAY_MS = 86_400_000

export function toYmd(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

export function parseYmd(ymd: string): Date {
  const [y, m, d] = ymd.split('-').map(Number)
  return new Date(y, m - 1, d)
}

function addDays(d: Date, days: number): Date {
  const next = new Date(d)
  next.setDate(d.getDate() + days)
  return next
}

export function summarizeUsage(days: DailyUsage[], today: Date): UsageSummary {
  let words = 0
  let entries = 0
  let durationMs = 0
  const active = new Set<string>()
  for (const day of days) {
    words += day.words
    entries += day.entries
    durationMs += day.duration_ms
    if (day.entries > 0) active.add(day.date)
  }

  const sorted = [...active].sort()
  let longestStreak = 0
  let run = 0
  let prev: Date | null = null
  for (const ymd of sorted) {
    const date = parseYmd(ymd)
    run = prev && Math.round((date.getTime() - prev.getTime()) / DAY_MS) === 1 ? run + 1 : 1
    longestStreak = Math.max(longestStreak, run)
    prev = date
  }

  // A streak stays alive through today even before today's first dictation.
  let currentStreak = 0
  let cursor = active.has(toYmd(today)) ? today : addDays(today, -1)
  while (active.has(toYmd(cursor))) {
    currentStreak += 1
    cursor = addDays(cursor, -1)
  }

  const minutes = durationMs / 60_000
  const wpm = minutes > 0 ? Math.round(words / minutes) : 0
  const savedMs = Math.max(0, Math.round((words / TYPING_WPM) * 60_000 - durationMs))

  return {
    words,
    entries,
    durationMs,
    activeDays: active.size,
    currentStreak,
    longestStreak,
    wpm,
    savedMs,
  }
}

function levelFor(entries: number, max: number): GridCell['level'] {
  if (entries <= 0 || max <= 0) return 0
  const ratio = entries / max
  if (ratio <= 0.25) return 1
  if (ratio <= 0.5) return 2
  if (ratio <= 0.75) return 3
  return 4
}

/**
 * Build a GitHub-style contribution grid ending in the week that contains `today`.
 * `weekCount` columns are produced; `weekOffset` shifts the window back in time.
 */
export function buildActivityGrid(
  days: DailyUsage[],
  today: Date,
  weekCount: number,
  weekOffset = 0,
): ActivityGrid {
  const byDate = new Map(days.map((d) => [d.date, d]))
  const firstDate = days.length ? [...byDate.keys()].sort()[0] : null
  const max = days.reduce((m, d) => Math.max(m, d.entries), 0)
  const todayYmd = toYmd(today)

  const lastSunday = addDays(today, -today.getDay() - weekOffset * 7)
  const start = addDays(lastSunday, -(weekCount - 1) * 7)

  const weeks: GridCell[][] = []
  const monthStarts: ActivityGrid['monthStarts'] = []
  for (let w = 0; w < weekCount; w++) {
    const column: GridCell[] = []
    for (let d = 0; d < 7; d++) {
      const date = addDays(start, w * 7 + d)
      const ymd = toYmd(date)
      const usage = byDate.get(ymd)
      if (date.getDate() === 1) monthStarts.push({ index: w, date: ymd })
      column.push({
        date: ymd,
        entries: usage?.entries ?? 0,
        words: usage?.words ?? 0,
        level: levelFor(usage?.entries ?? 0, max),
        future: ymd > todayYmd,
        beforeStart: firstDate === null || ymd < firstDate,
      })
    }
    weeks.push(column)
  }
  return { weeks, monthStarts }
}

/** 15700 → "15.7K" (Latin compact notation regardless of UI language). */
export function formatCompact(value: number): string {
  return new Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 1 }).format(
    value,
  )
}

export function splitDuration(ms: number): { hours: number; minutes: number } {
  const totalMinutes = Math.floor(ms / 60_000)
  return { hours: Math.floor(totalMinutes / 60), minutes: totalMinutes % 60 }
}
