import { onUnmounted } from 'vue'

interface DebounceOptions {
  delay?: number
  leading?: boolean
  trailing?: boolean
}

/**
 * A highly configurable debounce/throttle composable.
 * 
 * @param fn The function to execute
 * @param options Configuration for delay, leading, and trailing edges
 * @returns A debounced version of the provided function
 */
export function useDebounce<T extends (...args: any[]) => any>(
  fn: T,
  options: DebounceOptions = {}
) {
  const { 
    delay = 300, 
    leading = true, 
    trailing = false 
  } = options

  let timeout: ReturnType<typeof setTimeout> | null = null
  let lastCall = 0

  const debouncedFn = (...args: Parameters<T>): void => {
    const now = Date.now()
    const isWithinDelay = now - lastCall < delay

    if (leading && !isWithinDelay) {
      lastCall = now
      fn(...args)
      return
    }

    if (trailing) {
      if (timeout) clearTimeout(timeout)
      timeout = setTimeout(() => {
        lastCall = Date.now()
        fn(...args)
      }, delay)
    }
  }

  // Cleanup on component unmount
  onUnmounted(() => {
    if (timeout) {
      clearTimeout(timeout)
    }
  })

  return debouncedFn
}
