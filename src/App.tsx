import { useEffect } from 'react'
import { useStateStore } from './store/State'
import { StateEnum } from './types/state'
import TrackPage from './pages/TrackPage'
import MainPage from './pages/MainPage'
import SearchPage from './pages/SearchPage'
import Toast from './components/Toast'
import { usePlayerStore } from './store/Player'
import { useQueueStore } from './store/Queue'
import { useLikedStore, useRecentStore } from './store/Db'
import { listen } from '@tauri-apps/api/event'
import { safeInvoke } from './services/invoke'
import type { PlaybackStateView } from './types/player'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
/**
 * Registers Tauri playback event listeners at app level (not tied to any UI
 * component lifecycle). The store actions are stable references obtained via
 * `getState()`, so no re-subscription is needed on state changes.
 */
function usePlaybackEvents() {
  useEffect(() => {
    let unlistenFuncs: Array<() => void> = []

    const setup = async () => {
      const subs = [
        listen<{ secs: number; nanos: number }>('play_progress', (e) => {
          usePlayerStore.getState().setProgress(e.payload.secs)
        }),
        listen('play_start', () => {
          usePlayerStore.getState().onPlaybackStarted()
        }),
        listen('play_end', () => {
          usePlayerStore.getState().onPlaybackEnded()
          useQueueStore.getState().handleEnded()
        }),
        listen<string>('play_failed', (e) => {
          usePlayerStore.getState().onPlaybackFailed(e.payload)
        }),
        listen<string>('db_tracks_changed', (e) => {
          if (e.payload === 'recent') void useRecentStore.getState().getRecentTracks()
        }),
      ]
      unlistenFuncs = await Promise.all(subs)

      // Snapshot reconciliation: covers events missed due to async listener registration.
      const snapshot = await safeInvoke<PlaybackStateView>('get_playback_state')
      if (snapshot) usePlayerStore.getState().onBackendState(snapshot)
    }

    void setup()
    return () => { unlistenFuncs.forEach((fn) => fn()) }
  }, [])
}

function useLibraryBootstrap() {
  useEffect(() => {
    void Promise.all([
      useRecentStore.getState().getRecentTracks(),
      useLikedStore.getState().getLikedTracks(),
    ]).then(([, likedLoaded]) => {
      if (!likedLoaded) return
      const player = usePlayerStore.getState()
      const currentTrack = player.currentTrack
      if (!currentTrack) return
      const liked = useLikedStore.getState().likedTracks.some((track) => track.id === currentTrack.id)
      player.syncLikedState(currentTrack.id, liked)
    })
  }, [])
}

function App() {
  const currentState = useStateStore((state) => state.currentState)
  usePlaybackEvents()
  useLibraryBootstrap()
  useKeyboardShortcuts()

  let page
  switch (currentState) {
    case StateEnum.detail:
      page = <TrackPage />
      break
    case StateEnum.searchResult:
      page = <SearchPage />
      break
    default:
      page = <MainPage />
  }

  return (
    <>
      {page}
      <p id="keyboard-shortcuts-description" className="sr-only" role="note">
        键盘快捷键：空格播放或暂停，左右方向键前后跳转 5 秒，上下方向键调节音量，M 静音，N 下一首，P 上一首，L 收藏或取消收藏。
      </p>
      <Toast />
    </>
  )
}

export default App
