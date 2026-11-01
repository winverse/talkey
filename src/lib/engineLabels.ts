import type { ShortcutBinding } from '../stores/appStore'

const STT_NAMES: Record<string, string> = {
  'apple-speech': 'Apple Speech',
  'openai-whisper': 'OpenAI Whisper',
  'groq-whisper': 'Groq Whisper',
  'glm-asr': 'GLM-ASR',
  siliconflow: 'SiliconFlow',
  deepgram: 'Deepgram',
  assemblyai: 'AssemblyAI',
  cloud: 'OpenTypeless Cloud',
}

function isLocalUrl(url: string): boolean {
  return /^https?:\/\/(localhost|127\.0\.0\.1|\[::1\])(:|\/|$)/i.test(url.trim())
}

/** Human-readable speech engine name, e.g. "Whisper large-v3-turbo". */
export function sttEngineLabel(provider: string, customModel: string, customBaseUrl: string) {
  if (provider === 'custom-whisper') {
    const model = customModel.trim().split('/').pop() ?? ''
    const detail = model.replace(/^faster-whisper-/, '')
    return {
      name: detail ? `Whisper ${detail}` : 'Whisper',
      short: 'Whisper',
      local: isLocalUrl(customBaseUrl),
    }
  }
  const name = STT_NAMES[provider] ?? provider
  return { name, short: name, local: provider === 'apple-speech' }
}

/** "claude-haiku-5-5" → "Claude Haiku 5.5"; other ids are returned unchanged. */
export function llmModelLabel(model: string): string {
  const claude = /^claude-([a-z]+)-(\d+)(?:-(\d+))?$/.exec(model.trim())
  if (claude) {
    const family = claude[1][0].toUpperCase() + claude[1].slice(1)
    return `Claude ${family} ${claude[2]}${claude[3] ? `.${claude[3]}` : ''}`
  }
  return model.trim()
}

const isMac = typeof navigator !== 'undefined' && /mac/i.test(navigator.platform)

const MODIFIER_LABELS: Record<string, string> = {
  Command: 'Cmd',
  Cmd: 'Cmd',
  Meta: isMac ? 'Cmd' : 'Win',
  Super: isMac ? 'Cmd' : 'Win',
  Ctrl: 'Ctrl',
  Control: 'Ctrl',
  Alt: isMac ? 'Option' : 'Alt',
  Option: 'Option',
  Shift: 'Shift',
  Fn: 'Fn',
}

/** Key chips for a shortcut binding, e.g. ["Cmd", "Ctrl", "/"]. */
export function bindingKeys(binding: ShortcutBinding | null | undefined): string[] {
  if (!binding || !binding.primary) return []
  const keys = binding.modifiers.map((m) => MODIFIER_LABELS[m] ?? m)
  const primary = binding.primary.length === 1 ? binding.primary.toUpperCase() : binding.primary
  return [...keys, primary]
}
