<template>
  <div class="send-transaction-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>发送交易</span>
          <el-button text type="primary" @click="goBack">
            返回
          </el-button>
        </div>
      </template>

      <BaseForm
        ref="formRef"
        v-model="formData"
        :rules="rules"
        :fields="fields"
        label-width="120px"
        @submit="handleSubmit"
      >
        <!-- 余额显示 -->
        <el-form-item label="可用余额">
          <div class="balance-display">
            <span class="balance-value">{{ formattedBalance }} NRCS</span>
            <el-button
              size="small"
              type="primary"
              link
              @click="useMaxBalance"
              v-if="balance > 0"
            >
              全部使用
            </el-button>
          </div>
        </el-form-item>

        <!-- 费用预览 -->
        <el-form-item label="费用预览">
          <div class="fee-preview">
            <span class="fee-label">估计Gas消耗:</span>
            <span class="fee-value">{{ estimatedGas ? estimatedGas.gas_limit.toLocaleString() : '-' }}</span>
            <span class="fee-label" style="margin-left: 12px;">Gas价格:</span>
            <span class="fee-value">{{ estimatedGas ? formatGasPrice(estimatedGas.gas_price) : '-' }}</span>
            <span class="fee-label" style="margin-left: 12px;">总费用:</span>
            <span class="fee-value total">{{ estimatedGas ? formatTotalFee(estimatedGas) : '-' }}</span>
          </div>
        </el-form-item>

        <template #actions>
          <el-button @click="resetForm">重置</el-button>
        </template>
      </BaseForm>
    </el-card>

    <!-- 交易确认对话框 -->
    <BaseDialog
      v-model="showConfirmDialog"
      title="确认交易"
      width="500px"
      :confirm-loading="confirmLoading"
      @confirm="confirmSend"
    >
      <div class="confirm-dialog">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="发送方">
            {{ formData.to }}
          </el-descriptions-item>
          <el-descriptions-item label="接收方">
            {{ formData.from }}
          </el-descriptions-item>
          <el-descriptions-item label="金额">
            <span class="amount-value">{{ formData.value }} NRCS</span>
          </el-descriptions-item>
          <el-descriptions-item label="Gas限制">
            {{ formData.gasLimit }}
          </el-descriptions-item>
          <el-descriptions-item label="Gas价格">
            {{ formData.gasPrice }} Gwei
          </el-descriptions-item>
          <el-descriptions-item label="总费用">
            {{ calculateTotalFee() }}
          </el-descriptions-item>
        </el-descriptions>

        <el-alert
          title="重要提示"
          type="warning"
          :closable="false"
          style="margin-top: 16px;"
        >
          <template #default>
            <p>确认交易信息无误后，交易将被广播到网络。</p>
            <p>交易一旦提交，无法撤销。</p>
          </template>
        </el-alert>
      </div>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { useTransactionStore } from '@/stores/modules/transaction.store'
import { transactionApi } from '@/api/modules'
import BaseForm from '@/components/base/BaseForm.vue'
import BaseDialog from '@/components/base/BaseDialog.vue'
import { formatWeiToEth, formatWei } from '@/utils/format'

const router = useRouter()
const route = useRoute()
const accountStore = useAccountStore()
const transactionStore = useTransactionStore()

const formRef = ref()
const loading = ref(false)
const confirmLoading = ref(false)
const showConfirmDialog = ref(false)
const balance = ref('0')
const estimatedGas = ref<{ gas_limit: number; gas_price: string } | null>(null)

// 预填接收地址（如果有）
const initialTo = route.query.to as string || ''

const formData = reactive({
  from: '',
  to: initialTo,
  value: '',
  gasPrice: 20,
  gasLimit: 21000,
  data: ''
})

const fields = [
  {
    prop: 'from',
    label: '发送方地址',
    type: 'input' as const,
    props: {
      placeholder: '0x...',
      disabled: true
    },
    required: true
  },
  {
    prop: 'to',
    label: '接收方地址',
    type: 'input' as const,
    props: {
      placeholder: '0x...'
    },
    required: true
  },
  {
    prop: 'value',
    label: '金额 (NRCS)',
    type: 'input' as const,
    props: {
      placeholder: '0.0'
    },
    required: true
  },
  {
    prop: 'gasPrice',
    label: 'Gas价格 (Gwei)',
    type: 'number' as const,
    props: {
      min: 1,
      max: 1000
    },
    required: true
  },
  {
    prop: 'gasLimit',
    label: 'Gas限制',
    type: 'number' as const,
    props: {
      min: 21000,
      max: 5000000
    },
    required: true
  },
  {
    prop: 'data',
    label: '数据 (可选)',
    type: 'textarea' as const,
    props: {
      rows: 3,
      placeholder: '0x...'
    },
    required: false
  }
]

const rules = {
  from: [
    { required: true, message: '请输入发送方地址', trigger: 'blur' },
    {
      pattern: /^0x[a-fA-F0-9]{40}$/,
      message: '请输入有效的地址格式',
      trigger: 'blur'
    }
  ],
  to: [
    { required: true, message: '请输入接收方地址', trigger: 'blur' },
    {
      pattern: /^0x[a-fA-F0-9]{40}$/,
      message: '请输入有效的地址格式',
      trigger: 'blur'
    }
  ],
  value: [
    { required: true, message: '请输入金额', trigger: 'blur' },
    {
      validator: (rule: any, value: string, callback: Function) => {
        if (!value || parseFloat(value) <= 0) {
          callback(new Error('金额必须大于0'))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ],
  gasPrice: [
    { required: true, message: '请输入Gas价格', trigger: 'blur' }
  ],
  gasLimit: [
    { required: true, message: '请输入Gas限制', trigger: 'blur' }
  ]
}

// 计算属性
const formattedBalance = computed(() => {
  const eth = formatWeiToEth(balance.value)
  return parseFloat(eth).toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 6
  })
})

// 监听金额变化，重新估算Gas
watch(() => formData.value, async (newVal) => {
  if (newVal.from && newVal.to && newVal.value) {
    await estimateGas()
  }
}, { deep: true })

// 获取账户余额
const fetchBalance = async () => {
  try {
    if (!accountStore.walletAddress) return
    const response = await accountApi.getAccountBalance(accountStore.walletAddress)
    balance.value = response.data.balance
  } catch (error) {
    console.error('Failed to fetch balance:', error)
  }
}

// 估算Gas
const estimateGas = async () => {
  try {
    const response = await transactionApi.estimateGas({
      from: formData.from,
      to: formData.to,
      value: weiFromEth(formData.value),
      data: formData.data || undefined
    })
    estimatedGas.value = response.data
  } catch (error) {
    console.error('Failed to estimate gas:', error)
    // 使用默认值
    estimatedGas.value = {
      gas_limit: formData.gasLimit,
      gas_price: weiFromGwei(formData.gasPrice)
    }
  }
}

// 发送交易
const handleSubmit = async () => {
  if (!formRef.value) return
  const isValid = await formRef.value.validate()
  if (!isValid) return

  showConfirmDialog.value = true
}

const confirmSend = async () => {
  try {
    confirmLoading.value = true

    const response = await transactionApi.sendTransaction({
      from: formData.from,
      to: formData.to,
      value: weiFromEth(formData.value),
      gasPrice: weiFromGwei(formData.gasPrice),
      gasLimit: formData.gasLimit,
      data: formData.data || undefined
    })

    const txHash = response.data.tx_hash
    ElMessage.success('交易发送成功')

    // 跳转到详情页
    router.push(`/transaction/detail/${txHash}`)
  } catch (error: any) {
    console.error('Failed to send transaction:', error)
    ElMessage.error('交易发送失败：' + (error.message || '未知错误'))
    throw error
  } finally {
    confirmLoading.value = false
    showConfirmDialog.value = false
  }
}

// 重置表单
const resetForm = () => {
  formData.from = accountStore.walletAddress || ''
  formData.to = ''
  formData.value = ''
  formData.data = ''
  formData.gasPrice = 20
  formData.gasLimit = 21000
  estimatedGas.value = null
  if (formRef.value) {
    formRef.value.clearValidate()
  }
}

// 使用最大余额
const useMaxBalance = () => {
  // TODO: 计算最大可发送金额（考虑Gas费用）
  const maxAmount = parseFloat(formatWeiToEth(balance.value))
  formData.value = maxAmount.toFixed(6)
}

// 计算总费用
const calculateTotalFee = (): string => {
  if (!estimatedGas.value) return '-'
  const totalWei = BigInt(estimatedGas.value.gas_limit) * BigInt(estimatedGas.value.gas_price)
  return formatWei(totalWei.toString())
}

// 格式化Gas价格
const formatGasPrice = (wei: string): string => {
  const gwei = BigInt(wei) / BigInt(1e9)
  return `${gwei.toLocaleString()} Gwei`
}

// 格式化总费用（转换为NRCS）
const formatTotalFee = (gas: { gas_limit: number; gas_price: string }): string => {
  const totalWei = BigInt(gas.gas_limit) * BigInt(gas.gas_price)
  const nrsc = formatWeiToEth(totalWei.toString())
  return `${parseFloat(nrsc).toFixed(6)} NRCS`
}

// wei转换工具
const weiFromEth = (eth: string): string => {
  const wei = BigInt(Math.floor(parseFloat(eth) * 1e18))
  return wei.toString()
}

const weiFromGwei = (gwei: number): string => {
  const wei = BigInt(Math.floor(gwei * 1e9))
  return wei.toString()
}

const goBack = () => {
  router.back()
}

onMounted(() => {
  if (accountStore.walletAddress) {
    formData.from = accountStore.walletAddress
  }
  fetchBalance()
})
</script>

<style lang="scss" scoped>
.send-transaction-page {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .balance-display {
    display: flex;
    align-items: center;
    gap: 12px;

    .balance-value {
      font-family: 'Roboto Mono', monospace;
      font-size: 18px;
      font-weight: 600;
      color: #303133;
    }
  }

  .fee-preview {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    padding: 12px;
    background: #f5f7fa;
    border-radius: 4px;

    .fee-label {
      font-size: 13px;
      color: #909399;
    }

    .fee-value {
      font-family: 'Roboto Mono', monospace;
      font-weight: 500;
      color: #303133;

      &.total {
        color: #f56c6c;
        font-weight: 600;
      }
    }
  }

  .confirm-dialog {
    .amount-value {
      font-family: 'Roboto Mono', monospace;
      font-size: 16px;
      font-weight: 600;
    }
  }
}
</style>
