import { useEffect, useRef } from "react";
import { useDialogStore } from "./store/Dialog";
import { getFromWXUrl, parseTrackFromUrl, validateWechatUrl } from "./Library";
import { usePlayerStore } from "./store/Player";
import type { Track } from "./types/track";
import { invoke } from "@tauri-apps/api/core";



export function Dialog() {

  const { handleClose, isOpen, onInputValueChange, inputValue } = useDialogStore((state) => state);
  const dialogRef = useRef<HTMLDialogElement>(null);
  const { setCurrentTrack } = usePlayerStore((state) => state)
  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (isOpen) {
      if (!dialog.open) dialog.showModal();
    } else {
      // 关闭
      dialog.close();
    }
  }, [isOpen]);
  if (!isOpen) {
    return null
  }
  const handleConfirm = async () => {
    console.log("用户输入的是:", inputValue);
    let checkMsg = validateWechatUrl(inputValue)
    if (checkMsg != null) {
      console.error(checkMsg)
      handleClose()
      return
    }
    const track = await invoke<Track>("parse_track_from_wx", { url: inputValue });

    if (track) {
      setCurrentTrack(track)
    } else {
      console.error("track is null")
    }

    handleClose();
  };

  return (
    <dialog
      className="p-6 rounded-lg shadow-xl backdrop:bg-black/50 border-none"
      ref={dialogRef}
      onClose={handleClose}
    >
      <h3 className="text-lg font-bold mb-4">输入微信URL</h3>

      <input
        type="text"
        value={inputValue}
        onChange={(e) => onInputValueChange(e.target.value)}
        className="border p-2 rounded w-full mb-4"
        placeholder="写点什么..."
        autoFocus
      />

      <div className="flex justify-end gap-2">
        <button
          onClick={handleClose}
          className="px-4 py-2 text-gray-600 hover:bg-gray-100 focus:ring-2 focus:ring-blue-500 rounded"
        >
          取消
        </button>
        <button
          onClick={handleConfirm}
          className="px-4 py-2 bg-blue-600 text-white rounded focus:ring-2 focus:ring-blue-500 hover:bg-blue-700"
        >
          确定
        </button>
      </div>
    </dialog>
  );
}


