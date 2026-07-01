import { useEffect } from "react"
import { useLikedStore, useRecentStore } from "../store/Db"
import { usePlayerStore } from "../store/Player"
import { TrackCard } from "./TrackCard.tsx"




export default function MainPageContent() {
  const recentTracks = useRecentStore((state) => state.recentTracks);
  const getRecentTracks = useRecentStore((state) => state.getRecentTracks);
  const setTrack = usePlayerStore((state) => state.setCurrentTrack);
  const getLikedTracks = useLikedStore((state) => state.getLikedTracks);
  const likedTracks = useLikedStore((state) => state.likedTracks);

  useEffect(() => {
    getRecentTracks();
    getLikedTracks();
  }, []);

  return (
    <div className="w-full bg-neutral-900 max-w-5xl mx-auto p-8 pb-32 space-y-10">

      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
            <span className="w-1 h-6 bg-purple-500 rounded-full inline-block"></span>
            Recent Tracks
          </h2>
        </div>

        {/* Grid 布局：手机1列，平板2列，桌面3列 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {recentTracks.map((track) => (
            <TrackCard key={track.id} track={track} onClick={setTrack} />
          ))}
          {recentTracks.length === 0 && (
            <div className="text-gray-500 text-sm col-span-full py-8 text-center italic">
              No recent tracks found.
            </div>
          )}
        </div>
      </section>

      {/* Liked Tracks Section */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
            <span className="w-1 h-6 bg-pink-500 rounded-full inline-block"></span>
            Liked Tracks
          </h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {likedTracks.map((track) => (
            <TrackCard key={track.id} track={track} onClick={setTrack} />
          ))}
          {likedTracks.length === 0 && (
            <div className="text-gray-500 text-sm col-span-full py-8 text-center italic">
              Go like some music!
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
