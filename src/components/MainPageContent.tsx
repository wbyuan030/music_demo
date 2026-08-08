import { useEffect } from "react"
import { useLikedStore, useRecentStore } from "../store/Db"
import { usePlayerStore } from "../store/Player"
import { TrackCard } from "./TrackCard.tsx"
import { History, Heart } from "lucide-react"

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

  const SectionTitle = ({ icon, label }: { icon: React.ReactNode; label: string }) => (
    <h2 className="text-lg font-bold text-white tracking-tight flex items-center gap-2.5">
      <span className="w-1 h-5 bg-emerald-400 rounded-full inline-block"></span>
      <span className="text-neutral-400">{icon}</span>
      {label}
    </h2>
  );

  const EmptyState = ({ text }: { text: string }) => (
    <div className="col-span-full py-10 text-center text-sm text-neutral-500 border border-dashed border-neutral-800 rounded-xl bg-neutral-900/30">
      {text}
    </div>
  );

  return (
    <div className="w-full max-w-5xl mx-auto p-6 pb-36 space-y-10">

      <section className="space-y-4">
        <SectionTitle icon={<History size={18} />} label="最近播放" />
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {recentTracks.map((track) => (
            <TrackCard key={track.id} track={track} onClick={setTrack} />
          ))}
          {recentTracks.length === 0 && <EmptyState text="还没有播放记录，去搜索一首歌吧" />}
        </div>
      </section>

      <section className="space-y-4">
        <SectionTitle icon={<Heart size={18} />} label="我喜欢的音乐" />
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {likedTracks.map((track) => (
            <TrackCard key={track.id} track={track} onClick={setTrack} />
          ))}
          {likedTracks.length === 0 && <EmptyState text="还没有喜欢的歌曲" />}
        </div>
      </section>
    </div>
  );
}
