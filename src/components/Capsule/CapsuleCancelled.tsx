import { useTranslation } from 'react-i18next'
import { Info } from 'lucide-react'

export function CapsuleCancelled() {
  const { t } = useTranslation()
  return (
    <div className="relative z-10 flex h-9 items-center justify-center gap-2 px-4">
      <Info size={14} strokeWidth={2.4} className="flex-shrink-0 text-[#5b9bff]" />
      <span className="truncate text-[12px] font-semibold text-white">
        {t('capsule.cancelled')}
      </span>
    </div>
  )
}
