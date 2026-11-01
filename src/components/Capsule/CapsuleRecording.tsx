import { useTranslation } from 'react-i18next'
import { Check, X } from 'lucide-react'
import { abortRecording } from '../../lib/tauri'
import { useRecording } from '../../hooks/useRecording'
import { useCancelWithNotice } from './useCancelWithNotice'
import { Waveform } from './Waveform'
import { TranslateTargetChip } from './TranslateTargetChip'
import { CapsuleButton } from './CapsuleButton'

export function CapsuleRecording() {
  const { t } = useTranslation()
  const { stopRecording } = useRecording()
  const cancel = useCancelWithNotice(abortRecording)

  return (
    <div className="relative z-10 flex h-9 items-center gap-2 px-[5px]">
      <CapsuleButton label={t('capsule.cancelRecording')} tone="quiet" onClick={cancel}>
        <X size={13} strokeWidth={2.6} />
      </CapsuleButton>
      <div className="flex min-w-0 flex-1 justify-center">
        <Waveform bars={14} minHeight={2} maxHeight={16} />
      </div>
      <TranslateTargetChip />
      <CapsuleButton
        label={t('capsule.finishRecording')}
        tone="primary"
        onClick={() => {
          void stopRecording().catch((err) => console.error('Failed to stop recording:', err))
        }}
      >
        <Check size={14} strokeWidth={3} />
      </CapsuleButton>
    </div>
  )
}
