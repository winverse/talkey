import { useAppStore } from '../../stores/appStore'

/** Shows "cancelled" in the capsule for a moment after the user cancels. */
export function useCancelWithNotice(abort: () => Promise<void>) {
  const setCapsuleNotice = useAppStore((s) => s.setCapsuleNotice)
  return async () => {
    try {
      await abort()
      setCapsuleNotice('cancelled')
      window.setTimeout(() => {
        if (useAppStore.getState().capsuleNotice === 'cancelled') setCapsuleNotice(null)
      }, 1800)
    } catch (err) {
      console.error('Failed to cancel:', err)
    }
  }
}
