<template>
  <el-dialog
    v-model="visible"
    :title="t('accountDetails.title')"
    width="640px"
    :close-on-click-modal="false"
    destroy-on-close
    class="account-details-modal"
    @close="handleClose"
  >
    <!-- 余额信息表（对标 nrs.modals.accountdetails.js:42-70） -->
    <div v-if="balanceWarning" class="account-details-warning">
      <el-alert :title="balanceWarning" type="warning" :closable="false" show-icon />
    </div>

    <InfoTable v-if="balanceRows.length > 0" :rows="balanceRows" :column="2" />

    <!-- QR 码区域（对标 :25-38, :106-118） -->
    <div class="account-details-qr">
      <h4 class="account-details-qr-title">{{ t('accountDetails.qrCode') }}</h4>
      <div class="account-details-qr-content">
        <!-- QR 码切换按钮（对标 :106-118 account_display / passphrase_display 互斥） -->
        <el-radio-group v-if="hasPassphrase" v-model="qrMode" size="small" class="account-details-qr-toggle">
          <el-radio-button value="account">{{ t('accountDetails.showAccount') }}</el-radio-button>
          <el-radio-button value="passphrase">{{ t('accountDetails.showPassphrase') }}</el-radio-button>
        </el-radio-group>

        <div class="account-details-qr-image">
          <img v-if="qrDataUrl" :src="qrDataUrl" :alt="qrMode" class="qr-img" />
          <span v-else class="account-details-qr-na">{{ t('accountDetails.passphraseNotAvailable') }}</span>
        </div>
      </div>
    </div>

    <!-- 纸钱包（对标 :120-122 printPaperWallet） -->
    <div v-if="hasPassphrase" class="account-details-paper-wallet">
      <el-button size="small" @click="printPaperWallet">
        <el-icon><Printer /></el-icon>
        {{ t('accountDetails.paperWallet') }}
      </el-button>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * AccountDetailsModal 组件 —— 当前登录账户详情弹窗。
 *
 * 对标 nrs.modals.accountdetails.js（124 行）的完整实现。
 *
 * 主要功能：
 *   1. 余额表（balance/unconfirmedBalance/effectiveBalance/guaranteedBalance/forgedBalance）
 *   2. 账户 RS / account ID / publicKey 展示
 *   3. QR 码切换：账户地址 ↔ 密码短语（对标 :106-118）
 *   4. 纸钱包打印（对标 :120-122）
 *   5. 无公钥警告（对标 :64-68）
 *
 * 对标参考：accountDetailsModal.on("show.bs.modal", ...)
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Printer } from '@element-plus/icons-vue'
import QRCode from 'qrcode'
import { useAccountStore } from '@/stores/modules/account.store'
import { formatAmount } from '@/utils/format'
import type { InfoRow } from '@/utils/transaction-info-table'
import InfoTable from '@/components/base/InfoTable.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const visible = defineModel<boolean>('visible', { default: false })

/** QR 码模式：account（账户地址）/ passphrase（密码短语） */
const qrMode = ref<'account' | 'passphrase'>('account')
/** QR 码 data URL */
const qrDataUrl = ref<string>('')

/** 是否拥有密码短语（对标 _password 是否设置） */
const hasPassphrase = computed<boolean>(() => accountStore.hasSecretPhrase)

// ----------------------------------------------------------------
// 计算属性
// ----------------------------------------------------------------

/**
 * 余额信息表行（对标 nrs.modals.accountdetails.js:53-69）。
 *
 * 展示：
 *   - 账户 RS
 *   - 账户 ID
 *   - 公钥（无公钥时显示 "/"）
 *   - 余额 / 未确认余额 / 有效余额 / 保证余额 / 锻造余额
 */
const balanceRows = computed<InfoRow[]>(() => {
  const info = accountStore.accountInfo
  const rows: InfoRow[] = []

  rows.push({ label: t('accountDetails.accountRS'), value: accountStore.accountRS || '-' })
  rows.push({ label: t('accountDetails.accountId'), value: accountStore.accountId || '-' })

  // 公钥（对标 :64-68，无公钥显示 "/"）
  const pk = info?.publicKey || accountStore.publicKey
  rows.push({ label: t('accountDetails.publicKey'), value: pk || '/' })

  // 余额（对标 :54-58）
  if (info) {
    rows.push({
      label: t('accountDetails.balance'),
      value: `${formatAmount(info.balanceNQT || '0')} NRC`,
    })
    rows.push({
      label: t('accountDetails.unconfirmedBalance'),
      value: `${formatAmount(info.unconfirmedBalanceNQT || '0')} NRC`,
    })
    rows.push({
      label: t('accountDetails.effectiveBalance'),
      value: `${formatAmount(String(info.effectiveBalanceNRCS || 0))} NRC`,
    })
    rows.push({
      label: t('accountDetails.guaranteedBalance'),
      value: `${formatAmount(info.guaranteedBalanceNQT || '0')} NRC`,
    })
    rows.push({
      label: t('accountDetails.forgedBalance'),
      value: `${formatAmount(info.forgedBalanceNQT || '0')} NRC`,
    })
  } else {
    // 回退到 store 中的独立字段
    rows.push({
      label: t('accountDetails.balance'),
      value: `${formatAmount(accountStore.balanceNQT || '0')} NRC`,
    })
    rows.push({
      label: t('accountDetails.unconfirmedBalance'),
      value: `${formatAmount(accountStore.unconfirmedBalanceNQT || '0')} NRC`,
    })
    rows.push({
      label: t('accountDetails.effectiveBalance'),
      value: `${formatAmount(String(accountStore.effectiveBalance || 0))} NRC`,
    })
    rows.push({
      label: t('accountDetails.forgedBalance'),
      value: `${formatAmount(accountStore.forgedBalanceNQT || '0')} NRC`,
    })
  }

  return rows
})

/**
 * 余额警告（对标 :64-68）。
 *
 * 当账户无公钥时显示警告。
 */
const balanceWarning = computed<string>(() => {
  const info = accountStore.accountInfo
  if (info && !info.publicKey) {
    if (accountStore.publicKey) {
      return t('accountDetails.publicKeyNotAnnounced')
    }
    return t('accountDetails.balanceWarning')
  }
  return ''
})

// ----------------------------------------------------------------
// QR 码生成
// ----------------------------------------------------------------

/**
 * 生成 QR 码（对标 NRS.generateQRCode）。
 *
 * @param text 待编码文本
 */
async function generateQR(text: string): Promise<void> {
  if (!text) {
    qrDataUrl.value = ''
    return
  }
  try {
    qrDataUrl.value = await QRCode.toDataURL(text, { width: 200, margin: 2 })
  } catch {
    qrDataUrl.value = ''
  }
}

/**
 * 根据 qrMode 生成对应 QR 码（对标 :106-118 互斥切换）。
 */
async function refreshQR(): Promise<void> {
  if (qrMode.value === 'passphrase') {
    if (hasPassphrase.value) {
      await generateQR(accountStore.secretPhrase)
    } else {
      qrDataUrl.value = ''
    }
  } else {
    await generateQR(accountStore.accountRS)
  }
}

// ----------------------------------------------------------------
// 纸钱包
// ----------------------------------------------------------------

/**
 * 打印纸钱包（对标 :120-122 NRS.printPaperWallet）。
 *
 * 打开新窗口，写入账户地址、公钥、密码短语，调用浏览器打印。
 */
function printPaperWallet(): void {
  const win = window.open('', '_blank', 'width=600,height=400')
  if (!win) return
  win.document.write(`
    <html>
    <head><title>${t('accountDetails.paperWallet')}</title></head>
    <body style="font-family: monospace; padding: 20px;">
      <h2>${t('accountDetails.paperWallet')}</h2>
      <p><strong>${t('accountDetails.accountRS')}:</strong> ${accountStore.accountRS}</p>
      <p><strong>${t('accountDetails.accountId')}:</strong> ${accountStore.accountId}</p>
      <p><strong>${t('accountDetails.publicKey')}:</strong> ${accountStore.publicKey || '-'}</p>
      <p><strong>${t('common.secretPhrase')}:</strong> ${accountStore.secretPhrase}</p>
    </body>
    </html>
  `)
  win.document.close()
  win.focus()
  win.print()
}

// ----------------------------------------------------------------
// 事件处理
// ----------------------------------------------------------------

function handleClose(): void {
  visible.value = false
  qrMode.value = 'account'
  qrDataUrl.value = ''
}

// ----------------------------------------------------------------
// 监听
// ----------------------------------------------------------------

watch(
  [visible, qrMode],
  async ([isVisible]) => {
    if (isVisible) {
      qrMode.value = 'account'
      await refreshQR()
    }
  },
  { immediate: true },
)
</script>

<style scoped lang="scss">
.account-details-warning {
  margin-bottom: 12px;
}
.account-details-qr {
  margin-top: 16px;
  .account-details-qr-title {
    margin: 0 0 8px 0;
    font-size: 14px;
    color: var(--el-text-color-secondary);
  }
  .account-details-qr-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .account-details-qr-toggle {
    margin-bottom: 4px;
  }
  .account-details-qr-image {
    width: 200px;
    height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--el-border-color);
    border-radius: 4px;
    .qr-img {
      width: 200px;
      height: 200px;
    }
    .account-details-qr-na {
      font-size: 13px;
      color: var(--el-text-color-secondary);
      text-align: center;
      padding: 20px;
    }
  }
}
.account-details-paper-wallet {
  margin-top: 16px;
  text-align: center;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
