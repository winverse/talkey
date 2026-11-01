// Dev-only preview: `npm run dev`, then open http://localhost:1420/preview.html#/
// Optional query: ?theme=dark&lang=en&empty=1
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks'
import { useAppStore, type HistoryEntry } from '../stores/appStore'
import type { DailyUsage } from '../lib/tauri'

const params = new URLSearchParams(window.location.search)
const empty = params.has('empty')
const lang = params.get('lang') ?? 'ko'
const dark = params.get('theme') === 'dark'

function seeded(seed: number) {
  let s = seed
  return () => {
    s = (s * 1664525 + 1013904223) % 4294967296
    return s / 4294967296
  }
}

function ymd(d: Date) {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

const SAMPLES = [
  'React 컴포넌트에서 useState를 쓸 때 리렌더링이 너무 많이 일어나는데, 어떻게 줄일 수 있는지 알려 줘',
  '이번 주 회의는 금요일로 미뤄졌어',
  'Docker Compose 파일에서 볼륨 설정이 잘못된 것 같은데 한번 봐 줄래?',
  '이 함수 이름을 getUserById로 바꿔 줘',
  '커밋하고 푸시해 줘',
  '그 학생에게 맞는 스타일로 공부할 수 있게 세팅을 해 주는 거지',
  '컴퓨터 과학 개론 5강 슬라이드와 복습 교재를 만들어 줘',
  '테스트 다 통과했으니까 이제 배포해도 돼',
]

function makeUsage(): DailyUsage[] {
  if (empty) return []
  const rand = seeded(7)
  const out: DailyUsage[] = []
  const today = new Date()
  for (let i = 150; i >= 0; i--) {
    const d = new Date(today)
    d.setDate(today.getDate() - i)
    if (rand() < 0.45) continue
    const entries = Math.ceil(rand() * rand() * 40)
    const words = entries * (6 + Math.floor(rand() * 10))
    out.push({ date: ymd(d), entries, words, chars: words * 3, duration_ms: words * 380 })
  }
  return out
}

function makeHistory(): HistoryEntry[] {
  if (empty) return []
  const now = Date.now()
  return Array.from({ length: 24 }, (_, i) => {
    const t = new Date(now - i * 47 * 60 * 1000)
    const text = SAMPLES[i % SAMPLES.length]
    return {
      id: 1000 - i,
      created_at: `${ymd(t)}T${String(t.getHours()).padStart(2, '0')}:${String(t.getMinutes()).padStart(2, '0')}:00`,
      context_profile_id: 'general.native',
      context_label: i % 3 === 0 ? 'Terminal' : 'General',
      context_icon_key: i % 3 === 0 ? 'terminal' : 'general',
      context_family: i % 3 === 0 ? 'prompt_or_code' : 'general',
      browser_access_status: 'not_applicable',
      provider_kind: 'byok',
      raw_text: `어 음 ${text}`,
      polished_text: text,
      language: 'ko',
      duration_ms: 4000,
      active_scene_id: null,
      active_scene_source: null,
      active_scene_name: null,
      active_scene_prompt_chars: null,
      active_scene_prompt_truncated: false,
      output_status: null,
      output_error: null,
    } as HistoryEntry
  })
}

const config = {
  ...useAppStore.getState().config,
  ui_language: lang,
  theme: dark ? 'dark' : 'light',
  stt_provider: 'custom-whisper',
  stt_custom_base_url: 'http://127.0.0.1:8000/v1',
  stt_custom_model: 'large-v3-turbo',
  llm_provider: 'claude',
  llm_model: 'claude-haiku-5-5',
  polish_enabled: true,
  polish_style: 'clean',
}

const dictionary = [
  'React',
  'useState',
  'useEffect',
  'TypeScript',
  'Next.js',
  'Node.js',
  'cargo build',
  'Docker Compose',
].map((word, i) => ({ id: i + 1, word, pronunciation: null }))

mockWindows('main')
mockIPC(
  (cmd, args) => {
    const a = (args ?? {}) as Record<string, unknown>
    switch (cmd) {
      case 'plugin:store|load':
        return 1
      case 'plugin:store|get':
        return a.key === 'onboarding_completed' ? [true, true] : [null, false]
      case 'get_config':
        return config
      case 'get_history':
        return makeHistory()
      case 'get_daily_usage':
        return makeUsage()
      case 'get_dictionary':
        return empty ? [] : dictionary
      case 'get_correction_rules':
        return []
      case 'check_accessibility_permission':
        return true
      case 'get_hotkey_registration_error':
        return null
      case 'plugin:app|version':
        return '1.1.64'
      default:
        return null
    }
  },
  { shouldMockEvents: true },
)

if (dark) document.documentElement.classList.add('dark')
localStorage.setItem('ui_language', lang)

await import('../main')
