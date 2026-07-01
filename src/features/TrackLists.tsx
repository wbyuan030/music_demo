import TrackLibrary from "../components/TrackLibrary";
import { useTrackLists } from "../hooks/TrackLists";
import { usePlayerStore } from "../store/Player";

export default function RecentTrackList() {
  const { recentTracks } = useTrackLists();
  const setCurrentTrack = usePlayerStore((state) => state.setCurrentTrack);
  return (
    <div>
      <TrackLibrary onClick={async (track) => { setCurrentTrack(track); }} libraryName="Recent" trackList={recentTracks} />
    </div>
  )
}
