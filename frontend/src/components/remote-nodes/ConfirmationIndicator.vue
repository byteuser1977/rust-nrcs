<script setup lang="ts">
/**
 * ConfirmationIndicator.vue —— 远程节点响应确认率指示器。
 *
 * 对标参考实现 `nrs.remote.nodes.js:275` 的 `NRS.updateConfirmationsIndicator` 与
 * `nrs.remote.nodes.js:321` 的 `NRS.updateConfirmationsTable`。
 *
 * 在 API 代理或移动端模拟场景下，远程节点不可信，需对可转发的 GET 请求做
 * 多节点交叉验证。本组件实时展示确认率（绿/黄/红渐变徽章）与确认历史表格。
 *
 * 数据来源：`@/utils/remote-nodes` 暴露的响应式 `confirmationRate` 与 `confirmations`。
 * 仅在远程节点管理器已启用（isMobileApp 或 apiProxy）时渲染。
 */
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Lightning, Warning } from '@element-plus/icons-vue'
import { ElDrawer, ElTable, ElTableColumn, ElEmpty, ElTooltip } from 'element-plus'
import { confirmationRate, confirmations, type ConfirmationReport } from '@/utils/remote-nodes'
import { isMobileApp, getFeatureContext } from '@/utils/feature-detection'
import { formatTimestamp } from '@/utils/format'

const { t } = useI18n()

/** 历史抽屉可见性 */
const drawerVisible = ref(false)

/** 是否启用远程节点确认（移动端或 API 代理） */
const enabled = computed(() => isMobileApp() || !!getFeatureContext().apiProxy)

/** 徽章图标（有拒绝时用 Warning，否则用 Lightning） */
const icon = computed(() => (confirmationRate.value.hasRejections ? Warning : Lightning))

/** 徽章提示文案 */
const tooltipContent = computed(() => {
  const { confirmations: c, rejections: r } = confirmationRate.value
  return `${t('remoteNodes.confirmationRate')}: ${c - r}/${c}`
})

/** 打开历史抽屉 */
function openDrawer(): void {
  drawerVisible.value = true
}

/**
 * 渲染节点地址列表为字符串（对标 `nrs.remote.nodes.js:313` 的 printRemoteAddresses）。
 */
function formatNodes(nodes: { announcedAddress?: string; address: string }[]): string {
  if (!nodes || nodes.length === 0) return '-'
  return nodes.map((n) => n.announcedAddress || n.address).join(', ')
}
</script>

<template>
  <div v-if="enabled" class="confirmation-indicator">
    <el-tooltip :content="tooltipContent" placement="bottom">
      <span
        class="indicator-badge"
        :style="{ backgroundColor: confirmationRate.color }"
        @click="openDrawer"
      >
        <el-icon class="indicator-icon"><component :is="icon" /></el-icon>
        <span class="indicator-text">{{ confirmationRate.confirmations - confirmationRate.rejections }}/{{ confirmationRate.confirmations }}</span>
      </span>
    </el-tooltip>

    <!-- 确认历史抽屉（对标 #request_confirmations_info_table） -->
    <el-drawer
      v-model="drawerVisible"
      :title="t('remoteNodes.requestConfirmations')"
      direction="rtl"
      size="720px"
      destroy-on-close
    >
      <el-table
        v-if="confirmations.length > 0"
        :data="confirmations as ConfirmationReport[]"
        style="width: 100%"
        size="small"
        border
      >
        <el-table-column :label="t('remoteNodes.requestTime')" min-width="160">
          <template #default="{ row }">
            <div class="mono-text">{{ formatTimestamp(row.requestTime) }}</div>
            <div class="request-type">{{ row.requestType }}</div>
          </template>
        </el-table-column>
        <el-table-column :label="t('remoteNodes.confirmingNodes')" min-width="200">
          <template #default="{ row }">
            <div class="mono-text">{{ formatNodes(row.confirmingNodes) }}</div>
          </template>
        </el-table-column>
        <el-table-column :label="t('remoteNodes.rejectingNodes')" min-width="200">
          <template #default="{ row }">
            <div v-if="row.rejectingNodes.length > 0" class="mono-text rejecting">
              {{ formatNodes(row.rejectingNodes) }}
            </div>
            <span v-else>-</span>
          </template>
        </el-table-column>
      </el-table>
      <el-empty v-else :description="t('remoteNodes.noConfirmations')" />
    </el-drawer>
  </div>
</template>

<style scoped>
.confirmation-indicator {
  display: inline-flex;
  align-items: center;
}

.indicator-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 12px;
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  user-select: none;
  transition: opacity 0.2s;
  background-color: #3ea940;
}

.indicator-badge:hover {
  opacity: 0.85;
}

.indicator-icon {
  font-size: 14px;
}

.indicator-text {
  font-variant-numeric: tabular-nums;
}

.mono-text {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 12px;
  word-break: break-all;
}

.request-type {
  color: var(--el-text-color-secondary);
  font-size: 11px;
  margin-top: 2px;
}

.rejecting {
  color: var(--el-color-danger);
}
</style>
