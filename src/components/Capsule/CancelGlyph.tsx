import { X } from 'lucide-react'

/** Small cancel button for the quieter capsule states. */
export function CancelGlyph({ label, onClick }: { label: string; onClick: () => void }) {
  const stop = (event: React.PointerEvent) => event.stopPropagation()
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onPointerDown={stop}
      onPointerUp={stop}
      onClick={(event) => {
        event.stopPropagation()
        onClick()
      }}
      className="grid h-5 w-5 flex-shrink-0 cursor-pointer place-items-center rounded-full border-none bg-transparent p-0 text-white/45 transition-colors hover:bg-white/10 hover:text-white"
    >
      <X size={11} strokeWidth={2.6} />
    </button>
  )
}
