import { useEffect } from 'react'
import { usePlayerStore } from '../store/Player'
import { useQueueStore } from '../store/Queue'

const MAX_VOLUME = 50
const SEEK_STEP_SECONDS = 5
const VOLUME_STEP = 5
const DISCRETE_KEYS: Record<string, true> = {
  ' ': true,
  space: true,
  spacebar: true,
  mediaplaypause: true,
  m: true,
  n: true,
  mediatracknext: true,
  p: true,
  mediatrackprevious: true,
  l: true,
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return target.isContentEditable
    || target.tagName === 'INPUT'
    || target.tagName === 'TEXTAREA'
    || target.tagName === 'SELECT'
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value))
}

/** Registers the app-wide playback keyboard shortcuts. */
export function useKeyboardShortcuts(): void {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        isEditableTarget(event.target)
        || event.ctrlKey
        || event.metaKey
        || event.altKey
      ) return

      const key = event.code === 'Space' ? ' ' : event.key.toLowerCase()
      if (event.repeat && DISCRETE_KEYS[key] === true) return
      const player = usePlayerStore.getState()
      const queue = useQueueStore.getState()
      const currentTrack = player.currentTrack
      const hasCurrentTrack = currentTrack !== null
      let handled = false

      switch (key) {
        case ' ':
        case 'space':
        case 'spacebar':
        case 'mediaplaypause':
          if (!hasCurrentTrack) break
          void player.onTogglePlay()
          handled = true
          break
        case 'arrowleft':
        case 'arrowright': {
          const duration = currentTrack?.duration
          if (!hasCurrentTrack || duration === undefined || !Number.isFinite(duration) || duration < 0) break
          const currentTime = Number.isFinite(player.currentTime) ? player.currentTime : 0
          const offset = key === 'arrowleft' ? -SEEK_STEP_SECONDS : SEEK_STEP_SECONDS
          void player.onSeek(clamp(currentTime + offset, 0, duration))
          handled = true
          break
        }
        case 'arrowup':
        case 'arrowdown':
          if (!hasCurrentTrack) break
          {
            const currentVolume = Number.isFinite(player.volume) ? player.volume : 0
            const offset = key === 'arrowup' ? VOLUME_STEP : -VOLUME_STEP
            void player.onVolumeChange(clamp(currentVolume + offset, 0, MAX_VOLUME))
            handled = true
          }
          break
        case 'm':
          if (!hasCurrentTrack) break
          player.onToggleMute()
          handled = true
          break
        case 'n':
        case 'mediatracknext':
          queue.playNext()
          handled = true
          break
        case 'p':
        case 'mediatrackprevious':
          queue.playPrevious()
          handled = true
          break
        case 'l':
          if (!hasCurrentTrack) break
          void player.onToggleLike()
          handled = true
          break
        default:
          break
      }

      if (handled) event.preventDefault()
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])
}
