<template>
  <el-dialog
    v-model="visible"
    :title="t('monetary.foundersModalTitle', { code: currency?.code || '' })"
    width="780px"
    :close-on-click-modal="false"
    destroy-on-close
    class="nrcs-modal"
    @close="handleClose"
  >
    <div v-loading="loading">
      <!-- 货币概览信息（对标 founders_blocks_active / founders_reserve_units / founders_issuer_units） -->
      <el-descriptions :column="3" border size="small" class="mb-16">
        <el-descriptions-item :label="t('monetary.foundersBlocksActive')">
          <span v-if="blocksActive > 0">{{ blocksActive }}</span>
          <el-tag v-else type="success" size="small">Active</el-tag>
        </el-descriptions-item>
        <el-descriptions-item :label="t('monetary.foundersReserveUnits')">
          {{ formatQNT(reserveSupply, decimals) }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('monetary.foundersIssuerUnits')">
          {{ formatQNT(initialSupply, decimals) }}
        </el-descriptions-item>
      </el-descriptions>

      <!-- 创建者表格（对标 currency_founders_table） -->
      <el-table :data="foundersRows" stripe size="small" :empty-text="t('common.noData')">
        <el-table-column prop="accountRS" :label="t('monetary.foundersAccount')" min-width="180" />
        <el-table-column prop="amountPerUnitNRC" :label="t('monetary.foundersAmountPerUnit')" align="right" width="150" />
        <el-table-column prop="totalNRC" :label="t('monetary.foundersTotalUnits')" align="right" width="150" />
        <el-table-column prop="units" :label="t('monetary.units')" align="right" width="120" />
        <el-table-column prop="percentage" :label="t('monetary.foundersPercentage')" align="right" width="100" />
      </el-table>

      <!-- 合计行（对标参考 rows += "<tr><td><b>Totals</b></td>..."） -->
      <el-descriptions :column="5" border size="small" class="mt-16">
        <el-descriptions-item :label="t('monetary.foundersTotals')"></el-descriptions-item>
        <el-descriptions-item :label="''">
          <b>{{ formatNqtToNrc(totalAmountReservedNqt) }} NRC</b>
        </el-descriptions-item>
        <el-descriptions-item :label="''">
          <b>{{ formatNqtToNrc(totalAmountReservedNqt * reserveSupplyWhole) }} NRC</b>
        </el-descriptions-item>
        <el-descriptions-item :label="''">
          <b>{{ formatQNT(reserveSupplyMinusInitial, decimals) }}</b>
        </el-descriptions-item>
        <el-descriptions-item :label="''">
          <b>{{ totalPercentage }}%</b>
        </el-descriptions-item>
      </el-descriptions>
    </div>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.close') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * CurrencyFoundersModal.vue —— 货币创建者详情弹窗。
 *
 * 对标参考 `nrs.monetarysystem.js:409-472` 的 `#currency_founders_modal`。
 *
 * 流程：
 *   1. 打开时调用 `getCurrencyFounders` 获取创建者列表
 *   2. 对每个创建者计算：
 *      - amountPerUnitNQT × 10^decimals → 每 whole unit 的 NRC
 *      - amountPerUnitNQT × reserveSupply → 总 NRC
 *      - percentage = amountPerUnitNQT / minReservePerUnitNQT × 100
 *   3. 显示合计行（totalAmountReserved / totalUnits / totalPercentage）
 *
 * 对标 `NRS.calculatePercentage(amountPerUnitNQT, minReservePerUnitNQT)`。
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useNodeStore } from '@/stores/modules/node.store'
import { formatNqtToNrc, qntToQntf } from '@/utils/format'

const props = defineProps<{ currency: any }>()
const visible = defineModel<boolean>('visible', { default: false })

const { t } = useI18n()
const nodeStore = useNodeStore()

const loading = ref(false)
const founders = ref<any[]>([])

/** 货币参数（从 props.currency 提取，对标 $invoker.data(...)） */
const decimals = computed(() => Number(props.currency?.decimals ?? 0))
const issueHeight = computed(() => Number(props.currency?.issuanceHeight ?? 0))
const reserveSupply = computed(() => String(props.currency?.reserveSupply ?? '0'))
const initialSupply = computed(() => String(props.currency?.initialSupply ?? '0'))
const minReservePerUnitNQT = computed(() => String(props.currency?.minReservePerUnitNQT ?? '0'))

/** 剩余区块（对标 founders_blocks_active = issueHeight - lastBlockHeight） */
const blocksActive = computed(() => {
  const last = nodeStore.lastBlockHeight || 0
  return Math.max(0, issueHeight.value - last)
})

/** minReservePerUnitNQT × 10^decimals（对标参考 minReservePerUnitNQT = minReserve * 10^decimals） */
const minReservePerUnitNqtScaled = computed(() => {
  try {
    const v = BigInt(minReservePerUnitNQT.value || '0')
    const factor = 10n ** BigInt(decimals.value)
    return v * factor
  } catch {
    return 0n
  }
})

/** reserveSupply 的 whole unit 表示（对标 NRS.convertToQNTf(resSupply, decimals)） */
const reserveSupplyWhole = computed(() => {
  try {
    return BigInt(Math.round(Number(qntToQntf(reserveSupply.value, decimals.value))))
  } catch {
    return 0n
  }
})

/** reserveSupply - initialSupply（对标 resSupply.subtract(initialSupply)） */
const reserveSupplyMinusInitial = computed(() => {
  try {
    const res = BigInt(reserveSupply.value || '0')
    const init = BigInt(initialSupply.value || '0')
    return (res - init).toString()
  } catch {
    return '0'
  }
})

interface FounderRow {
  accountRS: string
  amountPerUnitNRC: string
  totalNRC: string
  units: string
  percentage: string
}

/** 创建者行数据（对标参考 founders 循环构建 rows） */
const foundersRows = computed<FounderRow[]>(() => {
  if (!founders.value.length) return []
  const rows: FounderRow[] = []
  let totalAmountReserved = 0n
  // 先算总储备金额（对标参考第一次循环）
  for (const f of founders.value) {
    try {
      const amountPerUnit = BigInt(String(f.amountPerUnitNQT || '0'))
      const scaled = amountPerUnit * 10n ** BigInt(decimals.value)
      totalAmountReserved += scaled
    } catch {
      // 忽略
    }
  }
  // 第二次循环构建行（对标参考第二次循环）
  for (const f of founders.value) {
    try {
      const amountPerUnit = BigInt(String(f.amountPerUnitNQT || '0'))
      const scaled = amountPerUnit * 10n ** BigInt(decimals.value)
      const amountPerUnitNRC = formatNqtToNrc(scaled.toString())
      const totalNRC = formatNqtToNrc((scaled * reserveSupplyWhole.value).toString())
      // units = (resSupply - initialSupply) * amountPerUnitNQT / totalAmountReserved
      let units = '0'
      if (totalAmountReserved > 0n) {
        const resMinusInit = BigInt(reserveSupplyMinusInitial.value)
        const unitsQNT = (resMinusInit * scaled) / totalAmountReserved
        units = qntToQntf(unitsQNT.toString(), decimals.value)
      }
      // percentage = amountPerUnitNQT / minReservePerUnitNQT × 100
      let percentage = '0.00'
      if (minReservePerUnitNqtScaled.value > 0n) {
        // 使用 BigInt 计算：percentage = (scaled * 10000 / minReserve) / 100
        const pctScaled = (scaled * 10000n) / minReservePerUnitNqtScaled.value
        percentage = (Number(pctScaled) / 100).toFixed(2)
      }
      rows.push({
        accountRS: f.accountRS || f.account || '-',
        amountPerUnitNRC,
        totalNRC,
        units,
        percentage,
      })
    } catch {
      // 忽略单行错误
    }
  }
  return rows
})

/** 合计金额（对标 totalAmountReserved） */
const totalAmountReservedNqt = computed(() => {
  let total = 0n
  for (const f of founders.value) {
    try {
      const amountPerUnit = BigInt(String(f.amountPerUnitNQT || '0'))
      total += amountPerUnit * 10n ** BigInt(decimals.value)
    } catch {
      // 忽略
    }
  }
  return total
})

/** 合计百分比（对标 NRS.calculatePercentage(totalAmountReserved, minReservePerUnitNQT)） */
const totalPercentage = computed(() => {
  if (minReservePerUnitNqtScaled.value <= 0n) return '0.00'
  const pctScaled = (totalAmountReservedNqt.value * 10000n) / minReservePerUnitNqtScaled.value
  return (Number(pctScaled) / 100).toFixed(2)
})

/**
 * 打开时加载创建者列表（对标 foundersModal show.bs.modal 调用 getCurrencyFounders）。
 */
watch(visible, async (val) => {
  if (!val || !props.currency?.currency) return
  loading.value = true
  founders.value = []
  try {
    const result = await nrcsApi.getCurrencyFounders(props.currency.currency)
    founders.value = result?.founders || []
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
})

/** 格式化 QNT 数量（对标 NRS.formatQuantity） */
function formatQNT(qnt: string, decimals: number): string {
  if (!qnt) return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

function handleClose(): void {
  founders.value = []
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.mb-16 {
  margin-bottom: 16px;
}
.mt-16 {
  margin-top: 16px;
}
</style>
