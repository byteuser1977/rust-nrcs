<template>
  <div class="transaction-detail-page">
    <el-card v-if="loading" v-loading="true" style="min-height: 400px;">
    </el-card>

    <template v-else-if="transaction">
      <!-- 状态头部 -->
      <el-card shadow="never" class="status-card" :body-style="{ padding: '20px' }">
        <div class="status-header">
          <div class="status-info">
            <StatusBadge :value="transaction.status" :size="'large'" />
            <span class="status-text">{{ statusText }}</span>
          </div>
          <div class="action-buttons">
            <el-button @click="goBack">
              <el-icon><ArrowLeft /></el-icon>
              返回
            </el-button>
            <el-button
              v-if="transaction.receipt"
              type="primary"
              @click="viewInExplorer"
            >
              <el-icon><Monitor /></el-icon>
              在浏览器中查看
            </el-button>
          </div>
        </div>
      </el-card>

      <!-- 基本信息 -->
      <el-row :gutter="20" style="margin-top: 20px;">
        <el-col :xs="24" :sm="24" :md="12" :lg="8">
          <el-card shadow="never">
            <template #header>
              <span>交易信息</span>
            </template>
            <el-descriptions :column="1" border>
              <el-descriptions-item label="交易哈希">
                <el-tooltip :content="transaction.hash" placement="top">
                  <span class="hash-text">{{ formatHash(transaction.hash) }}</span>
                </el-tooltip>
                <el-button
                  size="small"
                  text
                  @click="copyText(transaction.hash)"
                >
                  <el-icon><CopyDocument /></el-icon>
                </el-button>
              </el-descriptions-item>
              <el-descriptions-item label="状态">
                <StatusBadge :value="transaction.status" />
              </el-descriptions-item>
              <el-descriptions-item label="区块高度">
                {{ transaction.block_number }}
                <el-link
                  v-if="transaction.block_number > 0"
                  type="primary"
                  style="margin-left: 8px;"
                  @click="goToBlock(transaction.block_number)"
                >
                  查看区块
                </el-link>
              </el-descriptions-item>
              <el-descriptions-item label="时间戳">
                {{ formatDateTime(transaction.created_at) }}
              </el-descriptions-item>
              <el-descriptions-item label="确认数">
                {{ confirmationCount }}
              </el-descriptions-item>
            </el-descriptions>
          </el-card>
        </el-col>

        <el-col :xs="24" :sm="24" :md="12" :lg="8">
          <el-card shadow="never">
            <template #header>
              <span>发送方</span>
            </template>
            <div class="address-info">
              <span class="address-label">From:</span>
              <span class="address-value">{{ formatAddress(transaction.from) }}</span>
              <el-button
                size="small"
                text
                @click="copyText(transaction.from)"
              >
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </div>
          </el-card>
        </el-col>

        <el-col :xs="24" :sm="24" :md="12" :lg="8">
          <el-card shadow="never">
            <template #header>
              <span>接收方</span>
            </template>
            <div class="address-info">
              <span class="address-label">To:</span>
              <span class="address-value">
                {{ transaction.to ? formatAddress(transaction.to) : '(合约创建)' }}
              </span>
              <el-button
                v-if="transaction.to"
                size="small"
                text
                @click="copyText(transaction.to)"
              >
                <el-icon><CopyDocument /></el-icon>
              </el-button>
              <el-link
                v-if="transaction.to"
                type="primary"
                style="margin-left: 8px;"
                @click="goToContract(transaction.to)"
              >
                查看合约
              </el-link>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 交易详情 -->
      <el-card shadow="never" style="margin-top: 20px;">
        <template #header>
          <span>交易详情</span>
        </template>

        <el-descriptions :column="2" border>
          <el-descriptions-item label="价值">
            <span class="value-amount">{{ formatValue(transaction.value) }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="Gas 限制">
            {{ transaction.gas_limit }}
          </el-descriptions-item>
          <el-descriptions-item label="Gas 价格">
            {{ formatGasPrice(transaction.gas_price) }}
          </el-descriptions-item>
          <el-descriptions-item label="Nonce">
            {{ transaction.nonce }}
          </el-descriptions-item>
          <el-descriptions-item label="Gas 消耗" :span="2">
            {{ transaction.gas_used }} / {{ transaction.gas_limit }}
            <el-progress
              :percentage="gasUsagePercentage"
              :stroke-width="8"
              style="width: 200px; margin-left: 16px;"
            />
          </el-descriptions-item>
        </el-descriptions>

        <!-- 输入数据 -->
        <el-divider content-position="left">输入数据</el-divider>
        <div class="input-data-section">
          <div v-if="transaction.input && transaction.input !== '0x'" class="has-data">
            <el-tabs v-model="inputTab">
              <el-tab-pane label="Hex" name="hex">
                <pre class="code-block">{{ transaction.input }}</pre>
              </el-tab-pane>
              <el-tab-pane label="Decoded" name="decoded">
                <div v-if="decodedInput" class="decoded-data">
                  <pre>{{ JSON.stringify(decodedInput, null, 2) }}</pre>
                </div>
                <el-empty v-else description="无法解析输入数据" :image-size="80" />
              </el-tab-pane>
            </el-tabs>
          </div>
          <el-empty v-else description="无输入数据" :image-size="80" />
        </div>
      </el-card>

      <!-- 交易收据 -->
      <el-card shadow="never" style="margin-top: 20px;" v-if="transaction.receipt">
        <template #header>
          <span>交易收据</span>
        </template>

        <el-descriptions :column="2" border>
          <el-descriptions-item label="交易索引">
            {{ transaction.receipt.transaction_index }}
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <StatusBadge :value="transaction.receipt.status" />
          </el-descriptions-item>
          <el-descriptions-item label="Root">
            {{ transaction.receipt.root || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="Cumulative Gas Used">
            {{ transaction.receipt.cumulative_gas_used }}
          </el-descriptions-item>
          <el-descriptions-item v-if="transaction.receipt.contract_address" label="合约地址" :span="2">
            {{ transaction.receipt.contract_address }}
          </el-descriptions-item>
          <el-descriptions-item v-if="transaction.receipt.logs" label="日志数量" :span="2">
            {{ transaction.receipt.logs.length }}
          </el-descriptions-item>
        </el-descriptions>

        <!-- 事件日志 -->
        <div v-if="transaction.receipt.logs && transaction.receipt.logs.length > 0" style="margin-top: 20px;">
          <el-divider content-position="left">事件日志</el-divider>
          <el-table :data="transaction.receipt.logs" style="width: 100%">
            <el-table-column prop="index" label="日志索引" width="100" />
            <el-table-column prop="address" label="合约地址" min-width="200">
              <template #default="{ row }">
                {{ formatAddress(row.address) }}
              </template>
            </el-table-column>
            <el-table-column prop="event" label="事件名称" width="150" />
            <el-table-column prop="data" label="数据" min-width="300">
              <template #default="{ row }">
                <pre class="log-data">{{ row.data }}</pre>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-card>
    </template>

    <!-- 交易不存在 -->
    <el-empty v-else description="交易不存在或已被删除" />

    <!-- 返回顶部 -->
    <el-backtop target=".el-main" :right="50" :bottom="50" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { ArrowLeft, CopyDocument, Monitor } from '@element-plus/icons-vue'
import { useTransactionStore } from '@/stores/modules/transaction.store'
import { transactionApi } from '@/api/modules'
import StatusBadge from '@/components/base/StatusBadge.vue'
import type { Transaction } from '@/types/business'
import { formatAddress, formatTime } from '@/utils/format'

const router = useRoute()
const transactionStore = useTransactionStore()

const loading = ref(false)
const transaction = ref<Transaction | null>(null)
const inputTab = ref('hex')

// 从路由获取交易哈希
const txHash = computed(() => router.params.hash as string)

// 计算状态文本
const statusText = computed(() => {
  if (!transaction.value) return ''
  switch (transaction.value.status) {
    case 'success': return '交易成功'
    case 'failed': return '交易失败'
    case 'pending': return '等待确认'
    case 'confirming': return '确认中'
    default: return transaction.value.status
  }
})

// 确认数（模拟）
const confirmationCount = computed(() => {
  if (!transaction.value) return 0
  return transaction.value.block_number > 0 ? 1 : 0
})

// Gas使用百分比
const gasUsagePercentage = computed(() => {
  if (!transaction.value) return 0
  const { gas_used, gas_limit } = transaction.value
  return Math.round((gas_used / gas_limit) * 100)
})

// 解码的输入数据（模拟）
const decodedInput = ref<any>(null)

// 获取交易详情
const fetchTransaction = async () => {
  try {
    loading.value = true
    const response = await transactionApi.getTransactionByHash(txHash.value)
    transaction.value = response.data

    // 尝试解码输入数据
    if (transaction.value.input && transaction.value.input !== '0x') {
      decodedInput.value = decodeInput(transaction.value.input)
    }
  } catch (error: any) {
    console.error('Failed to fetch transaction:', error)
    ElMessage.error('获取交易详情失败')
  } finally {
    loading.value = false
  }
}

// 解码输入数据（模拟）
const decodeInput = (input: string): any => {
  // TODO: 根据合约ABI解码
  try {
    // 简单的解析示例
    if (input.startsWith('0xa9059cbb')) {
      return {
        method: 'transfer(address,uint256)',
        params: {
          to: '0x' + input.slice(10, 74),
          amount: parseInt(input.slice(74), 16)
        }
      }
    }
    return { raw: input }
  } catch {
    return null
  }
}

// 复制文本
const copyText = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 导航
const goBack = () => {
  router.back()
}

const goToBlock = (blockNumber: number) => {
  router.push(`/node/blocks/${blockNumber}`)
}

const goToContract = (address: string) => {
  router.push(`/contract/detail/${address}`)
}

const viewInExplorer = () => {
  // TODO: 根据链ID构造浏览器URL
  const url = `https://explorer.nrcs.io/tx/${txHash.value}`
  window.open(url, '_blank')
}

// 格式化方法
const formatHash = (hash: string): string => {
  return `${hash.slice(0, 12)}...${hash.slice(-8)}`
}

const formatAddress = (addr: string): string => {
  return formatAddress(addr, 8)
}

const formatDateTime = (dateStr: string): string => {
  return formatTime(dateStr, 'YYYY-MM-DD HH:mm:ss')
}

const formatValue = (wei: string): string => {
  const eth = parseFloat(formatWeiToEth(wei))
  return `${eth.toLocaleString('en-US', { minimumFractionDigits: 4, maximumFractionDigits: 6 })} NRCS`
}

const formatGasPrice = (price: string): string => {
  const wei = BigInt(price)
  const gwei = wei / BigInt(1e9)
  return `${gwei.toLocaleString()} Gwei`
}

// 监听哈希变化
watch(
  () => txHash.value,
  (newHash) => {
    if (newHash) {
      fetchTransaction()
    }
  },
  { immediate: true }
)
</script>

<script lang="ts">
import { watch } from 'vue'
import { formatWeiToEth } from '@/utils/format'
</script>

<style lang="scss" scoped>
.transaction-detail-page {
  .status-card {
    .status-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .status-info {
        display: flex;
        align-items: center;
        gap: 12px;

        .status-text {
          font-size: 18px;
          font-weight: 500;
        }
      }
    }
  }

  .address-info {
    display: flex;
    align-items: center;
    gap: 8px;

    .address-label {
      color: #909399;
      font-size: 13px;
    }

    .address-value {
      font-family: 'Roboto Mono', monospace;
      font-size: 14px;
      color: #303133;
    }
  }

  .hash-text {
    font-family: 'Roboto Mono', monospace;
    color: #409eff;
  }

  .value-amount {
    font-family: 'Roboto Mono', monospace;
    font-size: 18px;
    font-weight: 600;
    color: #303133;
  }

  .input-data-section {
    .has-data {
      .code-block {
        background: #f5f7fa;
        padding: 12px;
        border-radius: 4px;
        overflow-x: auto;
        font-family: 'Roboto Mono', monospace;
        font-size: 12px;
        margin: 0;
      }

      .decoded-data {
        background: #f5f7fa;
        padding: 12px;
        border-radius: 4px;

        pre {
          margin: 0;
          font-size: 13px;
          white-space: pre-wrap;
          word-wrap: break-word;
        }
      }
    }
  }

  .log-data {
    margin: 0;
    font-size: 12px;
    font-family: 'Roboto Mono', monospace;
    white-space: pre-wrap;
    word-wrap: break-word;
  }
}
</style>
