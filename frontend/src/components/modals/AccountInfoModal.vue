<template>
  <el-dialog
    v-model="visible"
    :title="t('accountInfo.title')"
    width="860px"
    :close-on-click-modal="false"
    destroy-on-close
    class="account-info-modal"
    @close="handleClose"
  >
    <!-- 加载中 -->
    <div v-if="loading" class="account-info-loading">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>{{ t('common.loading') }}</span>
    </div>

    <template v-else-if="account">
      <!-- 错误提示 -->
      <el-alert
        v-if="error"
        :title="error"
        type="error"
        :closable="false"
        show-icon
        class="account-info-error"
      />

      <!-- 账户头部信息（对标 nrs.modals.account.js:54-77 showAccountModal） -->
      <div class="account-info-header">
        <div class="account-info-header-row">
          <span class="account-info-label">{{ t('accountDetails.accountRS') }}:</span>
          <span class="account-info-value mono">{{ account.accountRS || account.account }}</span>
        </div>
        <div v-if="account.name" class="account-info-header-row">
          <span class="account-info-label">{{ t('accountInfo.name') }}:</span>
          <span class="account-info-value">{{ account.name }}</span>
        </div>
        <div class="account-info-header-row">
          <span class="account-info-label">{{ t('accountInfo.balance') }}:</span>
          <span class="account-info-value">{{ formattedBalance }} NRC</span>
        </div>
      </div>

      <!-- 账户描述（对标 :96-101） -->
      <div v-if="account.description" class="account-info-description">
        <span class="account-info-label">{{ t('accountInfo.description') }}:</span>
        <p class="account-info-description-text">{{ account.description }}</p>
      </div>

      <!-- 操作按钮（对标 :55-64） -->
      <div class="account-info-actions">
        <el-button size="small" type="primary" @click="onSendMoney">{{ t('accountInfo.sendMoney') }}</el-button>
        <el-button size="small" @click="onSendMessage">{{ t('accountInfo.sendMessage') }}</el-button>
        <el-button size="small" @click="onAddContact">{{ t('accountInfo.addAsContact') }}</el-button>
      </div>

      <!-- 标签页（对标 :133-145 ul.nav li click） -->
      <el-tabs v-model="activeTab" class="account-info-tabs" @tab-change="onTabChange">
        <!-- 交易标签页（对标 :163-209 transactions） -->
        <el-tab-pane :label="t('accountInfo.transactions')" name="transactions">
          <el-table
            v-loading="tabLoading.transactions"
            :data="transactions"
            stripe
            size="small"
            :empty-text="t('accountInfo.noTransactions')"
            @row-click="onTransactionClick"
          >
            <el-table-column :label="t('accountInfo.timestamp')" width="150">
              <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
            </el-table-column>
            <el-table-column :label="t('accountInfo.amount')" width="120" align="right">
              <template #default="{ row }">
                <span :style="{ color: isReceiving(row) ? '#006400' : row.amountNQT && row.amountNQT !== '0' ? 'red' : '' }">
                  {{ !isReceiving(row) && row.amountNQT && row.amountNQT !== '0' ? '-' : '' }}{{ formatAmount(row.amountNQT || '0') }}
                </span>
              </template>
            </el-table-column>
            <el-table-column :label="t('accountInfo.fee')" width="100" align="right">
              <template #default="{ row }">
                <span :style="{ color: !isReceiving(row) ? 'red' : '' }">{{ formatAmount(row.feeNQT || '0') }}</span>
              </template>
            </el-table-column>
            <el-table-column :label="t('accountInfo.account')" min-width="200">
              <template #default="{ row }">
                <span class="mono">{{ isReceiving(row) ? row.senderRS || row.sender : row.recipientRS || row.recipient }}</span>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <!-- 别名标签页（对标 :235-262 aliases） -->
        <el-tab-pane :label="t('accountInfo.aliases')" name="aliases">
          <el-table
            v-loading="tabLoading.aliases"
            :data="aliases"
            stripe
            size="small"
            :empty-text="t('accountInfo.noAliases')"
          >
            <el-table-column prop="aliasName" :label="t('accountInfo.aliasName')" min-width="200" />
            <el-table-column :label="t('accountInfo.uri')" min-width="300">
              <template #default="{ row }">
                <a v-if="String(row.aliasURI || '').indexOf('http') === 0" :href="row.aliasURI" target="_blank" rel="noopener">{{ row.aliasURI }}</a>
                <span v-else>{{ row.aliasURI }}</span>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <!-- 资产标签页（对标 :318-358 assets） -->
        <el-tab-pane :label="t('accountInfo.assets')" name="assets">
          <el-table
            v-loading="tabLoading.assets"
            :data="assets"
            stripe
            size="small"
            :empty-text="t('accountInfo.noAssets')"
          >
            <el-table-column prop="name" :label="t('accountInfo.assetName')" min-width="200">
              <template #default="{ row }">
                <span :style="{ fontWeight: row.issued ? 'bold' : 'normal' }">{{ row.name }}</span>
              </template>
            </el-table-column>
            <el-table-column :label="t('accountInfo.quantity')" width="150" align="right">
              <template #default="{ row }">{{ formatQuantity(row.balanceQNT || '0', row.decimals || 0) }}</template>
            </el-table-column>
            <el-table-column :label="t('accountInfo.total')" width="150" align="right">
              <template #default="{ row }">{{ formatQuantity(row.quantityQNT || '0', row.decimals || 0) }}</template>
            </el-table-column>
            <el-table-column :label="t('accountInfo.percentage')" width="100" align="right">
              <template #default="{ row }">{{ calculatePercentage(row.balanceQNT || '0', row.quantityQNT || '1') }}%</template>
            </el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </template>

    <!-- 空状态 -->
    <el-empty v-else :description="t('accountInfo.notFound')" />

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * AccountInfoModal 组件 —— 任意账户详情弹窗。
 *
 * 对标 nrs.modals.account.js（464 行）中的 user_info_modal。
 *
 * 主要功能：
 *   1. 通过 getAccount 拉取账户基本信息（名称/描述/余额）
 *   2. 标签页懒加载：
 *      - 交易（getBlockchainTransactions，对标 :163-209）
 *      - 别名（getAliases，对标 :235-262）
 *      - 资产（getAccount includeAssets + getAssetsByIssuer，对标 :318-462）
 *   3. 操作按钮：发送 NRC / 发送消息 / 添加联系人
 *   4. 交易列表区分收入/支出（绿色/红色）
 *
 * 对标参考：NRS.showAccountModal(account) / NRS.processAccountModalData(account)
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatAmount, formatTimestamp, qntToQntf } from '@/utils/format'

const props = defineProps<{
  /** 账户 RS 地址或数字 ID */
  account?: string | null
}>()

const emit = defineEmits<{
  sendMoney: [recipient: string]
  sendMessage: [recipient: string]
  addContact: [account: string]
  transactionClick: [transactionId: string]
  close: []
}>()

const { t } = useI18n()

const visible = defineModel<boolean>('visible', { default: false })

const loading = ref(false)
const error = ref<string | null>(null)
const account = ref<any | null>(null)
const activeTab = ref<'transactions' | 'aliases' | 'assets'>('transactions')

/** 标签页加载状态 */
const tabLoading = ref({
  transactions: false,
  aliases: false,
  assets: false,
})

/** 标签页是否已加载（懒加载，对标 :142-144 data-loading） */
const tabLoaded = ref({
  transactions: false,
  aliases: false,
  assets: false,
})

// 数据
const transactions = ref<any[]>([])
const aliases = ref<any[]>([])
const assets = ref<any[]>([])

// ----------------------------------------------------------------
// 计算属性
// ----------------------------------------------------------------

/** 格式化余额（对标 :83-87） */
const formattedBalance = computed<string>(() => {
  if (!account.value) return '0'
  const nqt = account.value.unconfirmedBalanceNQT || '0'
  if (nqt === '0') return '0'
  return formatAmount(nqt)
})

// ----------------------------------------------------------------
// 数据加载
// ----------------------------------------------------------------

/**
 * 加载账户基本信息（对标 nrs.modals.account.js:68-73 getAccount）。
 *
 * @param accountRsOrId 账户 RS 或数字 ID
 */
async function loadAccount(accountRsOrId: string): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const resp = await nrcsApi.getAccount(accountRsOrId)
    if (!resp || (resp as any).errorCode) {
      throw new Error((resp as any)?.errorDescription || t('accountInfo.notFound'))
    }
    account.value = resp
    // 加载默认标签页（交易）
    await loadTab('transactions')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    account.value = null
  } finally {
    loading.value = false
  }
}

/**
 * 加载标签页数据（对标 :142-144 懒加载机制）。
 *
 * @param tab 标签名
 */
async function loadTab(tab: 'transactions' | 'aliases' | 'assets'): Promise<void> {
  if (tabLoaded.value[tab]) return
  tabLoading.value[tab] = true
  try {
    switch (tab) {
      case 'transactions':
        await loadTransactions()
        break
      case 'aliases':
        await loadAliases()
        break
      case 'assets':
        await loadAssets()
        break
    }
    tabLoaded.value[tab] = true
  } catch (e) {
    console.error(`Failed to load tab ${tab}:`, e)
  } finally {
    tabLoading.value[tab] = false
  }
}

/**
 * 加载交易列表（对标 :163-209 transactions）。
 */
async function loadTransactions(): Promise<void> {
  if (!props.account) return
  const resp = await nrcsApi.getBlockchainTransactions(props.account, 0, 100)
  transactions.value = resp.transactions || []
}

/**
 * 加载别名列表（对标 :235-262 aliases）。
 */
async function loadAliases(): Promise<void> {
  if (!props.account) return
  const resp = await nrcsApi.getAliases(props.account, 0, 100)
  const list = resp.aliases || []
  // 按名称排序（对标 :243-252）
  list.sort((a: any, b: any) => a.aliasName.toLowerCase().localeCompare(b.aliasName.toLowerCase()))
  aliases.value = list
}

/**
 * 加载资产列表（对标 :318-462 assets）。
 *
 * 先调 getAccount(includeAssets) 获取余额，再对非零余额资产调 getAsset 获取详情，
 * 最后调 getAssetsByIssuer 合并发行方资产。
 */
async function loadAssets(): Promise<void> {
  if (!props.account) return
  const resp = await nrcsApi.getAccount(props.account, { includeAssets: true })
  const balances = resp.assetBalances || []
  const assetMap: Record<string, any> = {}

  // 加载非零余额资产详情（对标 :327-357）
  for (const bal of balances) {
    if (bal.balanceQNT === '0') continue
    try {
      const assetInfo = await nrcsApi.getAsset(bal.asset)
      assetInfo.asset = bal.asset
      assetInfo.balanceQNT = bal.balanceQNT
      assetInfo.issued = false
      assetMap[bal.asset] = assetInfo
    } catch {
      // 忽略单个资产加载失败
    }
  }

  assets.value = Object.values(assetMap).sort((a: any, b: any) => {
    // 发行方优先（对标 :424-445）
    if (a.issued && !b.issued) return -1
    if (!a.issued && b.issued) return 1
    return a.name.toLowerCase().localeCompare(b.name.toLowerCase())
  })
}

// ----------------------------------------------------------------
// 辅助函数
// ----------------------------------------------------------------

/**
 * 判断交易是否为收款（对标 :182-186 receiving）。
 *
 * @param tx 交易对象
 * @returns 是否收款
 */
function isReceiving(tx: any): boolean {
  if (!props.account) return false
  if (/^NRCS-/i.test(props.account)) {
    return tx.recipientRS === props.account
  }
  return tx.recipient === props.account
}

/**
 * 格式化资产数量（对标 NRS.formatQuantity）。
 *
 * @param qnt 数量 QNT
 * @param decimals 小数位
 * @returns 格式化后的字符串
 */
function formatQuantity(qnt: string, decimals: number): string {
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

/**
 * 计算占比百分比（对标 :455 calculatePercentage）。
 *
 * @param balanceQNT 持有数量
 * @param quantityQNT 总量
 * @returns 百分比字符串
 */
function calculatePercentage(balanceQNT: string, quantityQNT: string): string {
  try {
    const bal = BigInt(balanceQNT)
    const total = BigInt(quantityQNT)
    if (total === 0n) return '0'
    return ((bal * 10000n) / total).toString()
  } catch {
    return '0'
  }
}

// ----------------------------------------------------------------
// 事件处理
// ----------------------------------------------------------------

/**
 * 标签页切换（对标 :133-145）。
 *
 * @param tab 新标签名
 */
function onTabChange(tab: string): void {
  if (tab === 'transactions' || tab === 'aliases' || tab === 'assets') {
    loadTab(tab)
  }
}

function onSendMoney(): void {
  if (account.value) emit('sendMoney', account.value.accountRS || account.value.account)
}

function onSendMessage(): void {
  if (account.value) emit('sendMessage', account.value.accountRS || account.value.account)
}

function onAddContact(): void {
  if (account.value) emit('addContact', account.value.accountRS || account.value.account)
}

function onTransactionClick(row: any): void {
  if (row.transaction) emit('transactionClick', row.transaction)
}

function handleClose(): void {
  visible.value = false
  account.value = null
  error.value = null
  transactions.value = []
  aliases.value = []
  assets.value = []
  tabLoaded.value = { transactions: false, aliases: false, assets: false }
  activeTab.value = 'transactions'
  emit('close')
}

// ----------------------------------------------------------------
// 监听 props 变化自动加载
// ----------------------------------------------------------------

watch(
  () => [props.account, visible.value] as const,
  async ([newAccount, isVisible]) => {
    if (isVisible && newAccount) {
      // 重置标签页加载状态
      tabLoaded.value = { transactions: false, aliases: false, assets: false }
      activeTab.value = 'transactions'
      await loadAccount(newAccount)
    }
  },
  { immediate: true },
)
</script>

<style scoped lang="scss">
.account-info-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: var(--el-text-color-secondary);
}
.account-info-error {
  margin-bottom: 12px;
}
.account-info-header {
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  margin-bottom: 12px;
  .account-info-header-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
    &:last-child { margin-bottom: 0; }
  }
}
.account-info-label {
  color: var(--el-text-color-secondary);
  font-size: 13px;
  min-width: 80px;
}
.account-info-value {
  font-size: 14px;
  word-break: break-all;
}
.account-info-description {
  margin-bottom: 12px;
  .account-info-description-text {
    margin: 4px 0 0 0;
    white-space: pre-wrap;
    word-break: break-word;
  }
}
.account-info-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}
.account-info-tabs {
  margin-top: 4px;
}
.mono {
  font-family: 'Roboto Mono', monospace;
  font-size: 13px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
