import { useErrorStore } from "../store/Error"
import { X } from "lucide-react"

export default function Toast() {
  const errors = useErrorStore((state) => state.errors)
  const dismissError = useErrorStore((state) => state.dismissError)

  if (errors.length === 0) return null

  return (
    <div className="fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
      {errors.map((err) => (
        <div
          key={err.id}
          className="flex items-start gap-2 bg-red-900/90 text-red-100 px-4 py-3 rounded-lg shadow-lg border border-red-700/50 backdrop-blur-sm animate-in slide-in-from-right"
        >
          <span className="flex-1 text-sm leading-snug">{err.message}</span>
          <button
            onClick={() => dismissError(err.id)}
            className="shrink-0 text-red-300 hover:text-red-100 transition-colors"
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  )
}
