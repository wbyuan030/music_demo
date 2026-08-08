import { invoke } from "@tauri-apps/api/core"
import { useErrorStore } from "../store/Error"
import { forwardFrontendLog } from "./frontendLog"

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
    const message = `${command} 失败: ${e}`
    useErrorStore.getState().pushError(message)
    forwardFrontendLog({
      level: "error",
      source: "safeInvoke",
      command,
      message,
      stack: e instanceof Error ? e.stack : undefined,
    })
    return null
  }
}
