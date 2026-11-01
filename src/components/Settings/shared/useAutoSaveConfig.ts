import { useCallback, useEffect, useRef } from 'react'
import { useAppStore } from '../../../stores/appStore'
import {
  getConfig,
  getHotkeyRegistrationError,
  setAutoStart,
  updateConfig,
} from '../../../lib/tauri'
import { toast } from '../../toast-service'
import { isConfigDirty, useDirtyConfig } from './useDirtyConfig'

/** Settings save this long after the last change, so typing saves once. */
export const AUTO_SAVE_DELAY_MS = 600

async function refreshHotkeyRegistrationError() {
  try {
    useAppStore.getState().setHotkeyRegistrationError(await getHotkeyRegistrationError())
  } catch (error) {
    console.error('[settings] failed to refresh hotkey registration error', error)
  }
}

let inFlight: Promise<void> | null = null

/**
 * Saves the settings draft to the backend. On failure it shows the error and
 * reloads the backend config, so the UI never drifts from what is stored.
 */
export function saveSettingsConfig(): Promise<void> {
  if (inFlight) return inFlight
  inFlight = (async () => {
    const { config, savedConfig, setConfig, setSavedConfig } = useAppStore.getState()
    const autoStartChanged = savedConfig !== null && savedConfig.auto_start !== config.auto_start
    let autoStartApplied = false
    try {
      if (autoStartChanged) {
        await setAutoStart(config.auto_start)
        autoStartApplied = true
      }
      await updateConfig(config)
      await refreshHotkeyRegistrationError()
      setSavedConfig(config)
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Failed to save settings'
      if (autoStartApplied && savedConfig) {
        await setAutoStart(savedConfig.auto_start).catch((rollbackError) => {
          console.error('[settings] failed to roll back auto-start', rollbackError)
        })
      }
      toast(msg, 'error')
      try {
        const backendConfig = await getConfig()
        setConfig(backendConfig)
        setSavedConfig(backendConfig)
      } catch {
        // Keep the draft if backend truth cannot be reloaded.
      }
      await refreshHotkeyRegistrationError()
    }
  })().finally(() => {
    inFlight = null
  })
  return inFlight
}

/**
 * Saves settings automatically: AUTO_SAVE_DELAY_MS after the last change,
 * and right away when Settings closes. Returns `flush` to save immediately.
 */
export function useAutoSaveConfig(): () => Promise<void> {
  const config = useAppStore((state) => state.config)
  const isDirty = useDirtyConfig()
  const timer = useRef<number | undefined>(undefined)

  useEffect(() => {
    if (!isDirty) return
    timer.current = window.setTimeout(() => void saveSettingsConfig(), AUTO_SAVE_DELAY_MS)
    return () => window.clearTimeout(timer.current)
  }, [config, isDirty])

  // Leaving Settings must not drop a change that is still waiting to save.
  useEffect(
    () => () => {
      if (isConfigDirty()) void saveSettingsConfig()
    },
    [],
  )

  return useCallback(async () => {
    window.clearTimeout(timer.current)
    if (isConfigDirty()) await saveSettingsConfig()
  }, [])
}
