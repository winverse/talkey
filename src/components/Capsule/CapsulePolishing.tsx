import { useTranslation } from 'react-i18next'
import { abortRecording } from '../../lib/tauri'
import { useCancelWithNotice } from './useCancelWithNotice'
import { CancelGlyph } from './CancelGlyph'

export function CapsulePolishing() {
  const { t } = useTranslation()
  const cancel = useCancelWithNotice(abortRecording)

  return (
    <div className="relative z-10 flex h-9 items-center gap-1.5 pl-4 pr-2">
      <p className="capsule-shimmer-text min-w-0 flex-1 truncate text-[12px] font-medium leading-none">
        {t('capsule.thinking')}
      </p>
      <CancelGlyph label={t('capsule.cancelPolishing')} onClick={cancel} />
    </div>
  )
}
