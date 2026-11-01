import { useTranslation } from 'react-i18next'
import { motion } from 'framer-motion'
import { Check } from 'lucide-react'
import { useEffect } from 'react'
import { useAppStore } from '../../stores/appStore'
import { spring } from '../../lib/animations'

export function CapsuleComplete() {
  const { t } = useTranslation()
  const resetRecording = useAppStore((s) => s.resetRecording)
  const setPipelineState = useAppStore((s) => s.setPipelineState)

  useEffect(() => {
    const timer = setTimeout(() => {
      resetRecording()
      setPipelineState('idle')
    }, 1200)
    return () => clearTimeout(timer)
  }, [resetRecording, setPipelineState])

  return (
    <motion.div className="relative z-10 flex h-9 items-center justify-center gap-2 pl-[5px] pr-4">
      <motion.div
        initial={{ scale: 0, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={spring.smooth}
        className="grid h-[26px] w-[26px] place-items-center rounded-full bg-white text-black"
      >
        <Check size={14} strokeWidth={3} />
      </motion.div>
      <span className="text-[12px] font-semibold text-white">{t('capsule.done')}</span>
    </motion.div>
  )
}
