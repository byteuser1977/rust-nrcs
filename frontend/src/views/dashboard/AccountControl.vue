<template>
  <!--
    AccountControl —— 账户控制（Phasing Only Control）页面。
    对标 nrs.accountcontrol.js 的 NRS.pages.account_control。

    功能：
      1. 展示当前账户的强制审批策略状态（phasingOnlyControl）
      2. 展示策略详情：投票模型/法定人数/白名单/最小余额/持续时间/最大手续费
      3. 设置/修改策略按钮 → SetPhasingOnlyControlModal
      4. 移除策略按钮（需当前策略审批通过才能移除）
  -->
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Lock /></el-icon>
        {{ t('accountControl.title') }}
      </h2>
      <div class="header-actions">
        <el-button
          type="primary"
          size="small"
          @click="showSetModal = true"
        >
          <el-icon><Setting /></el-icon>
          {{ phasingOnlyControl ? t('accountControl.modifyControl') : t('accountControl.setControl') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card
      shadow="hover"
      class="control-card"
      v-loading="loading"
    >
      <!-- 无控制策略 -->
      <div v-if="!phasingOnlyControl" class="no-control">
        <el-icon class="no-control-icon"><Unlock /></el-icon>
        <p class="no-control-text">{{ t('accountControl.noControlSet') }}</p>
        <p class="no-control-hint">{{ t('accountControl.noControlHint') }}</p>
      </div>

      <!-- 有控制策略 -->
      <div v-else class="control-info">
        <el-descriptions :column="2" border>
          <el-descriptions-item :label="t('accountControl.votingModel')">
            <el-tag :type="votingModelTagType" size="small">
              {{ votingModelLabel }}
            </el-tag>
          </el-descriptions-item>

          <el-descriptions-item :label="t('accountControl.quorum')">
            <span class="text-mono">{{ quorumDisplay }}</span>
          </el-descriptions-item>

          <el-descriptions-item
            v-if="phasingOnlyControl.minBalanceModel && phasingOnlyControl.minBalanceModel > 0"
            :label="t('accountControl.minBalanceType')"
          >
            {{ minBalanceModelLabel }}
          </el-descriptions-item>

          <el-descriptions-item
            v-if="phasingOnlyControl.minBalance"
            :label="t('accountControl.minBalance')"
          >
            <span class="text-mono">{{ minBalanceDisplay }}</span>
          </el-descriptions-item>

          <el-descriptions-item
            v-if="phasingOnlyControl.holding !== undefined && phasingOnlyControl.holding !== 0"
            :label="t('accountControl.holding')"
          >
            <span class="text-mono">{{ phasingOnlyControl.holdingId || '-' }}</span>
          </el-descriptions-item>

          <el-descriptions-item :label="t('accountControl.minDuration')">
            {{ phasingOnlyControl.minDuration || 0 }} {{ t('accountControl.blocks') }}
          </el-descriptions-item>

          <el-descriptions-item :label="t('accountControl.maxDuration')">
            {{ phasingOnlyControl.maxDuration || 0 }} {{ t('accountControl.blocks') }}
          </el-descriptions-item>

          <el-descriptions-item :label="t('accountControl.maxPendingFees')">
            <span class="text-mono">{{ maxFeesDisplay }}</span>
          </el-descriptions-item>

          <el-descriptions-item :label="t('common.height')">
            {{ phasingOnlyControl.height || '-' }}
          </el-descriptions-item>
        </el-descriptions>

        <!-- 白名单账户 -->
        <div v-if="phasingOnlyControl.phasingWhitelisted && phasingOnlyControl.phasingWhitelisted.length > 0" class="whitelist-section">
          <h4 class="section-title">
            <el-icon><User /></el-icon>
            {{ t('accountControl.whitelist') }}
            <span class="count-badge">({{ phasingOnlyControl.phasingWhitelisted.length }})</span>
          </h4>
          <div class="whitelist-list">
            <el-tag
              v-for="acct in phasingOnlyControl.phasingWhitelisted"
              :key="acct"
              size="small"
              class="whitelist-tag"
            >
              {{ acct }}
            </el-tag>
          </div>
        </div>

        <!-- 警告提示 -->
        <el-alert
          type="info"
          :closable="false"
          show-icon
          class="control-notice"
        >
          {{ t('accountControl.controlNotice') }}
        </el-alert>
      </div>
    </el-card>

    <SetPhasingOnlyControlModal
      v-model="showSetModal"
      @success="refreshData"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * AccountControl 组件 —— 账户控制（Phasing Only Control）页面。
 *
 * 对标 nrs.accountcontrol.js 的 NRS.pages.account_control：
 *   - 展示当前账户的 phasingOnlyControl 状态（由 account.store 管理）
 *   - 展示策略详情：投票模型/法定人数/白名单/最小余额/持续时间/最大手续费
 *   - 设置/修改策略入口 → SetPhasingOnlyControlModal
 *
 * phasingOnlyControl 数据来源：account.store.updateAccountControlStatus
 * 在账户信息轮询时自动拉取 getPhasingOnlyControl API。
 */
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Lock, Unlock, Setting, Refresh, User } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { nrcsApi } from '@/api/modules'
import SetPhasingOnlyControlModal from '@/components/modals/SetPhasingOnlyControlModal.vue'
import { nqtToNxt } from '@/utils/format'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)
const showSetModal = ref(false)

/** 当前账户的 phasingOnlyControl（从 store 读取） */
const phasingOnlyControl = computed(() => accountStore.phasingOnlyControl)

/** 投票模型显示标签 */
const votingModelLabel = computed(() => {
  const model = phasingOnlyControl.value?.votingModel
  if (model === undefined || model === null) return '-'
  const labels: Record<number, string> = {
    [-1]: t('accountControl.noApproval'),
    [0]: t('accountControl.byAccount'),
    [1]: t('accountControl.byBalance'),
    [2]: t('accountControl.byAsset'),
    [3]: t('accountControl.byCurrency'),
  }
  return labels[model] || t('common.unknown')
})

/** 投票模型标签颜色 */
const votingModelTagType = computed<'info' | 'success' | 'warning' | 'danger'>(() => {
  const model = phasingOnlyControl.value?.votingModel
  if (model === -1 || model === undefined) return 'info'
  return 'warning'
})

/** 法定人数显示 */
const quorumDisplay = computed(() => {
  const control = phasingOnlyControl.value
  if (!control) return '-'
  if (control.quorum !== undefined) {
    return control.quorum
  }
  return '-'
})

/** 最小余额模型标签 */
const minBalanceModelLabel = computed(() => {
  const model = phasingOnlyControl.value?.minBalanceModel
  if (!model) return '-'
  const labels: Record<number, string> = {
    [0]: t('accountControl.mbNone'),
    [1]: t('accountControl.mbBalance'),
    [2]: t('accountControl.mbAsset'),
    [3]: t('accountControl.mbCurrency'),
  }
  return labels[model] || t('common.unknown')
})

/** 最小余额显示（根据模型转换单位） */
const minBalanceDisplay = computed(() => {
  const control = phasingOnlyControl.value
  if (!control || !control.minBalance) return '-'
  const model = control.minBalanceModel
  // 模型 1（余额）是 NQT，需要转 NXT
  if (model === 1) {
    return `${nqtToNxt(control.minBalance)} NRC`
  }
  // 模型 2/3 是 QNT，直接显示
  return control.minBalance
})

/** 最大手续费显示（NQT → NXT） */
const maxFeesDisplay = computed(() => {
  const fees = phasingOnlyControl.value?.maxFees
  if (!fees) return '-'
  return `${nqtToNxt(fees)} NRC`
})

/**
 * 刷新数据：重新拉取账户控制状态。
 */
async function refreshData(): Promise<void> {
  loading.value = true
  try {
    await accountStore.updateAccountControlStatus()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  // 如果 store 中尚无控制状态，主动拉取一次
  if (!phasingOnlyControl.value) {
    refreshData()
  }
})
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-lg;

    .page-title {
      font-size: $font-size-lg;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
      color: $text-primary;
    }

    .header-actions {
      display: flex;
      gap: $space-sm;
    }
  }
}

.control-card {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9)) !important;
}

.no-control {
  text-align: center;
  padding: $space-xl 0;

  .no-control-icon {
    font-size: 48px;
    color: $text-muted;
    margin-bottom: $space-md;
  }

  .no-control-text {
    font-size: $font-size-md;
    color: $text-secondary;
    margin-bottom: $space-xs;
  }

  .no-control-hint {
    font-size: $font-size-sm;
    color: $text-muted;
  }
}

.control-info {
  .whitelist-section {
    margin-top: $space-lg;

    .section-title {
      display: flex;
      align-items: center;
      gap: $space-xs;
      font-size: $font-size-sm;
      font-weight: 600;
      color: $text-secondary;
      margin-bottom: $space-sm;

      .count-badge {
        color: $text-muted;
        font-weight: 400;
      }
    }

    .whitelist-list {
      display: flex;
      flex-wrap: wrap;
      gap: $space-xs;

      .whitelist-tag {
        font-family: $font-mono;
        font-size: $font-size-xs;
      }
    }
  }

  .control-notice {
    margin-top: $space-lg;
  }
}

.text-mono {
  font-family: $font-mono;
  font-size: $font-size-sm;
}
</style>
