/******************************************************************************
 * useBlockHeight composable —— 区块高度相关工具。
 *
 * 对标 `nrs.util.js:526-540` 的 `NRS.getBlockHeightMoment` /
 * `NRS.getBlockHeightTimeEstimate`：
 *   - getBlockHeightMoment(height)：根据当前链高度与平均出块时间估算目标高度对应的时刻
 *   - getBlockHeightTimeEstimate(height)：将上述时刻格式化为展示字符串
 *
 * 依赖：
 *   - useNodeStore()：提供 `lastBlockHeight`（当前链高度）与
 *     `state.averageBlockGenerationTime`（平均出块时间，秒）
 *
 * 用法：供 `BlockHeightPicker` 等 UI 元素在用户输入目标高度时实时展示预计到达时间。
 ******************************************************************************/
import { computed } from 'vue'
import { useNodeStore } from '@/stores/modules/node.store'

/**
 * 区块高度时间估算 composable。
 *
 * @returns lastBlockHeight    当前链高度（响应式）
 * @returns averageBlockGenerationTime  平均出块时间（秒，响应式）
 * @returns getBlockHeightMoment        估算目标高度对应时刻（Date | null）
 * @returns getBlockHeightTimeEstimate  估算目标高度对应时刻的格式化字符串
 */
export function useBlockHeight() {
  const nodeStore = useNodeStore()

  /** 当前链高度（对标 NRS.lastBlockHeight） */
  const lastBlockHeight = computed<number>(() => nodeStore.lastBlockHeight || 0)

  /** 平均出块时间（秒，对标 NRS.averageBlockGenerationTime） */
  const averageBlockGenerationTime = computed<number>(
    () => nodeStore.state?.averageBlockGenerationTime || 0,
  )

  /**
   * 估算目标高度对应的时刻（对标 nrs.util.js:526 getBlockHeightMoment）。
   *
   * 计算方式：`now + (height - lastBlockHeight) * averageBlockGenerationTime` 秒。
   * 当缺少 lastBlockHeight / averageBlockGenerationTime 或 height 非法时返回 null。
   *
   * @param height 目标区块高度
   * @returns 估算时刻（Date），无法估算时返回 null
   */
  function getBlockHeightMoment(height: number | string): Date | null {
    const targetHeight = typeof height === 'string' ? parseInt(height, 10) : height
    if (!Number.isFinite(targetHeight)) return null
    if (!lastBlockHeight.value || !averageBlockGenerationTime.value) return null

    const heightDiff = targetHeight - lastBlockHeight.value
    // 未来高度 → 正数偏移；过去高度 → 负数偏移（moment.add 支持负值）
    const offsetMs = heightDiff * averageBlockGenerationTime.value * 1000
    return new Date(Date.now() + offsetMs)
  }

  /**
   * 估算目标高度对应的格式化时间字符串（对标 nrs.util.js:534 getBlockHeightTimeEstimate）。
   *
   * 格式：`YYYY/MM/DD hh:mm a`（与参考实现一致）。
   * 无法估算时返回 "-"。
   *
   * @param height 目标区块高度
   * @returns 格式化时间字符串，或 "-"
   */
  function getBlockHeightTimeEstimate(height: number | string): string {
    const moment = getBlockHeightMoment(height)
    if (!moment) return '-'

    // 格式化：YYYY/MM/DD hh:mm a（对标 nrs.util.js:539 moment.format("YYYY/MM/DD hh:mm a")）
    const year = moment.getFullYear()
    const month = String(moment.getMonth() + 1).padStart(2, '0')
    const day = String(moment.getDate()).padStart(2, '0')
    // 12 小时制
    let hours = moment.getHours() % 12
    if (hours === 0) hours = 12
    const minutes = String(moment.getMinutes()).padStart(2, '0')
    const ampm = moment.getHours() >= 12 ? 'PM' : 'AM'
    return `${year}/${month}/${day} ${String(hours).padStart(2, '0')}:${minutes} ${ampm}`
  }

  return {
    lastBlockHeight,
    averageBlockGenerationTime,
    getBlockHeightMoment,
    getBlockHeightTimeEstimate,
  }
}
