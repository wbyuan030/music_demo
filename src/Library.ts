import { open } from '@tauri-apps/plugin-dialog';
import { readDir, BaseDirectory, readFile } from '@tauri-apps/plugin-fs';
import { audioDir, join } from '@tauri-apps/api/path'; // v2 路径拼接工具
import { usePlayerStore } from './store/Player';
import { parseBuffer } from 'music-metadata-browser';
import type { Track } from './types/track';
import { URL } from 'url';
// import { Blob } from 'buffer';
import { fetch } from "@tauri-apps/plugin-http"
export async function selectFolder() {
  const selected = await open({
    directory: true, // 只选文件夹
    multiple: false, // 单选
  });

  if (selected) {
    console.log("用户选了路径:", selected);
    const musicFiles = await scanMusicFiles(selected);
    console.log("目录下的mp3文件:", musicFiles)
  }
}

async function scanMusicFiles(dirPath: string): Promise<string[]> {
  let musicFiles: string[] = [];
  try {
    const entries = await readDir(dirPath)

    for (const entry of entries) {
      if (entry.isDirectory) {
        try {
          const fullPath = await join(dirPath, entry.name);
          const subFiles = await scanMusicFiles(fullPath)
          musicFiles = [...musicFiles, ...subFiles]
        } catch (err) {
          console.warn(err);
        }

      } else {

        try {
          const fullPath = await join(dirPath, entry.name);
          if (fullPath.endsWith(".mp3")) {
            musicFiles.push(fullPath);
            musicFiles = [...musicFiles, fullPath]
          }
        } catch (err) {
          console.warn(err);
        }
      }
    }
  } catch (err) {
    console.warn(err);
  }
  return musicFiles;
}

async function parseTrackFromFile(filePath: string): Promise<Track> {
  const buffer = await readFile(filePath);
  const metaData = await parseBuffer(buffer, 'audio/mpeg', { duration: true });
  const { common, format } = metaData;
  let coverUrl = "";
  if (common.picture && common.picture.length > 0) {
    const pic = common.picture[0];
    const blob = new Blob([new Uint8Array(pic.data)], { type: pic.format });
    coverUrl = URL.createObjectURL(blob as any);
  }
  // const assetUrl = convertFileSrc(filePath);
  // const assetUrl = `asset://localhost${filePath}`;
  const audioBlob = new Blob([buffer], { type: 'audio/mpeg' });
  const assetUrl = URL.createObjectURL(audioBlob as any);
  return {
    title: common.title || filePath.split("/").pop() || "unknown",
    artist: common.artist || "unknown",
    coverUrl: coverUrl,
    duration: format.duration || 0,
    src: assetUrl
  }

}

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
export async function parseTrackFromUrl(url: string): Promise<Track> {
  const buffer = await fetch(url, {
    method: 'GET', headers: {
      "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36",
      "Referer": url,
      "Accept-Language": "zh-CN,zh;q=0.9"
    }
  }).then(response => response.arrayBuffer())
  const toParsedBuffer = new Uint8Array(buffer);

  const metaData = await parseBuffer(toParsedBuffer)
  const { common, format } = metaData
  let coverUrl = ""
  if (common.picture && common.picture.length > 0) {
    const pic = common.picture[0];
    const blob = new Blob([new Uint8Array(pic.data)], { type: pic.format });
    coverUrl = URL.createObjectURL(blob as any);
  }
  const audioBlob = new Blob([buffer], { type: 'audio/mpeg' });
  const assetUrl = URL.createObjectURL(audioBlob as any);
  return {
    title: common.title || "unknown",
    artist: common.artist || "unknown",
    coverUrl: coverUrl,
    duration: format.duration || 0,
    src: url
  }
}

export async function selectFile() {
  const setCurrentTrack = usePlayerStore.getState().setCurrentTrack
  const selected = await open(
    {
      directory: false,
      multiple: false
    }
  )
  if (selected) {
    console.log("用户选了路径:", selected);
    if (selected.endsWith(".mp3")) {
      const track = await parseTrackFromFile(selected);
      setCurrentTrack(track);
    }
  }
}

export async function getFromWXUrl(url: string) {
  const requestText = await fetch(url, {
    method: 'GET', headers: {

      "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36",
      "Referer": url,
      "Accept-Language": "zh-CN,zh;q=0.9"
    }
  }).then(response => response.text())

  console.log("end fetch")
  console.log(requestText.slice(0, 10))
  const matches = /voice_encode_fileid="([^"]+)"/.exec(requestText)
  if (matches) {
    console.log(matches)
    const audio_url = "https://res.wx.qq.com/voice/getvoice?mediaid=" + matches[1]
    return audio_url
  }
  return null
}
