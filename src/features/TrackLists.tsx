import TrackLibrary from "../components/TrackLibrary";
import { useTrackLists } from "../hooks/TrackLists";
import { usePlayerStore } from "../store/Player";
import type { Track } from "../types/track";


export default function RecentTrackList() {
  const { likedTracks, recentTracks } = useTrackLists();
  const setCurrentTrack = usePlayerStore((state) => state.setCurrentTrack);
  return (
    <div>
      <TrackLibrary onClick={setCurrentTrack} libraryName="Recent" trackList={recentTracks} />
    </div>
  )
}
