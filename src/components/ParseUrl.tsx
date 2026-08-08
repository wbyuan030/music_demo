import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import type { Track } from "../types/track";
import { usePlayerStore } from "../store/Player";
import { Check, Loader2 } from "lucide-react";
import { useStateStore } from "../store/State";
import { StateEnum } from "../types/state";

export function validateWechatUrl(url: string): string | null {
  const trimmed = url.trim();

  if (!trimmed) {
    return "链接不能为空";
  }

  // 简单的 URL 格式检查
  try {
    const urlObj = new URL(trimmed);
    // 检查域名 (兼容 mp.weixin.qq.com)
    if (urlObj.hostname !== "mp.weixin.qq.com") {
      return "这不是一个有效的微信公众号文章链接";
    }
  } catch (e) {
    return "链接格式错误，请检查是否完整";
  }

  return null; // 返回 null 代表校验通过
}
const handleConfirm = async (inputValue: string, setIsParsing: Function, setTrack: Function, setErrorMessage: Function, setState: Function) => {
  setErrorMessage("")
  let checkMsg = validateWechatUrl(inputValue)
  //TODO:错误处理
  if (checkMsg != null) {
    console.error(checkMsg)
    setErrorMessage(checkMsg)
    return
  }
  setIsParsing(true)
  const track = await invoke<Track>("parse_track_from_wx", { url: inputValue });
  setIsParsing(false)
  if (track) {
    setTrack(track)
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
  const setTrack = usePlayerStore((state) => state.setCurrentTrack)
  const setState = useStateStore((state) => state.setCurrentState)
  return (
    <div className="flex flex-row items-center gap-2 w-full max-w-2xl mx-auto">
      <input
        type="text"
        value={inputValue}
        onChange={(e) => onInputValueChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") handleConfirm(inputValue, setIsParsing, setTrack, setErrorMessage, setState)
        }}
        className="h-10 flex-1 px-4 rounded-full bg-neutral-900 border border-neutral-800 text-gray-200 placeholder:text-neutral-500 shadow-inner transition-all focus:outline-none focus:ring-2 focus:ring-emerald-500/50 focus:border-emerald-500/60"
        placeholder="粘贴微信公众号文章链接..."
        autoFocus
      />

      <button
        onClick={() => { handleConfirm(inputValue, setIsParsing, setTrack, setErrorMessage, setState) }}
        className="w-10 h-10 shrink-0 rounded-full bg-emerald-500 hover:bg-emerald-400 active:scale-90 text-neutral-950 shadow-lg shadow-emerald-500/20 flex items-center justify-center transition-all disabled:opacity-50 disabled:pointer-events-none"
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
