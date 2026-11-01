import { useEffect, useRef } from 'react'
import { useReducedMotion } from 'framer-motion'
import { useAppStore } from '../../stores/appStore'
import { levelFromRms } from './audioLevel'

interface WaveformProps {
  bars?: number
  minHeight?: number
  maxHeight?: number
}

/** How often the waveform scrolls one bar to the left. */
const SHIFT_MS = 60

/**
 * Live input level as a scrolling waveform: the newest level enters on the
 * right, so speech shows as moving peaks and silence as a row of dots.
 */
export function Waveform({ bars = 7, minHeight = 3, maxHeight = 16 }: WaveformProps) {
  const BAR_COUNT = bars
  const MIN_HEIGHT = minHeight
  const MAX_HEIGHT = maxHeight
  const barsRef = useRef<(HTMLDivElement | null)[]>([])
  const rafRef = useRef<number>(0)
  const reduced = useReducedMotion()

  useEffect(() => {
    const paint = (index: number, level: number) => {
      const bar = barsRef.current[index]
      if (!bar) return
      bar.style.height = `${MIN_HEIGHT + (MAX_HEIGHT - MIN_HEIGHT) * level}px`
      bar.style.opacity = `${0.55 + 0.45 * level}`
    }

    if (reduced) {
      // No scrolling: every bar follows the current level together.
      const unsubscribe = useAppStore.subscribe((state) => {
        const level = levelFromRms(state.audioVolume)
        for (let i = 0; i < BAR_COUNT; i++) paint(i, level)
      })
      return unsubscribe
    }

    const history: number[] = new Array(BAR_COUNT).fill(0)
    let smoothed = 0
    let lastShift = 0

    const animate = (now: number) => {
      const target = levelFromRms(useAppStore.getState().audioVolume)
      // Rise quickly when speech starts, fall back more slowly.
      smoothed += (target - smoothed) * (target > smoothed ? 0.6 : 0.2)
      if (now - lastShift >= SHIFT_MS) {
        history.shift()
        history.push(smoothed)
        lastShift = now
        history.forEach((level, i) => paint(i, level))
      }
      rafRef.current = requestAnimationFrame(animate)
    }

    rafRef.current = requestAnimationFrame(animate)
    return () => cancelAnimationFrame(rafRef.current)
  }, [reduced, BAR_COUNT, MIN_HEIGHT, MAX_HEIGHT])

  return (
    <div className="flex items-center justify-center gap-[3px]" style={{ height: MAX_HEIGHT }}>
      {Array.from({ length: BAR_COUNT }).map((_, i) => (
        <div
          key={i}
          ref={(el) => {
            barsRef.current[i] = el
          }}
          className="w-[2px] rounded-full bg-white/80"
          style={{
            height: `${MIN_HEIGHT}px`,
            opacity: 0.55,
            transition: 'height 60ms linear, opacity 60ms linear',
          }}
        />
      ))}
    </div>
  )
}
