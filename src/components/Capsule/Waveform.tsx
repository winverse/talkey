import { useEffect, useRef } from 'react'
import { useReducedMotion } from 'framer-motion'
import { useAppStore } from '../../stores/appStore'

interface WaveformProps {
  bars?: number
  minHeight?: number
  maxHeight?: number
}

/** Live input level as bars; at rest the bars shrink to dots. */
export function Waveform({ bars = 7, minHeight = 3, maxHeight = 16 }: WaveformProps) {
  const BAR_COUNT = bars
  const MIN_HEIGHT = minHeight
  const MAX_HEIGHT = maxHeight
  const barsRef = useRef<(HTMLDivElement | null)[]>([])
  const rafRef = useRef<number>(0)
  const reduced = useReducedMotion()

  useEffect(() => {
    if (reduced) {
      // Static bars at mid-height when reduced motion is preferred
      barsRef.current.forEach((bar) => {
        if (!bar) return
        bar.style.height = `${(MIN_HEIGHT + MAX_HEIGHT) / 2}px`
        bar.style.opacity = '0.7'
      })
      return
    }

    const animate = () => {
      const volume = useAppStore.getState().audioVolume
      barsRef.current.forEach((bar, i) => {
        if (!bar) return
        const offset = Math.sin(Date.now() / 200 + i * 0.9) * 0.15
        const normalized = Math.max(0, Math.min(1, volume + offset))
        const height = MIN_HEIGHT + (MAX_HEIGHT - MIN_HEIGHT) * normalized
        const opacity = Math.max(0.55, normalized)
        bar.style.height = `${height}px`
        bar.style.opacity = `${opacity}`
      })
      rafRef.current = requestAnimationFrame(animate)
    }

    rafRef.current = requestAnimationFrame(animate)
    return () => cancelAnimationFrame(rafRef.current)
  }, [reduced, MIN_HEIGHT, MAX_HEIGHT])

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
            opacity: 0.5,
            transition: 'height 75ms ease-out, opacity 75ms ease-out',
          }}
        />
      ))}
    </div>
  )
}
