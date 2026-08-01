<template>
  <el-dialog
    v-model="visible"
    :title="t('asset.dividendHistory')"
    width="820px"
    :close-on-click-modal="false"
    destroy-on-close
    class="asset-dividend-modal"
    @close="handleClose"
  >
    <div v-if="asset" class="modal-summary">
      <span class="summary-item">
        <span class="label">{{ t('asset.assetName') }}:</span>
        <span class="value">{{ asset.name }}</span>
      </span>
      <span class="summary-item">
        <span class="label">{{ t('asset.totalDividends') }}:</span>
        <span class="value text-mono">{{ dividends.length }}</span>
      </span>
    </div>

    <el-table
      v-loading="loading"
      :data="pagedDividends"
      stripe
      size="small"
      :empty-text="t('asset.noDividends')"
      max-height="420"
    >
      <el-table-column :label="t('common.date')" width="160">
        <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
      </el-table-column>
      <el-table-column :label="t('asset.dividendHeight')" width="110" align="right">
        <template #default="{ row }">{{ row.dividendHeight }}</template>
      </el-table-column>
      <el-table-column :label="t('asset.amountPerShare')" width="140" align="right">
        <template #default="{ row }">
          <span class="text-mono">{{ formatPricePerShare(row.amountNQTPerQNT) }} NRC</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('asset.totalDividend')" width="140" align="right">
        <template #default="{ row }">
          <span class="text-mono">{{ formatAmount(row.totalDividend) }} NRC</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('asset.accountsCount')" width="100" align="right">
        <template #default="{ row }">{{ row.numberOfAccounts }}</template>
      </el-table-column>
      <el-table-column :label="t('common.transaction')" min-width="120">
        <template #default="{ row }">
          <span class="text-mono tx-link" :title="row.assetDividend">{{ truncateHash(row.assetDividend, 10) }}</span>
        </template>
      </el-table-column>
    </el-table>

    <el-pagination
      v-if="dividends.length > pageSize"
      v-model:current-page="currentPage"
      :page-size="pageSize"
      :total="dividends.length"
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
 * AssetDividendHistoryModal 组件 —— 资产股息历史弹窗。
 *
 * 对标 nrs.assetexchange.js:764 NRS.getAssetDividendHistory：
 *   调用 getAssetDividends API 获取指定资产的历史股息派发记录，
 *   展示时间、分红高度、每股金额、总金额、参与账户数、交易 ID。
 *
 * 参考源码位置：nrs.assetexchange.js:764-807。
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi, type NrcsAsset } from '@/api/modules/nrcs.api'
import {
  formatTimestamp,
  formatAmount,
  formatOrderPricePerWholeQNT,
  truncateHash,
} from '@/utils/format'

const { t } = useI18n()

const props = defineProps<{
  /** 资产信息 */
  asset: NrcsAsset | null
}>()

const visible = defineModel<boolean>('visible', { default: false })

/** 股息记录（对标 getAssetDividends 响应的 dividends 元素） */
interface AssetDividend {
  assetDividend: string
  timestamp: number
  dividendHeight: number
  totalDividend: string
  numberOfAccounts: string | number
  amountNQTPerQNT: string
}

const dividends = ref<AssetDividend[]>([])
const loading = ref(false)
const currentPage = ref(1)
const pageSize = 20

/** 当前页股息记录 */
const pagedDividends = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return dividends.value.slice(start, start + pageSize)
})

/** 格式化每股金额（NQT/QNT → NXT/整股） */
function formatPricePerShare(amountNQTPerQNT: string): string {
  if (!props.asset || !amountNQTPerQNT) return '0'
  try {
    return formatOrderPricePerWholeQNT(amountNQTPerQNT, props.asset.decimals)
  } catch {
    return amountNQTPerQNT
  }
}

/**
 * 加载股息历史。
 *
 * 对标 NRS.getAssetDividendHistory（:764-807），调用 getAssetDividends API。
 */
async function loadDividends(): Promise<void> {
  if (!props.asset) return
  loading.value = true
  currentPage.value = 1
  try {
    const result = (await nrcsApi.getAssetDividends(
      props.asset.asset,
      0,
      100,
    )) as any
    if (result.errorCode) {
      throw new Error(result.errorDescription || 'Failed to load dividends')
    }
    dividends.value = result.dividends || []
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
    dividends.value = []
  } finally {
    loading.value = false
  }
}

/** 监听弹窗打开：自动加载股息历史 */
watch(visible, (open) => {
  if (open && props.asset) {
    loadDividends()
  }
})

function handleClose(): void {
  dividends.value = []
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

.tx-link {
  color: $primary;
  cursor: pointer;

  &:hover {
    text-decoration: underline;
  }
}

.pagination {
  margin-top: $space-md;
  display: flex;
  justify-content: flex-end;
}
</style>
