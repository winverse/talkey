/**
 * Lets the tray's "Add Word to Dictionary" item focus the word field, even
 * when the dictionary page has not mounted yet.
 */
const FOCUS_EVENT = 'talkey:focus-dictionary-word'

let pending = false

export function requestDictionaryWordFocus() {
  pending = true
  window.dispatchEvent(new Event(FOCUS_EVENT))
}

/** Calls `focus` now if a request is pending, and on every later request. */
export function onDictionaryWordFocus(focus: () => void): () => void {
  const handle = () => {
    pending = false
    focus()
  }
  if (pending) handle()
  window.addEventListener(FOCUS_EVENT, handle)
  return () => window.removeEventListener(FOCUS_EVENT, handle)
}
