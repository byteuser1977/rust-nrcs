<template>
  <div class="issue-currency-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Coin /></el-icon> {{ t('monetary.issueCurrency') }}</h2>
    </div>

    <el-row :gutter="16">
      <el-col :span="16">
        <el-card shadow="hover">
          <template #header>
            <h3 class="card-title">{{ t('monetary.issueCurrency') }}</h3>
          </template>

          <div class="intro-section">
            <el-alert
              :title="t('monetary.issueIntro') || 'Create a new currency on the NRCS Monetary System'"
              type="info"
              :closable="false"
              show-icon
              class="mb-16"
            />

            <el-descriptions :column="1" border size="small" class="mb-16">
              <el-descriptions-item :label="t('monetary.code') || 'Currency Code'">
                3-5 uppercase letters (e.g., USD, BTC, GOLD)
              </el-descriptions-item>
              <el-descriptions-item :label="t('common.name') || 'Name'">
                3-10 characters identifying the currency
              </el-descriptions-item>
              <el-descriptions-item :label="t('monetary.type') || 'Type'">
                <div class="type-desc">
                  <el-tag size="small" type="primary" class="mr-2">{{ t('monetary.exchangeable') || 'EXCH' }}</el-tag>
                  Can be traded on the built-in exchange
                </div>
                <div class="type-desc">
                  <el-tag size="small" type="warning" class="mr-2">{{ t('monetary.controllable') || 'CTRL' }}</el-tag>
                  Issuer can control supply after issuance
                </div>
                <div class="type-desc">
                  <el-tag size="small" type="info" class="mr-2">{{ t('monetary.reservable') || 'RESV' }}</el-tag>
                  Backed by NRC reserve per unit
                </div>
                <div class="type-desc">
                  <el-tag size="small" class="mr-2">{{ t('monetary.claimable') || 'CLAM' }}</el-tag>
                  Reserve can be claimed by holders
                </div>
                <div class="type-desc">
                  <el-tag size="small" type="success" class="mr-2">{{ t('monetary.mintable') || 'MINT' }}</el-tag>
                  Additional supply can be minted via PoW
                </div>
              </el-descriptions-item>
              <el-descriptions-item :label="t('monetary.decimals') || 'Decimals'">
                Number of decimal places (0-8) for the currency
              </el-descriptions-item>
              <el-descriptions-item :label="t('monetary.initialSupply') || 'Initial Supply'">
                The initial amount of currency units to create
              </el-descriptions-item>
            </el-descriptions>

            <div class="action-center">
              <el-button type="primary" size="large" @click="showIssue = true">
                <el-icon><Plus /></el-icon> {{ t('monetary.issueCurrency') }}
              </el-button>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :span="8">
        <el-card shadow="hover">
          <template #header>
            <h3 class="card-title">{{ t('monetary.issueInfo') || 'About Currency Issuance' }}</h3>
          </template>
          <div class="info-text">
            <p>{{ t('monetary.issueInfoText') || 'Issuing a currency on the NRCS Monetary System creates a new digital asset that can be traded, transferred, and exchanged on the blockchain.' }}</p>
            <p class="mt-8">{{ t('monetary.issueInfoFee') || 'A transaction fee applies. Currency issuance requires a minimum fee of 1000 NRC for the base transaction plus additional fees based on currency properties.' }}</p>
            <p class="mt-8">{{ t('monetary.issueInfoIrreversible') || 'Currency issuance is irreversible. Once created, the currency code and type cannot be changed. Please review all parameters carefully before submitting.' }}</p>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <IssueCurrencyModal v-model:visible="showIssue" @success="onIssueSuccess" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Coin, Plus } from '@element-plus/icons-vue'
import IssueCurrencyModal from '@/components/modals/IssueCurrencyModal.vue'

const { t } = useI18n()

const showIssue = ref(false)

function onIssueSuccess() {
  ElMessage.success(t('monetary.issueSuccess') || t('common.operationSuccess'))
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.issue-currency-page {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 0;
    }
  }

  .card-title {
    margin: 0;
    font-size: 15px;
    color: $text-primary;
  }

  .intro-section {
    .mb-16 {
      margin-bottom: 16px;
    }

    .mr-2 {
      margin-right: 8px;
    }

    .type-desc {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 4px 0;
      font-size: $font-size-sm;
      color: $text-secondary;
    }

    .action-center {
      display: flex;
      justify-content: center;
      padding-top: 8px;
    }
  }

  .info-text {
    font-size: $font-size-sm;
    color: $text-secondary;
    line-height: 1.6;

    p {
      margin: 0;
    }

    .mt-8 {
      margin-top: 8px;
    }
  }
}
</style>
