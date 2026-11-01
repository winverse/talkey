interface Props {
  label: string
  children: React.ReactNode
}

export function FormField({ label, children }: Props) {
  return (
    <div>
      <label className="block text-[13.5px] font-semibold text-text-primary mb-2">{label}</label>
      {children}
    </div>
  )
}
