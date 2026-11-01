/** Capsule sizes shared by the capsule view and the window resize hook. */

export type CapsuleViewState =
  | 'idle'
  | 'preparing'
  | 'recording'
  | 'transcribing'
  | 'polishing'
  | 'outputting'
  | 'ask_recording'
  | 'ask_thinking'
  | 'error'
  | 'cancelled'

export interface CapsuleShellSize {
  width: number
  height: number
}

export const CAPSULE_HEIGHT = 36

/** Space reserved above (and, to keep the capsule centred, below) for the microphone label. */
export const MIC_LABEL_SPACE = 26

export function getCapsuleShellSize(state: string, translating = false): CapsuleShellSize {
  const h = CAPSULE_HEIGHT
  switch (state) {
    case 'preparing':
    case 'recording':
      return { width: translating ? 190 : 156, height: h }
    case 'transcribing':
      return { width: 184, height: h }
    case 'polishing':
      return { width: 124, height: h }
    case 'outputting':
      return { width: 104, height: h }
    case 'ask_recording':
    case 'ask_thinking':
      return { width: 168, height: h }
    case 'cancelled':
      return { width: 176, height: h }
    case 'error':
      return { width: 200, height: h }
    default:
      return { width: 36, height: h }
  }
}

/** States that show the microphone name above the capsule. */
export function showsMicLabel(state: string): boolean {
  return state === 'preparing' || state === 'recording' || state === 'ask_recording'
}
