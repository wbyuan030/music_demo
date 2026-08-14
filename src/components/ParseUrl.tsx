import { useState } from "react";
import type { Track } from "../types/track";
import { useQueueStore } from "../store/Queue";
import { Check, Loader2 } from "lucide-react";
import { useStateStore } from "../store/State";
import { StateEnum } from "../types/state";
import { safeInvoke } from "../services/invoke";

function validateUrl(url: string): string | null {
  const trimmed = url.trim();
  if (!trimmed) {
    return "链接不能为空";
  }

  try {
    const parsed = new URL(trimmed);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
      return "仅支持 http(s) 链接";
    }
  } catch {
    return "链接格式错误，请检查是否完整";
  }

  return null;
}

const handleConfirm = async (
  inputValue: string,
  setIsParsing: (v: boolean) => void,
  playTrack: (track: Track, tracks: Track[]) => void,
  setErrorMessage: (v: string) => void,
  setState: (s: StateEnum) => void,
) => {
  setErrorMessage("")
  const checkMsg = validateUrl(inputValue)
  if (checkMsg != null) {
    console.error(checkMsg)
    setErrorMessage(checkMsg)
    return
  }
  setIsParsing(true)
  const track = await safeInvoke<Track>("parse_track_from_url", { url: inputValue })
  setIsParsing(false)
  if (track) {
    playTrack(track, [track])
    setState(StateEnum.detail)
  } else {
    console.error("track is null")
    setErrorMessage("track is null")
  }
};


export default function ParseUrl() {
  const [inputValue, onInputValueChange] = useState("")
  const [isParsing, setIsParsing] = useState(false)
  const [errorMessage, setErrorMessage] = useState("")
  const playTrack = useQueueStore((state) => state.playFromList)
  const setState = useStateStore((state) => state.setCurrentState)
  return (
    <div className="flex flex-row items-center gap-2 w-full max-w-2xl mx-auto">
      <input
        type="text"
        value={inputValue}
        onChange={(e) => onInputValueChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") handleConfirm(inputValue, setIsParsing, playTrack, setErrorMessage, setState)
        }}
        className="h-11 flex-1 px-4 rounded-full bg-neutral-900 border border-neutral-800 text-gray-200 placeholder:text-neutral-500 shadow-inner transition-all focus:outline-none focus:ring-2 focus:ring-emerald-500/50 focus:border-emerald-500/60"
        placeholder="粘贴音乐链接..."
        autoFocus
      />

      <button
        onClick={() => { handleConfirm(inputValue, setIsParsing, playTrack, setErrorMessage, setState) }}
        className="flex size-11 shrink-0 items-center justify-center rounded-full bg-emerald-500 text-neutral-950 shadow-lg shadow-emerald-500/20 touch-manipulation transition-all hover:bg-emerald-400 active:scale-90 disabled:opacity-50 disabled:pointer-events-none"
        disabled={isParsing}
      >
        {isParsing
          ? <Loader2 className="size-4 animate-spin" />
          : <Check className="size-4" />}
      </button>
      {errorMessage.length > 0 && (
        <span
          className="text-xs text-red-400 cursor-pointer select-none shrink-0"
          onClick={() => setErrorMessage("")}
        >
          {errorMessage}
        </span>
      )}
    </div>
  )
}
