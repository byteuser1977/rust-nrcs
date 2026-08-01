<template>
  <el-dialog
    v-model="visible"
    :title="t('asset.assetDistribution')"
    width="720px"
    :close-on-click-modal="false"
    destroy-on-close
    class="asset-holders-modal"
    @close="handleClose"
  >
    <div v-if="asset" class="modal-summary">
      <span class="summary-item">
        <span class="label">{{ t('asset.assetName') }}:</span>
        <span class="value">{{ asset.name }}</span>
      </span>
      <span class="summary-item">
        <span class="label">{{ t('asset.totalSupply') }}:</span>
        <span class="value text-mono">{{ formatQNT(asset.quantityQNT, asset.decimals) }}</span>
      </span>
      <span class="summary-item">
        <span class="label">{{ t('asset.holdersCount') }}:</span>
        <span class="value">{{ holders.length }}</span>
      </span>
    </div>

    <el-table
      v-loading="loading"
      :data="pagedHolders"
      stripe
      size="small"
      :empty-text="t('common.noData')"
      max-height="420"
    >
      <el-table-column :label="t('common.serial')" width="60" align="center">
        <template #default="{ $index }">{{ (currentPage - 1) * pageSize + $index + 1 }}</template>
      </el-table-column>
      <el-table-column :label="t('common.account')" min-width="200">
        <template #default="{ row }">
          <span class="text-mono">{{ row.accountRS }}</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('asset.quantity')" width="160" align="right">
        <template #default="{ row }">
          <span class="text-mono">{{ formatQNT(row.quantityQNT, asset?.decimals || 0) }}</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('asset.percentage')" width="120" align="right">
        <template #default="{ row }">
          <span class="text-mono">{{ calculatePercentage(row.quantityQNT) }}%</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('asset.issuer')" width="80" align="center">
        <template #default="{ row }">
          <el-tag v-if="isIssuer(row)" size="small" type="warning">{{ t('asset.issuer') }}</el-tag>
        </template>
      </el-table-column>
    </el-table>

    <el-pagination
      v-if="holders.length > pageSize"
      v-model:current-page="currentPage"
      :page-size="pageSize"
      :total="holders.length"
      layout="prev, pager, next, total"
      class="pagination"
      small
      background
    />

    <template #footer>
      <el-button @click="handleClose">{{ t('common.close') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * AssetHoldersModal 组件 —— 资产持有人分布弹窗。
 *
 * 对标 nrs.assetexchange.js:1164 NRS.getAssetAccounts：
 *   调用 getAssetAccounts API 获取指定资产的所有持有人及其持仓 QNT，
 *   展示账户 RS、持仓数量、占总量的百分比、是否为发行方。
 *
 * 参考源码位置：nrs.assetexchange.js:1634（getAssetAccounts 调用处）。
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi, type NrcsAsset } from '@/api/modules/nrcs.api'
import { qntToQntf } from '@/utils/format'

const { t } = useI18n()

const props = defineProps<{
  /** 资产信息 */
  asset: NrcsAsset | null
  /** 查询高度（可选，默认为当前高度） */
  height?: number
}>()

const visible = defineModel<boolean>('visible', { default: false })

/** 持有人记录（对标 getAssetAccounts 响应的 accountAssets 元素） */
interface AssetHolder {
  account: string
  accountRS: string
  quantityQNT: string
  unconfirmedQuantityQNT?: string
}

const holders = ref<AssetHolder[]>([])
const loading = ref(false)
const currentPage = ref(1)
const pageSize = 20

/** 当前页持有人 */
const pagedHolders = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return holders.value.slice(start, start + pageSize)
})

/** 格式化 QNT 为可读数量 */
function formatQNT(qnt: string | undefined, decimals: number): string {
  if (!qnt) return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

/** 计算持仓占总量的百分比 */
function calculatePercentage(quantityQNT: string): string {
  if (!props.asset || !props.asset.quantityQNT) return '0.00'
  try {
    const qty = BigInt(quantityQNT)
    const total = BigInt(props.asset.quantityQNT)
    if (total === BigInt(0)) return '0.00'
    // 保留 2 位小数：× 10000 / total → 整数部分 ÷ 100
    const percentage = (qty * BigInt(10000)) / total
    const intPart = Number(percentage / BigInt(100))
    const fracPart = Number(percentage % BigInt(100))
    return `${intPart}.${String(fracPart).padStart(2, '0')}`
  } catch {
    return '0.00'
  }
}

/** 是否为发行方 */
function isIssuer(row: AssetHolder): boolean {
  return !!props.asset && row.account === props.asset.issuer
}

/**
 * 加载资产持有人列表。
 *
 * 对标 NRS.getAssetAccounts（:1164-1172），调用 getAssetAccounts API。
 */
async function loadHolders(): Promise<void> {
  if (!props.asset) return
  loading.value = true
  currentPage.value = 1
  try {
    const result = (await nrcsApi.getAssetAccounts(
      props.asset.asset,
    )) as any
    if (result.errorCode) {
      throw new Error(result.errorDescription || 'Failed to load asset accounts')
    }
    holders.value = result.accountAssets || []
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
    holders.value = []
  } finally {
    loading.value = false
  }
}

/** 监听弹窗打开：自动加载持有人 */
watch(visible, (open) => {
  if (open && props.asset) {
    loadHolders()
  }
})

function handleClose(): void {
  holders.value = []
  currentPage.value = 1
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.modal-summary {
  display: flex;
  gap: $space-lg;
  padding: $space-md;
  background: var(--el-fill-color-light);
  border-radius: $radius-sm;
  margin-bottom: $space-md;
  flex-wrap: wrap;

  .summary-item {
    display: flex;
    flex-direction: column;
    gap: 2px;

    .label {
      font-size: $font-size-xs;
      color: $text-muted;
    }

    .value {
      font-weight: 600;
      color: $text-primary;
    }
  }
}

.text-mono {
  font-family: $font-mono;
}

.pagination {
  margin-top: $space-md;
  display: flex;
  justify-content: flex-end;
}
</style>
