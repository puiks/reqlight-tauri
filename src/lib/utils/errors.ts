import { toastStore } from '../stores/toast.svelte'

/**
 * Centralized error handler.
 * Logs to console and optionally shows a toast to the user.
 */
export function handleError(error: unknown, context: string, options?: { silent?: boolean }) {
  const message = error instanceof Error ? error.message : String(error)
  console.error(`[${context}]`, message)

  if (!options?.silent) {
    toastStore.show(`Error: ${message}`)
  }
}

/**
 * Wraps an async function with error handling.
 * Useful for event handlers that can't propagate errors.
 */
export function withErrorHandling<T extends (...args: unknown[]) => Promise<unknown>>(
  fn: T,
  context: string,
): T {
  return (async (...args: unknown[]) => {
    try {
      return await fn(...args)
    } catch (e) {
      handleError(e, context)
    }
  }) as T
}
