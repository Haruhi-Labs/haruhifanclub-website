import { ref } from 'vue'

export type SosToastTone = 'default' | 'success' | 'danger'

export interface SosToast {
  id: number
  title?: string
  message: string
  tone: SosToastTone
}

const toasts = ref<SosToast[]>([])
let seq = 0

export interface SosToastOptions {
  title?: string
  tone?: SosToastTone
  duration?: number
}

export function useToast() {
  function dismiss(id: number) {
    toasts.value = toasts.value.filter((toast) => toast.id !== id)
  }

  function push(message: string, options: SosToastOptions = {}) {
    const id = (seq += 1)
    toasts.value = [
      ...toasts.value,
      { id, message, title: options.title, tone: options.tone ?? 'default' },
    ]
    const duration = options.duration ?? 4000
    if (duration > 0 && typeof window !== 'undefined') {
      window.setTimeout(() => dismiss(id), duration)
    }
    return id
  }

  return {
    toasts,
    push,
    dismiss,
    success: (message: string, options: SosToastOptions = {}) =>
      push(message, { ...options, tone: 'success' }),
    danger: (message: string, options: SosToastOptions = {}) =>
      push(message, { ...options, tone: 'danger' }),
  }
}
