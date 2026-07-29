import { ref, computed } from 'vue'

/**
 * NRCS-style pagination composable.
 * Uses firstIndex/lastIndex pattern matching the NRCS blockchain API
 * (getAccountTransactions, getBlocks, getAliases, etc.)
 *
 * @param defaultPageSize Number of items per page (default 15)
 */
export function usePagination(defaultPageSize: number = 15) {
  const currentPage = ref(1)
  const pageSize = ref(defaultPageSize)
  const total = ref(0)
  const isLoading = ref(false)

  /** First index for NRCS API (0-based) */
  const firstIndex = computed(() => (currentPage.value - 1) * pageSize.value)

  /** Last index for NRCS API (inclusive) */
  const lastIndex = computed(() => firstIndex.value + pageSize.value - 1)

  /** Total number of pages */
  const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize.value)))

  /** Whether there is a next page */
  const hasNextPage = computed(() => currentPage.value < totalPages.value)

  /** Whether there is a previous page */
  const hasPrevPage = computed(() => currentPage.value > 1)

  /** Go to a specific page */
  function goToPage(page: number): void {
    if (page >= 1 && page <= totalPages.value) {
      currentPage.value = page
    }
  }

  /** Go to next page */
  function nextPage(): void {
    if (hasNextPage.value) {
      currentPage.value++
    }
  }

  /** Go to previous page */
  function prevPage(): void {
    if (hasPrevPage.value) {
      currentPage.value--
    }
  }

  /** Reset to first page */
  function reset(): void {
    currentPage.value = 1
    total.value = 0
  }

  /** Set total from an API response that returns the full list */
  function setTotalFromList(listLength: number): void {
    // When the API returns fewer than pageSize items, we know we're on the last page
    if (listLength < pageSize.value) {
      total.value = firstIndex.value + listLength
    }
    // Otherwise we estimate — the actual total should be set by the caller via setTotal()
  }

  /** Explicitly set total count */
  function setTotal(count: number): void {
    total.value = count
  }

  return {
    // State
    currentPage,
    pageSize,
    total,
    isLoading,
    // Computed
    firstIndex,
    lastIndex,
    totalPages,
    hasNextPage,
    hasPrevPage,
    // Actions
    goToPage,
    nextPage,
    prevPage,
    reset,
    setTotal,
    setTotalFromList,
  }
}
