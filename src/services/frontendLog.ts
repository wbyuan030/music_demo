import { invoke } from "@tauri-apps/api/core"

export type FrontendLogLevel = "error" | "warn" | "info" | "debug" | "log"

export interface FrontendLogPayload {
  level: FrontendLogLevel
  source: string
  message: string
  stack?: string
  command?: string
}

type ForwardableConsoleMethod = Exclude<FrontendLogLevel, "log"> | "log"

const MAX_FIELD_LENGTH = 4_000
let forwardingInstalled = false

function truncate(value: string): string {
  return value.length > MAX_FIELD_LENGTH
    ? `${value.slice(0, MAX_FIELD_LENGTH)}…`
    : value
}

function formatValue(value: unknown): string {
  if (value instanceof Error) {
    return value.stack ?? value.message
  }
  if (typeof value === "string") return value

  try {
    const serialized = JSON.stringify(value)
    return serialized === undefined ? String(value) : serialized
  } catch {
    return String(value)
  }
}

export function forwardFrontendLog(payload: FrontendLogPayload): void {
  const entry = {
    ...payload,
    message: truncate(payload.message),
    stack: payload.stack ? truncate(payload.stack) : undefined,
    command: payload.command ? truncate(payload.command) : undefined,
  }

  // This is intentionally not safeInvoke: reporting a log failure must never
  // create another frontend error report.
  void invoke("report_frontend_log", entry).catch(() => undefined)
}

export function installFrontendLogForwarding(): void {
  if (forwardingInstalled) return
  forwardingInstalled = true

  const methods: ForwardableConsoleMethod[] = ["error", "warn", "info", "debug", "log"]
  const consoleObject = console as unknown as Record<ForwardableConsoleMethod, (...args: unknown[]) => void>

  for (const level of methods) {
    const original = consoleObject[level].bind(console)
    consoleObject[level] = (...args: unknown[]) => {
      original(...args)
      forwardFrontendLog({
        level,
        source: "console",
        message: args.map(formatValue).join(" "),
      })
    }
  }

  window.addEventListener("error", (event) => {
    forwardFrontendLog({
      level: "error",
      source: "window.error",
      message: event.message || "uncaught frontend error",
      stack: event.error instanceof Error ? event.error.stack : undefined,
    })
  })

  window.addEventListener("unhandledrejection", (event) => {
    forwardFrontendLog({
      level: "error",
      source: "window.unhandledrejection",
      message: formatValue(event.reason),
      stack: event.reason instanceof Error ? event.reason.stack : undefined,
    })
  })
}
