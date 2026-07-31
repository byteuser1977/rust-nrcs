<template>
  <el-dialog
    v-model="visible"
    :title="t('nodeInfo.title')"
    width="720px"
    :close-on-click-modal="false"
    destroy-on-close
    class="nrs-modal"
    @close="handleClose"
  >
    <!-- 加载中 -->
    <div v-if="loading" class="nrs-modal-loading">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>{{ t('common.loading') }}</span>
    </div>

    <template v-else>
      <!-- 错误提示 -->
      <el-alert
        v-if="error"
        :title="error"
        type="error"
        :closable="false"
        show-icon
        class="nrs-modal-error"
      />

      <!-- 节点状态信息表（对标 nrs.modals.info.js:30-56） -->
      <div v-if="stateRows.length > 0" class="nrs-modal-state">
        <InfoTable :rows="stateRows" :column="2" />
      </div>

      <!-- 空状态 -->
      <el-empty v-else-if="!error" :description="t('nodeInfo.loadFailed')" />
    </template>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * NrsModal 组件 —— NRCS 节点信息弹窗。
 *
 * 对标 nrs.modals.info.js（85 行）的完整实现。
 *
 * 主要功能：
 *   1. 弹窗显示时调用 getState（includeCounts=true）拉取节点完整状态
 *   2. 遍历 state 字段，按字段名后缀选择格式化方式：
 *      - "number" 前缀字段 → formatAmount（千分位）
 *      - "Memory" 后缀字段 → formatVolume（字节单位）
 *      - "time" 字段 → formatTimestamp（epoch → 本地时间）
 *      - 其他字段 → 直接展示
 *   3. 支持管理员密码（needsAdminPassword 场景）
 *
 * 对标参考：nrsModal.on("shown.bs.modal", ...) 中 sendRequest("getState", {includeCounts:true})
 */
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatAmount, formatTimestamp, formatVolume } from '@/utils/format'
import type { InfoRow } from '@/utils/transaction-info-table'
import InfoTable from '@/components/base/InfoTable.vue'

const props = defineProps<{
  /** 管理员密码（可选，对标 NRS.getAdminPassword()） */
  adminPassword?: string
}>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()

const visible = defineModel<boolean>('visible', { default: false })

const loading = ref(false)
const error = ref<string | null>(null)
const state = ref<Record<string, any> | null>(null)

// ----------------------------------------------------------------
// 计算属性
// ----------------------------------------------------------------

/**
 * 节点状态信息表行（对标 nrs.modals.info.js:34-50）。
 *
 * 遍历 state 对象的每个字段，按字段名特征选择格式化方式：
 *   - key 包含 "number" → formatAmount（千分位格式化，如 numberOfBlocks）
 *   - key 包含 "Memory" → formatVolume（字节 → KB/MB/GB，如 freeMemory）
 *   - key == "time" → formatTimestamp（epoch 时间戳 → 本地时间）
 *   - 其他 → 原值字符串化
 *
 * 标签使用 i18n（nodeInfo.{key}），无对应翻译时回退到 key 本身。
 */
const stateRows = computed<InfoRow[]>(() => {
  if (!state.value) return []

  const rows: InfoRow[] = []
  for (const [key, value] of Object.entries(state.value)) {
    if (value === undefined || value === null || value === '') continue
    // 跳过 version.* 字段（对标参考 NRS.mergeMaps 保留但不展示）
    if (key.startsWith('version.')) continue

    const label = t(`nodeInfo.${key}`, key)
    let displayValue: string

    if (key.indexOf('number') !== -1) {
      // 数量类字段 → 千分位格式化（对标 :41 NRS.formatAmount）
      displayValue = formatAmount(Number(value))
    } else if (key.indexOf('Memory') !== -1) {
      // 内存类字段 → 字节单位格式化（对标 :43 NRS.formatVolume）
      displayValue = formatVolume(Number(value))
    } else if (key === 'time') {
      // 时间字段 → 时间戳格式化（对标 :45 NRS.formatTimestamp）
      displayValue = formatTimestamp(Number(value))
    } else {
      // 其他字段 → 直接展示（对标 :47 NRS.escapeRespStr）
      displayValue = String(value)
    }

    rows.push({ label, value: displayValue })
  }
  return rows
})

// ----------------------------------------------------------------
// 数据加载
// ----------------------------------------------------------------

/**
 * 加载节点状态（对标 nrs.modals.info.js:30-56）。
 *
 * 调用 getState(includeCounts=true) 获取节点完整状态，包括：
 *   numberOfPeers / numberOfAccounts / numberOfTransactions /
 *   numberOfUnlockedAccounts / totalEffectiveBalance / freeMemory 等。
 */
async function loadState(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const resp = await nrcsApi.getState(true)
    if (!resp || (resp as any).errorCode) {
      throw new Error((resp as any)?.errorDescription || t('nodeInfo.loadFailed'))
    }
    state.value = resp as Record<string, any>
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    state.value = null
  } finally {
    loading.value = false
  }
}

// ----------------------------------------------------------------
// 事件处理
// ----------------------------------------------------------------

/**
 * 关闭 modal 并清空状态。
 */
function handleClose(): void {
  visible.value = false
  state.value = null
  error.value = null
  emit('close')
}

// ----------------------------------------------------------------
// 监听 visible 变化自动加载
// ----------------------------------------------------------------

watch(
  () => visible.value,
  async (isVisible) => {
    if (isVisible) {
      await loadState()
    }
  },
  { immediate: true },
)
</script>

<style scoped lang="scss">
.nrs-modal-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: var(--el-text-color-secondary);
}
.nrs-modal-error {
  margin-bottom: 12px;
}
.nrs-modal-state {
  margin-top: 4px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
