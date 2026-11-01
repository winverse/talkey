/** Input level, in dBFS, drawn as silence (dots) and as full-height bars. */
const SILENCE_DB = -55
const FULL_DB = -15

/**
 * Maps the RMS level the backend reports (0..1, linear) to a 0..1 bar level
 * on a dB scale. Speech is usually around 0.01-0.1 RMS, which a linear scale
 * would draw as dots.
 */
export function levelFromRms(rms: number): number {
  if (!(rms > 0)) return 0
  const db = 20 * Math.log10(rms)
  return Math.max(0, Math.min(1, (db - SILENCE_DB) / (FULL_DB - SILENCE_DB)))
}
