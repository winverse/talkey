import { useState, useEffect, useCallback } from 'react'

export type Route = 'home' | 'settings' | 'history' | 'dictionary' | 'upgrade' | 'account'

/** Routes that render as a full page. `settings` opens as a modal over the last page. */
export type PageRoute = Exclude<Route, 'settings'>

export function parseHash(): Route {
  const hash = window.location.hash.replace('#/', '')
  if (hash === 'history' || hash === 'dictionary' || hash === 'upgrade' || hash === 'account')
    return hash
  if (hash === 'settings' || hash.startsWith('settings?')) return 'settings'
  return 'home'
}

let lastPage: PageRoute = 'home'

function rememberPage(route: Route) {
  if (route !== 'settings') lastPage = route
}

rememberPage(parseHash())

export function useRoute() {
  const [route, setRoute] = useState<Route>(parseHash)

  useEffect(() => {
    const onHashChange = () => {
      const next = parseHash()
      rememberPage(next)
      setRoute(next)
    }
    window.addEventListener('hashchange', onHashChange)
    return () => window.removeEventListener('hashchange', onHashChange)
  }, [])

  const navigate = useCallback((r: Route) => {
    window.location.hash = r === 'home' ? '#/' : `#/${r}`
  }, [])

  /** Open the settings modal, optionally on a specific pane. */
  const openSettings = useCallback((pane?: string) => {
    window.location.hash = pane ? `#/settings?pane=${pane}` : '#/settings'
  }, [])

  /** Close the settings modal and return to the page underneath. */
  const closeSettings = useCallback(() => {
    window.location.hash = lastPage === 'home' ? '#/' : `#/${lastPage}`
  }, [])

  const page: PageRoute = route === 'settings' ? lastPage : route

  return { route, page, navigate, openSettings, closeSettings }
}
