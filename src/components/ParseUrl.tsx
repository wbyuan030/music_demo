import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import type { Track } from "../types/track";
import { usePlayerStore } from "../store/Player";
import { Check } from "lucide-react";
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
    <div
      className="p-6 flex flex-row w-full h-full rounded-lg gap-2 shadow-lg backdrop:bg-black/50 border-none"
    >

      <input
        type="text"
        value={inputValue}
        onChange={(e) => onInputValueChange(e.target.value)}
        className="border p-2 rounded flex-1"
        placeholder="输入Url..."
        autoFocus
      />

      <button
        onClick={() => { handleConfirm(inputValue, setIsParsing, setTrack, setErrorMessage, setState) }}
        className="group px-4 py-2 bg-white! rounded  hover:text-blue-500 disabled:hidden transition-all duration-200"
        disabled={isParsing}
      >
        <Check className="bg-white group-hover:scale-125 transition-transform" />
      </button>
      <span hidden={errorMessage.length == 0} className="items-center justify-center text-red-600 flex flex-1" onClick={() => setErrorMessage("")}>
        {errorMessage}

      </span>


    </div>
  )
}
