<template>
  <div class="contract-detail-page">
    <el-card v-if="loading" v-loading="true" style="min-height: 400px;">
    </el-card>

    <template v-else-if="contract">
      <!-- 头部信息 -->
      <el-card shadow="never" class="header-card" :body-style="{ padding: '20px' }">
        <div class="contract-header">
          <div class="header-left">
            <h2 class="contract-name">{{ contract.name || '未命名合约' }}</h2>
            <div class="contract-address">
              <el-tooltip :content="contract.address" placement="top">
                <span class="address-text">{{ formatAddress(contract.address) }}</span>
              </el-tooltip>
              <el-button
                size="small"
                text
                @click="copyAddress(contract.address)"
              >
                <el-icon><CopyDocument /></el-icon>
              </el-button>
              <el-tag v-if="contract.is_verified" type="success" size="small" effect="plain">
                已验证
              </el-tag>
              <el-tag v-else type="warning" size="small" effect="plain">
                未验证
              </el-tag>
            </div>
            <div class="contract-meta">
              <span class="meta-item">
                <span class="label">部署者:</span>
                <el-link type="primary" @click="viewAccount(contract.deployer)">
                  {{ formatAddress(contract.deployer) }}
                </el-link>
              </span>
              <span class="meta-item">
                <span class="label">部署时间:</span>
                {{ formatDateTime(contract.deployed_at) }}
              </span>
            </div>
          </div>

          <div class="header-right">
            <el-button @click="goBack">
              <el-icon><ArrowLeft /></el-icon>
              返回
            </el-button>
            <el-button
              v-if="!contract.is_verified"
              type="primary"
              @click="showVerifyDialog = true"
            >
              <el-icon><Check /></el-icon>
              验证合约
            </el-button>
            <el-button
              type="success"
              @click="goToCallContract"
            >
              <el-icon><Position /></el-icon>
              调用合约
            </el-button>
          </div>
        </div>
      </el-card>

      <!-- tabs 内容 -->
      <el-tabs v-model="activeTab" class="detail-tabs" style="margin-top: 20px;">
        <!-- ABI -->
        <el-tab-pane label="ABI" name="abi">
          <el-card shadow="never">
            <template #header>
              <span>合约 ABI</span>
              <el-button
                size="small"
                type="primary"
                link
                @click="copyAbi"
                style="float: right;"
              >
                <el-icon><CopyDocument /></el-icon>
                复制
              </el-button>
            </template>
            <div class="abi-content">
              <pre v-if="contract.abi && contract.abi.length > 0">{{ formatAbi(contract.abi) }}</pre>
              <el-empty v-else description="暂无 ABI 数据" :image-size="100" />
            </div>
          </el-card>
        </el-tab-pane>

        <!-- 字节码 -->
        <el-tab-pane label="字节码" name="bytecode">
          <el-card shadow="never">
            <template #header>
              <span>部署字节码</span>
              <el-button
                size="small"
                type="primary"
                link
                @click="copyBytecode"
                style="float: right;"
              >
                <el-icon><CopyDocument /></el-icon>
                复制
              </el-button>
            </template>
            <div class="bytecode-content">
              <pre v-if="contract.bytecode">{{ formatBytecode(contract.bytecode) }}</pre>
              <el-empty v-else description="无字节码数据" :image-size="100" />
            </div>
          </el-card>
        </el-tab-pane>

        <!-- 源代码 -->
        <el-tab-pane label="源代码" name="source">
          <el-card shadow="never">
            <template #header>
              <span>源代码</span>
            </template>
            <div class="source-content">
              <pre v-if="contract.source_code">{{ contract.source_code }}</pre>
              <el-empty v-else description="未上传源代码或未验证" :image-size="100" />
            </div>
          </el-card>
        </el-tab-pane>

        <!-- 事件日志 -->
        <el-tab-pane label="事件日志" name="events">
          <el-card shadow="never">
            <template #header>
              <span>合约事件</span>
            </template>
            <EventLogs :contract-address="contract.address" />
          </el-card>
        </el-tab-pane>

        <!-- 存储状态 -->
        <el-tab-pane label="存储状态" name="storage">
          <el-card shadow="never">
            <template #header>
              <span>存储槽位</span>
            </template>
            <StorageViewer :contract-address="contract.address" />
          </el-card>
        </el-tab-pane>
      </el-tabs>
    </template>

    <!-- 合约不存在 -->
    <el-empty v-else description="合约不存在或已被删除" />

    <!-- 验证合约对话框 -->
    <BaseDialog
      v-model="showVerifyDialog"
      title="验证合约"
      width="600px"
      @confirm="handleVerify"
    >
      <el-form :model="verifyForm" label-width="120px">
        <el-form-item label="合约地址">
          <el-input v-model="verifyForm.address" disabled />
        </el-form-item>
        <el-form-item label="编译器版本">
          <el-select v-model="verifyForm.compilerVersion" placeholder="请选择编译器版本">
            <el-option label="Solidity 0.8.0" value="0.8.0" />
            <el-option label="Solidity 0.7.6" value="0.7.6" />
            <el-option label="Solidity 0.6.12" value="0.6.12" />
            <el-option label="Solidity 0.5.17" value="0.5.17" />
          </el-select>
        </el-form-item>
        <el-form-item label="优化">
          <el-switch v-model="verifyForm.optimizer" />
        </el-form-item>
        <el-form-item label="源代码">
          <el-input
            v-model="verifyForm.sourceCode"
            type="textarea"
            :rows="12"
            placeholder="请输入完整的源代码"
          />
        </el-form-item>
      </el-form>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { CopyDocument, ArrowLeft, Check, Position } from '@element-plus/icons-vue'
import { useContractStore } from '@/stores/modules/contract.store'
import { contractApi } from '@/api/modules'
import BaseDialog from '@/components/base/BaseDialog.vue'
import EventLogs from './EventLogs.vue'
import StorageViewer from './StorageViewer.vue'
import type { Contract } from '@/types/business'
import { formatAddress, formatTime } from '@/utils/format'

const router = useRouter()
const route = useRoute()
const contractStore = useContractStore()

const activeTab = ref('abi')
const loading = ref(false)
const contract = ref<Contract | null>(null)
const showVerifyDialog = ref(false)

const verifyForm = ref({
  address: '',
  compilerVersion: '',
  optimizer: false,
  sourceCode: ''
})

// 获取合约地址
const contractAddress = computed(() => route.params.address as string)

// 获取合约详情
const fetchContract = async () => {
  try {
    loading.value = true
    const response = await contractApi.getContractByAddress(contractAddress.value)
    contract.value = response.data

    // 初始化验证表单
    verifyForm.value.address = contractAddress.value
  } catch (error: any) {
    console.error('Failed to fetch contract:', error)
    ElMessage.error('获取合约信息失败')
    contract.value = null
  } finally {
    loading.value = false
  }
}

// 复制地址
const copyAddress = async (addr: string) => {
  try {
    await navigator.clipboard.writeText(addr)
    ElMessage.success('地址已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 复制 ABI
const copyAbi = async () => {
  if (!contract.value?.abi) return
  try {
    await navigator.clipboard.writeText(JSON.stringify(contract.value.abi, null, 2))
    ElMessage.success('ABI 已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 复制字节码
const copyBytecode = async () => {
  if (!contract.value?.bytecode) return
  try {
    await navigator.clipboard.writeText(contract.value.bytecode)
    ElMessage.success('字节码已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 导航
const goBack = () => {
  router.back()
}

const viewAccount = (address: string) => {
  router.push(`/account/detail/${address}`)
}

const goToCallContract = () => {
  router.push({
    path: '/contract/call',
    query: { address: contractAddress.value }
  })
}

// 验证合约
const handleVerify = async () => {
  try {
    await contractApi.verifyContract({
      address: verifyForm.value.address,
      sourceCode: verifyForm.value.sourceCode,
      compilerVersion: verifyForm.value.compilerVersion,
      optimizer: verifyForm.value.optimizer
    })

    ElMessage.success('合约验证提交成功')
    showVerifyDialog.value = false

    // 刷新合约信息
    await fetchContract()
  } catch (error: any) {
    console.error('Failed to verify contract:', error)
    ElMessage.error('合约验证失败：' + (error.message || '未知错误'))
  }
}

// 格式化方法
const formatAbi = (abi: any[]): string => {
  return JSON.stringify(abi, null, 2)
}

const formatBytecode = (bytecode: string): string => {
  // 每行显示 60 个字符
  const chunkSize = 60
  const chunks = []
  for (let i = 0; i < bytecode.length; i += chunkSize) {
    chunks.push(bytecode.slice(i, i + chunkSize))
  }
  return chunks.join('\n')
}

const formatAddress = (addr: string): string => {
  return formatAddress(addr, 12)
}

const formatDateTime = (dateStr: string): string => {
  return formatTime(dateStr, 'YYYY-MM-DD HH:mm:ss')
}

// 监听地址变化
watch(
  () => contractAddress.value,
  (newAddr) => {
    if (newAddr) {
      fetchContract()
    }
  },
  { immediate: true }
)
</script>

<script lang="ts">
import { watch } from 'vue'
</script>

<style scoped lang="scss">
.contract-detail-page {
  .header-card {
    .contract-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;

      .header-left {
        .contract-name {
          margin: 0 0 8px 0;
          font-size: 20px;
          color: #303133;
        }

        .contract-address {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 12px;

          .address-text {
            font-family: 'Roboto Mono', monospace;
            color: #409eff;
            cursor: pointer;
          }
        }

        .contract-meta {
          display: flex;
          gap: 24px;
          font-size: 13px;
          color: #606266;

          .meta-item {
            .label {
              color: #909399;
              margin-right: 4px;
            }
          }
        }
      }

      .header-right {
        display: flex;
        gap: 8px;
      }
    }
  }

  .detail-tabs {
    :deep(.el-tabs__content) {
      padding: 16px 0;
    }
  }

  .abi-content,
  .bytecode-content,
  .source-content {
    pre {
      margin: 0;
      padding: 16px;
      background: #f5f7fa;
      border-radius: 4px;
      overflow-x: auto;
      font-family: 'Roboto Mono', monospace;
      font-size: 13px;
      line-height: 1.5;
      max-height: 600px;
      overflow-y: auto;
    }
  }

  .source-content {
    pre {
      white-space: pre-wrap;
      word-break: break-all;
    }
  }
}
</style>
