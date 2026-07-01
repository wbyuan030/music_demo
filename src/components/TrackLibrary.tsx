import type { Track } from "../types/track";
import { TrackCard } from "./TrackCard";

interface TrackLibraryProps {
  libraryName: string
  trackList: Track[],
  onClick: (track: Track) => Promise<void>
}

export default function TrackLibrary({ libraryName, trackList, onClick }: TrackLibraryProps) {
  return (
    <div className="w-full h-full mx-auto flex flex-col justify-center">
      <h2 className="font-semibold ">{libraryName}</h2 >
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {
          trackList.map((track) => (
            <TrackCard key={track.id} track={track} onClick={() => onClick(track)} />
          ))
        }
        {trackList.length === 0 && (
          <h3>{libraryName} is Empty.</h3>
        )}
      </div>
    </div>
  )
}
