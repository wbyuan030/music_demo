import { useEffect, useState } from "react"
import type { Track } from "../types/track"
import { dbService } from "../services/dbServices"
import { listen } from "@tauri-apps/api/event"

interface DbChangedEvent {
  payload: string
}
export const useTrackLists = () => {
  const [likedTracks, setLikedTracks] = useState<Track[]>([]);
  const [recentTracks, setRecentTracks] = useState<Track[]>([]);
  useEffect(() => {
    dbService.list_track("like").then(data => setLikedTracks(data));
    dbService.list_track("recent").then(data => setRecentTracks(data));
    let unlistenFn: (() => void) | undefined;
    const setupListener = async () => {
      unlistenFn = await listen('db_tracks_changed', (event: DbChangedEvent) => {
        switch (event.payload) {
          case "like":
            dbService.list_track("like").then(data => setLikedTracks(data));
            break;
          case "recent":
            dbService.list_track("recent").then(data => setRecentTracks(data));
            break;
          default:
            dbService.list_track("like").then(data => setLikedTracks(data));
            dbService.list_track("recent").then(data => setRecentTracks(data));
        }
      })
    };
    setupListener()
    return () => {
      if (unlistenFn) {
        unlistenFn()
      }
    }


  }, [])
  return { likedTracks, recentTracks };
}
