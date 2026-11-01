import { useTranslation } from 'react-i18next'
import { Check, X } from 'lucide-react'
import { abortRecording } from '../../lib/tauri'
import { CapsuleButton } from './CapsuleButton'
import { CapsuleWorkIndicator } from './CapsuleWorkIndicator'
import { useCancelWithNotice } from './useCancelWithNotice'

export function CapsulePreparing() {
  const { t } = useTranslation()
  const cancel = useCancelWithNotice(abortRecording)

  return (
    <div className="relative z-10 flex h-9 items-center gap-2 px-[5px]">
      <CapsuleButton label={t('capsule.cancelRecording')} tone="quiet" onClick={cancel}>
        <X size={13} strokeWidth={2.6} />
      </CapsuleButton>
      <div className="flex min-w-0 flex-1 justify-center" title={t('capsule.preparing')}>
        <CapsuleWorkIndicator tone="steady" />
      </div>
      <CapsuleButton
        label={t('capsule.finishRecording')}
        tone="primary"
        onClick={() => {}}
        disabled
      >
        <Check size={14} strokeWidth={3} />
      </CapsuleButton>
    </div>
  )
}
