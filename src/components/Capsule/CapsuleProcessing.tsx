import { useTranslation } from 'react-i18next'
import { CancelGlyph } from './CancelGlyph'
import { abortRecording } from '../../lib/tauri'
import { useAppStore } from '../../stores/appStore'
import { useCancelWithNotice } from './useCancelWithNotice'

/** Transcribing: streams the partial transcript, or a shimmering "transcribing" label. */
export function CapsuleProcessing() {
  const { t } = useTranslation()
  const partialTranscript = useAppStore((s) => s.partialTranscript)
  const cancel = useCancelWithNotice(abortRecording)

  return (
    <div className="relative z-10 flex h-9 items-center gap-1.5 pl-4 pr-2">
      <p
        className={`min-w-0 flex-1 truncate text-[12px] font-medium leading-none ${
          partialTranscript ? 'text-white/85' : 'capsule-shimmer-text'
        }`}
      >
        {partialTranscript || t('capsule.transcribing')}
      </p>
      <CancelGlyph label={t('capsule.cancelProcessing')} onClick={cancel} />
    </div>
  )
}
