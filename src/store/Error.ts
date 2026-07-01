import { create } from 'zustand'

export interface ErrorItem {
  id: string
  message: string
  timestamp: number
}

interface ErrorState {
  errors: ErrorItem[]
  pushError: (message: string) => void
  dismissError: (id: string) => void
}

let counter = 0

export const useErrorStore = create<ErrorState>((set) => ({
  errors: [],
  pushError: (message: string) => {
    const id = `err_${++counter}`
    set((state) => ({
      errors: [...state.errors, { id, message, timestamp: Date.now() }],
    }))
    // 5 秒后自动移除
    setTimeout(() => {
      set((state) => ({
        errors: state.errors.filter((e) => e.id !== id),
      }))
    }, 5000)
  },
  dismissError: (id: string) => {
    set((state) => ({
      errors: state.errors.filter((e) => e.id !== id),
    }))
  },
}))
