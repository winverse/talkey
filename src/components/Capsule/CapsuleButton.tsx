import type { ReactNode } from 'react'

interface CapsuleButtonProps {
  label: string
  tone: 'quiet' | 'primary'
  onClick: () => void
  disabled?: boolean
  children: ReactNode
}

/** Round button inside the capsule. Stops pointer events so the capsule does not drag or stop. */
export function CapsuleButton({ label, tone, onClick, disabled, children }: CapsuleButtonProps) {
  const stop = (event: React.PointerEvent) => event.stopPropagation()
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      disabled={disabled}
      onPointerDown={stop}
      onPointerUp={stop}
      onClick={(event) => {
        event.stopPropagation()
        onClick()
      }}
      className={`grid h-[26px] w-[26px] flex-shrink-0 place-items-center rounded-full border-none p-0 transition-[background-color,transform,opacity] duration-150 active:scale-95 disabled:opacity-40 disabled:active:scale-100 ${
        tone === 'primary'
          ? 'bg-white text-black hover:bg-white/90 cursor-pointer'
          : 'bg-white/[0.14] text-white/85 hover:bg-white/[0.22] hover:text-white cursor-pointer'
      }`}
    >
      {children}
    </button>
  )
}
