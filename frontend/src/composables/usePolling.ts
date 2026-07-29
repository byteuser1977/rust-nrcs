import { ref, onMounted, onUnmounted } from 'vue'

/**
 * Generic polling composable for periodic data refresh.
 * Starts polling on mount, stops on unmount.
 *
 * @param fn The async function to call on each poll interval
 * @param intervalMs Poll interval in milliseconds (default 30000 = 30s)
 * @param immediate Whether to call fn immediately on start (default true)
 */
export function usePolling(
  fn: () => Promise<void>,
  intervalMs: number = 30000,
  immediate: boolean = true,
) {
  const isPolling = ref(false)
  const pollTimer = ref<ReturnType<typeof setInterval> | null>(null)
  const lastPollTime = ref<number>(0)
  const errorCount = ref(0)

  async function executePoll(): Promise<void> {
    try {
      await fn()
      lastPollTime.value = Date.now()
      errorCount.value = 0
    } catch (err) {
      errorCount.value++
      console.error('[usePolling] Poll error:', err)
    }
  }

  function start(): void {
    if (isPolling.value) return
    isPolling.value = true

    if (immediate) {
      executePoll()
    }

    pollTimer.value = setInterval(() => {
      if (!isPolling.value) return
      executePoll()
    }, intervalMs)
  }

  function stop(): void {
    if (pollTimer.value !== null) {
      clearInterval(pollTimer.value)
      pollTimer.value = null
    }
    isPolling.value = false
  }

  async function refreshNow(): Promise<void> {
    await executePoll()
  }

  onMounted(() => {
    start()
  })

  onUnmounted(() => {
    stop()
  })

  return {
    isPolling,
    lastPollTime,
    errorCount,
    start,
    stop,
    refreshNow,
  }
}
