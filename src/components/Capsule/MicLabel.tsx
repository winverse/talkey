import { useEffect, useState } from 'react'
import { Mic } from 'lucide-react'
import { motion } from 'framer-motion'
import { getInputDeviceName } from '../../lib/tauri'

/** Small label above the capsule naming the microphone that is listening. */
export function MicLabel({ capsuleWidth }: { capsuleWidth: number }) {
  const [name, setName] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    getInputDeviceName()
      .then((device) => {
        if (!cancelled) setName(device)
      })
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [])

  if (!name) return null

  return (
    <motion.div
      initial={{ opacity: 0, y: 3 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.18, ease: [0.2, 0, 0, 1] }}
      className="pointer-events-none absolute left-0 flex justify-center px-1"
      // Span the whole window (capsule + 12px padding each side) for long names.
      style={{ width: capsuleWidth + 24, bottom: 'calc(50% + 24px)' }}
      title={name}
    >
      <div className="capsule-label flex h-[20px] max-w-full items-center gap-1 rounded-full px-2 text-[10.5px] font-medium text-white/85">
        <Mic size={10} strokeWidth={2.4} className="flex-shrink-0 text-white/60" />
        <span className="truncate">{name}</span>
      </div>
    </motion.div>
  )
}
