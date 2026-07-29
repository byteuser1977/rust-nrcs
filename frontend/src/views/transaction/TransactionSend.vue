<template>
  <div class="transaction-send-page">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <span>{{ t('transaction.send') }}</span>
        </div>
      </template>

      <div class="send-intro">
        <el-result
          icon="success"
          :title="t('transaction.sendMoney')"
          :sub-title="t('transaction.sendMoneyIntro')"
        >
          <template #extra>
            <el-button type="primary" @click="openSendModal">
              <el-icon><Promotion /></el-icon>
              {{ t('transaction.sendMoney') }}
            </el-button>
          </template>
        </el-result>

        <el-descriptions :column="1" border style="margin-top: 24px;">
          <el-descriptions-item :label="t('transaction.sender')">
            <span class="mono-text">{{ accountRS || t('transaction.notLoggedIn') }}</span>
          </el-descriptions-item>
          <el-descriptions-item :label="t('transaction.balance')">
            <span class="balance-value">{{ balanceFormatted }} NRC</span>
          </el-descriptions-item>
          <el-descriptions-item :label="t('transaction.feeHint')">
            {{ t('transaction.minFee') }}: 1 NRC
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { Promotion } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const router = useRouter()
const accountStore = useAccountStore()

const accountRS = accountStore.accountRS
const balanceFormatted = accountStore.balanceFormatted

function openSendModal() {
  // Navigate to the full SendTransaction form
  router.push('/transaction/send-form')
}
</script>

<style scoped lang="scss">
.transaction-send-page {
  .card-header {
    font-size: 16px;
    font-weight: 600;
  }

  .send-intro {
    .mono-text {
      font-family: 'Roboto Mono', monospace;
      font-size: 14px;
    }

    .balance-value {
      font-family: 'Roboto Mono', monospace;
      font-size: 16px;
      font-weight: 600;
      color: #303133;
    }
  }
}
</style>
