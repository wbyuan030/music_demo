import { invoke } from "@tauri-apps/api/core"
import { useErrorStore } from "../store/Error"

/**
 * 封装 Tauri invoke，失败时自动 push 错误到 Toast
 * 调用方无需 try/catch，返回 null 表示失败
 */
export async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T | null> {
  try {
    return await invoke<T>(command, args)
  } catch (e) {
    useErrorStore.getState().pushError(`${command} 失败: ${e}`)
    return null
  }
}
