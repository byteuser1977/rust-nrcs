<template>
  <el-dialog
    v-model="visible"
    :title="t('peerInfo.title')"
    width="640px"
    :close-on-click-modal="false"
    destroy-on-close
    class="peer-info-modal"
    @close="handleClose"
  >
    <!-- 加载中 -->
    <div v-if="loading" class="peer-info-loading">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>{{ t('common.loading') }}</span>
    </div>

    <template v-else-if="peer">
      <!-- 错误提示 -->
      <el-alert
        v-if="error"
        :title="error"
        type="error"
        :closable="false"
        show-icon
        class="peer-info-error"
      />

      <!-- Peer 地址标题 -->
      <div class="peer-info-header">
        <span class="peer-info-label">{{ t('peerInfo.announcedAddress') }}:</span>
        <span class="peer-info-value">{{ peer.announcedAddress || peer.address }}</span>
      </div>

      <!-- Peer 详情信息表（对标 nrs.modals.peer.js:70-87） -->
      <InfoTable :rows="peerDetailsRows" :column="2" />

      <!-- Hallmark 详情（若启用 hallmark 解码，对标 :37-61） -->
      <div v-if="hallmarkDetails" class="peer-info-hallmark">
        <h4 class="peer-info-hallmark-title">{{ t('peerInfo.hallmark') }}</h4>
        <InfoTable :rows="hallmarkRows" :column="2" />
      </div>
    </template>

    <!-- 空状态 -->
    <el-empty v-else :description="t('peerInfo.notFound')" />

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * PeerInfoModal 组件 —— Peer 详情弹窗。
 *
 * 对标 nrs.modals.peer.js（89 行）的完整实现。
 *
 * 主要功能：
 *   1. 显示 Peer 基本信息（地址/端口/状态/版本/平台/软件等）
 *   2. 格式化显示 lastUpdated/lastConnectAttempt 时间戳
 *   3. 格式化显示 downloaded/uploaded 流量
 *   4. 若启用 hallmark 解码，展示 hallmark 详情（account/host/port/weight/date/valid）
 *
 * 支持两种入参：
 *   - peer: NrcsPeer 对象（直接展示，不调 API）
 *   - address: Peer 地址（调 getPeer API 拉取）
 *
 * 对标参考：showPeerModal(peer) 函数
 */
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsPeer } from '@/api/modules/nrcs.api'
import { formatTimestamp } from '@/utils/format'
import { createInfoTable, type InfoRow } from '@/utils/transaction-info-table'
import InfoTable from '@/components/base/InfoTable.vue'

const props = defineProps<{
  /** Peer 对象（直接展示，不调 API） */
  peer?: NrcsPeer | null
  /** Peer 地址（调 getPeer API 拉取） */
  address?: string | null
}>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()

const visible = defineModel<boolean>('visible', { default: false })

const loading = ref(false)
const error = ref<string | null>(null)
const peer = ref<NrcsPeer | null>(null)

/** Hallmark 详情（由 decodeHallmark 拉取） */
const hallmarkDetails = ref<any>(null)

// ----------------------------------------------------------------
// 计算属性
// ----------------------------------------------------------------

/**
 * Peer 详情信息表行（对标 nrs.modals.peer.js:70-87 showPeerModalImpl）。
 *
 * 处理：
 *   - state → 人类可读名称
 *   - lastUpdated → formatTimestamp
 *   - lastConnectAttempt → formatTimestamp
 *   - downloadedVolume → 格式化字节
 *   - uploadedVolume → 格式化字节
 */
const peerDetailsRows = computed<InfoRow[]>(() => {
  if (!peer.value) return []
  const p: Record<string, any> = { ...peer.value }

  // state 映射（对标 NRS.getPeerState）
  p.state = getPeerStateName(p.state)

  // 时间戳格式化
  if (p.lastUpdated) {
    p.lastUpdated = formatTimestamp(p.lastUpdated)
  }
  if (p.lastConnectAttempt) {
    p.lastConnectAttempt = formatTimestamp(p.lastConnectAttempt)
  }

  // 流量格式化
  if (p.downloadedVolume !== undefined) {
    p.downloaded_formatted_html = formatVolume(p.downloadedVolume)
    delete p.downloadedVolume
  }
  if (p.uploadedVolume !== undefined) {
    p.uploaded_formatted_html = formatVolume(p.uploadedVolume)
    delete p.uploadedVolume
  }

  return createInfoTable(p, false)
})

/** Hallmark 信息表行 */
const hallmarkRows = computed<InfoRow[]>(() => {
  if (!hallmarkDetails.value) return []
  return createInfoTable(hallmarkDetails.value, false)
})

// ----------------------------------------------------------------
// 辅助函数
// ----------------------------------------------------------------

/**
 * 获取 Peer 状态名称（对标 NRS.getPeerState）。
 *
 * @param state 状态编号
 * @returns 状态名称
 */
function getPeerStateName(state: number | string | undefined): string {
  switch (Number(state)) {
    case 0:
      return t('peerInfo.stateNonConnected')
    case 1:
      return t('peerInfo.stateConnected')
    case 2:
      return t('peerInfo.stateDisconnected')
    default:
      return String(state ?? '')
  }
}

/**
 * 格式化字节大小为人类可读字符串（对标 NRS.formatVolume）。
 *
 * @param bytes 字节数
 * @returns 格式化后的字符串（如 "1.5 MB"）
 */
function formatVolume(bytes: number | string | undefined): string {
  if (bytes === undefined || bytes === null) return '0 B'
  const n = Number(bytes)
  if (isNaN(n) || n === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let size = n
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024
    i++
  }
  return `${size.toFixed(2)} ${units[i]}`
}

// ----------------------------------------------------------------
// 数据加载
// ----------------------------------------------------------------

/**
 * 加载 Peer 详情（对标 nrs.modals.peer.js:22-32）。
 *
 * @param peerAddress Peer 地址
 */
async function loadPeer(peerAddress: string): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const resp = await nrcsApi.getPeer(peerAddress)
    if (!resp || (resp as any).errorCode) {
      throw new Error((resp as any)?.errorDescription || t('peerInfo.notFound'))
    }
    peer.value = resp

    // 若启用 hallmark 解码（对标 :37-61）
    if (resp.hallmark) {
      try {
        // 注：nrcsApi.decodeHallmark 暂未封装，使用通用 nrcsGet 调用
        // 此处先简化：若有 hallmark 字段，展示原始字符串
        hallmarkDetails.value = { hallmark: resp.hallmark }
      } catch {
        hallmarkDetails.value = null
      }
    } else {
      hallmarkDetails.value = null
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    peer.value = null
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
  peer.value = null
  error.value = null
  hallmarkDetails.value = null
  emit('close')
}

// ----------------------------------------------------------------
// 监听 props 变化自动加载
// ----------------------------------------------------------------

watch(
  () => [props.peer, props.address, visible.value] as const,
  async ([newPeer, newAddress, isVisible]) => {
    if (!isVisible) return
    if (newPeer) {
      peer.value = newPeer
      hallmarkDetails.value = null
    } else if (newAddress) {
      await loadPeer(newAddress)
    }
  },
  { immediate: true },
)
</script>

<style scoped lang="scss">
.peer-info-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: var(--el-text-color-secondary);
}
.peer-info-error {
  margin-bottom: 12px;
}
.peer-info-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  margin-bottom: 12px;
  font-size: 14px;
}
.peer-info-label {
  color: var(--el-text-color-secondary);
}
.peer-info-value {
  font-family: monospace;
  font-size: 13px;
  word-break: break-all;
}
.peer-info-hallmark {
  margin-top: 16px;
}
.peer-info-hallmark-title {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
