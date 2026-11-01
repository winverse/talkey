import { describe, expect, it } from 'vitest'
import type { DailyUsage } from '../tauri'
import {
  buildActivityGrid,
  formatCompact,
  splitDuration,
  summarizeUsage,
  TYPING_WPM,
} from '../usageStats'

const day = (date: string, entries: number, words = entries * 10, duration_ms = 0): DailyUsage => ({
  date,
  entries,
  words,
  chars: words * 3,
  duration_ms,
})

describe('summarizeUsage', () => {
  const today = new Date(2026, 10, 1) // 2026-11-01 (Sunday)

  it('counts streaks ending today or yesterday', () => {
    const days = [
      day('2026-10-20', 1),
      day('2026-10-21', 2),
      day('2026-10-22', 1),
      day('2026-10-30', 3),
      day('2026-10-31', 1),
    ]
    const summary = summarizeUsage(days, today)
    expect(summary.activeDays).toBe(5)
    expect(summary.longestStreak).toBe(3)
    expect(summary.currentStreak).toBe(2)
  })

  it('breaks the current streak after a missed day', () => {
    const summary = summarizeUsage([day('2026-10-29', 1)], today)
    expect(summary.currentStreak).toBe(0)
    expect(summary.longestStreak).toBe(1)
  })

  it('derives WPM and time saved from words and duration', () => {
    const summary = summarizeUsage([day('2026-11-01', 2, 120, 60_000)], today)
    expect(summary.wpm).toBe(120)
    expect(summary.savedMs).toBe((120 / TYPING_WPM) * 60_000 - 60_000)
    expect(summarizeUsage([], today)).toMatchObject({ words: 0, wpm: 0, savedMs: 0 })
  })
})

describe('buildActivityGrid', () => {
  it('ends with the week containing today and marks future and pre-start days', () => {
    const today = new Date(2026, 9, 28) // Wednesday
    const grid = buildActivityGrid([day('2026-10-27', 4), day('2026-10-26', 1)], today, 6)
    expect(grid.weeks).toHaveLength(6)
    const last = grid.weeks[5]
    expect(last[0].date).toBe('2026-10-25') // Sunday
    expect(last[2].level).toBe(4)
    expect(last[1].level).toBe(1)
    expect(last[4].future).toBe(true)
    expect(grid.weeks[0][0].beforeStart).toBe(true)
    expect(grid.monthStarts.some((m) => m.date === '2026-10-01')).toBe(true)
  })

  it('pages back in time with weekOffset', () => {
    const today = new Date(2026, 9, 28)
    const grid = buildActivityGrid([], today, 2, 1)
    expect(grid.weeks[1][0].date).toBe('2026-10-18')
  })
})

describe('formatting', () => {
  it('formats compact counts and durations', () => {
    expect(formatCompact(15_700)).toBe('15.7K')
    expect(formatCompact(269)).toBe('269')
    expect(splitDuration((8 * 60 + 21) * 60_000 + 5_000)).toEqual({ hours: 8, minutes: 21 })
  })
})
