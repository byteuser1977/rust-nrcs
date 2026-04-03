<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElCard, ElForm, ElFormItem, ElInput, ElButton, ElMessage } from 'element-plus'
import { useTransactionStore } from '@/stores/modules/transaction.store'
import { useAccountStore } from '@/stores/modules/account.store'
import { useRouter } from 'vue-router'
import { parseEthToWei } from '@/utils/crypto'

const { t } = useI18n()
const transactionStore = useTransactionStore()
const accountStore = useAccountStore()
const router = useRouter()

const formRef = ref()
const toAddress = ref('')
const amount = ref('')
const gasPrice = ref('')
const gasLimit = ref(21000)
const data = ref('')
const isSubmitting = ref(false)

async function handleSend() {
  if (!accountStore.walletAddress) {
    ElMessage.error('请先连接钱包')
    router.push('/wallet/connect')
    return
  }

  try {
    await formRef.value.validate()
    isSubmitting.value = true

    const txHash = await transactionStore.sendTransaction({
      from: accountStore.walletAddress,
      to: toAddress.value,
      value: parseEthToWei(amount.value).toString(),
      gasPrice: gasPrice.value || undefined,
      gasLimit: gasLimit.value || undefined,
      data: data.value || undefined
    })

    if (txHash) {
      ElMessage.success(t('transaction.sendSuccess'))
      router.push('/transaction/history')
    }
  } catch (error) {
    console.error('Send failed', error)
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="transaction-send">
    <h1 class="page-title">{{ t('transaction.send') }}</h1>

    <el-card shadow="never">
      <el-form ref="formRef" :model="{ toAddress, amount, gasPrice, gasLimit, data }" label-width="120px">
        <el-form-item :label="t('transaction.recipient')" prop="toAddress" required>
          <el-input v-model="toAddress" placeholder="0x..." />
        </el-form-item>

        <el-form-item :label="t('transaction.amount')" prop="amount" required>
          <el-input v-model="amount" type="number" step="0.0001" min="0" placeholder="0.0">
            <template #append>ETH</template>
          </el-input>
        </el-form-item>

        <el-form-item :label="t('transaction.gasPrice')">
          <el-input v-model="gasPrice" placeholder="Auto" />
        </el-form-item>

        <el-form-item :label="t('transaction.gasUsed')">
          <el-input v-model.number="gasLimit" type="number" :min="21000" />
        </el-form-item>

        <el-form-item :label="t('transaction.data')">
          <el-input v-model="data" type="textarea" :rows="4" placeholder="0x..." />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" :loading="isSubmitting" @click="handleSend">
            {{ t('transaction.send') }}
          </el-button>
          <el-button @click="router.back()">{{ t('common.cancel') }}</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<style lang="scss" scoped>
.transaction-send {
  .page-title {
    margin-bottom: 24px;
    font-size: 24px;
    font-weight: 600;
  }
}
</style>
