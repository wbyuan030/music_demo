import { safeInvoke } from "./invoke";
import type { Track } from "../types/track";

interface DbService {
  get_track: (id: string) => Promise<Track | null>
  delete_track: (id: string) => Promise<void>
  list_track: (sheet_name: string) => Promise<Track[]>
}

export const dbService: DbService = {
  get_track: async (id: string) => {
    return await safeInvoke<Track>("get_track", { id })
  },
  delete_track: async (_id: String) => { },
  list_track: async (sheet_name: string) => {
    switch (sheet_name) {
      case "like":
        return (await safeInvoke<Track[]>("list_like_tracks")) ?? []
      case "recent":
        return (await safeInvoke<Track[]>("list_recent_tracks")) ?? []
      default:
        console.error("not implemented", sheet_name)
        return []
    }
  },
}
