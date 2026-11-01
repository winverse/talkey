import { useTranslation } from 'react-i18next'
import { Check, MessageCircle, X } from 'lucide-react'
import { abortAskDictation, stopAskFlow } from '../../lib/tauri'
import { CapsuleButton } from './CapsuleButton'
import { Waveform } from './Waveform'

export function CapsuleAskRecording() {
  const { t } = useTranslation()

  const cancel = () => {
    void abortAskDictation().catch((error) => {
      console.error('Failed to abort Ask recording:', error)
    })
  }

  const finish = () => {
    void stopAskFlow().catch((error) => {
      console.error('Failed to stop Ask flow:', error)
    })
  }

  return (
    <div className="relative z-10 flex h-9 items-center gap-2 px-[5px]">
      <CapsuleButton label={t('capsule.cancelRecording')} tone="quiet" onClick={cancel}>
        <X size={13} strokeWidth={2.6} />
      </CapsuleButton>
      <MessageCircle size={13} className="shrink-0 text-white/70" aria-label={t('ask.title')} />
      <div className="flex min-w-0 flex-1 justify-center">
        <Waveform bars={10} minHeight={2} maxHeight={16} />
      </div>
      <CapsuleButton label={t('capsule.finishRecording')} tone="primary" onClick={finish}>
        <Check size={14} strokeWidth={3} />
      </CapsuleButton>
    </div>
  )
}
