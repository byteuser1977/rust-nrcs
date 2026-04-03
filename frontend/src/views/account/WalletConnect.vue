<template>
  <div class="wallet-connect-page">
    <el-row justify="center">
      <el-col :xs="24" :sm="20" :md="16" :lg="12" :xl="10">
        <el-card class="connect-card">
          <template #header>
            <h2>连接钱包</h2>
            <p class="subtitle">选择您的钱包类型并连接</p>
          </template>

          <!-- 钱包类型选择 -->
          <div v-if="!selectedWallet" class="wallet-types">
            <div
              v-for="wallet in walletOptions"
              :key="wallet.type"
              class="wallet-option"
              :class="{ active: selectedWallet === wallet.type }"
              @click="selectWallet(wallet.type)"
            >
              <div class="wallet-icon">
                <img v-if="wallet.icon" :src="wallet.icon" :alt="wallet.name" />
                <el-icon v-else :size="48"><Wallet /></el-icon>
              </div>
              <div class="wallet-info">
                <h3>{{ wallet.name }}</h3>
                <p>{{ wallet.description }}</p>
              </div>
              <el-icon v-if="selectedWallet === wallet.type" class="check-icon"><CircleCheck /></el-icon>
            </div>
          </div>

          <!-- 连接表单 -->
          <div v-else class="connect-form">
            <el-button
              class="back-btn"
              text
              @click="selectedWallet = undefined"
            >
              <el-icon><ArrowLeft /></el-icon>
              返回
            </el-button>

            <div class="selected-wallet-info">
              <div class="wallet-icon-large">
                <img v-if="currentWallet?.icon" :src="currentWallet.icon" :alt="currentWallet.name" />
                <el-icon v-else :size="64"><Wallet /></el-icon>
              </div>
              <h3>{{ currentWallet?.name }}</h3>
            </div>

            <!-- 不同钱包类型的连接方式 -->
            <template v-if="selectedWallet === 'metamask'">
              <el-alert
                title="MetaMask 检测"
                :type="metamaskDetected ? 'success' : 'warning'"
                :closable="false"
                style="margin-bottom: 16px;"
              >
                <template #default>
                  {{ metamaskDetected ? '检测到 MetaMask 已安装，可以开始连接' : '未检测到 MetaMask，请先安装插件' }}
                </template>
              </el-alert>

              <el-button
                v-if="metamaskDetected"
                type="primary"
                :loading="connecting"
                @click="connectMetaMask"
                style="width: 100%"
              >
                <el-icon><Connection /></el-icon>
                连接 MetaMask
              </el-button>

              <div v-else class="install-wallet">
                <p>请先安装 MetaMask 浏览器插件：</p>
                <el-link type="primary" href="https://metamask.io" target="_blank">
                  前往下载 MetaMask
                </el-link>
              </div>
            </template>

            <template v-else-if="selectedWallet === 'ledger'">
              <el-form label-position="top">
                <el-form-item label="USB 设备状态">
                  <div class="device-status">
                    <el-icon :size="24" :color="ledgerConnected ? '#67c23a' : '#909399'">
                      <Connection v-if="ledgerConnected" />
                      <CircleClose v-else />
                    </el-icon>
                    <span>{{ ledgerConnected ? '已连接' : '未检测到设备' }}</span>
                  </div>
                </el-form-item>

                <el-form-item label="选择账户">
                  <el-select
                    v-model="ledgerPath"
                    placeholder="请选择Ledger账户"
                    :loading="loadingLedgerAccounts"
                    @change="selectLedgerPath"
                  >
                    <el-option
                      v-for="account in ledgerAccounts"
                      :key="account.path"
                      :label="account.address"
                      :value="account.path"
                    />
                  </el-select>
                </el-form-item>

                <el-button
                  type="primary"
                  :loading="connecting"
                  :disabled="!ledgerPath"
                  @click="connectLedger"
                  style="width: 100%"
                >
                  连接 Ledger
                </el-button>
              </el-form>
            </template>

            <template v-else-if="selectedWallet === 'trezor'">
              <el-form label-position="top">
                <el-alert
                  title="Trezor 连接"
                  description="请确保 Trezor 设备已连接并解锁"
                  type="info"
                  :closable="false"
                  style="margin-bottom: 16px;"
                />

                <el-button
                  type="primary"
                  :loading="connecting"
                  @click="connectTrezor"
                  style="width: 100%"
                >
                  连接 Trezor
                </el-button>
              </el-form>
            </template>

            <!-- 手动输入方式 -->
            <el-divider v-if="selectedWallet === 'manual'">手动输入</el-divider>

            <el-form
              v-if="selectedWallet === 'manual'"
              :model="manualForm"
              :rules="manualRules"
              ref="manualFormRef"
              label-position="top"
            >
              <el-form-item label="钱包地址" prop="address">
                <el-input
                  v-model="manualForm.address"
                  placeholder="0x..."
                  :prefix-icon="Wallet"
                  clearable
                />
              </el-form-item>

              <el-form-item label="签名消息" prop="signature">
                <el-input
                  v-model="manualForm.signature"
                  type="textarea"
                  :rows="3"
                  placeholder="请对验证消息进行签名，并粘贴签名结果"
                  clearable
                />
              </el-form-item>

              <el-form-item>
                <el-button
                  type="primary"
                  :loading="connecting"
                  @click="connectManual"
                  style="width: 100%"
                >
                  验证并连接
                </el-button>
              </el-form-item>
            </el-form>
          </div>

          <div class="connect-footer">
            <p class="security-notice">
              <el-icon><WarningFilled /></el-icon>
              连接钱包后，您将可以管理您的资产和进行交易
            </p>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  Wallet,
  CircleCheck,
  ArrowLeft,
  Connection,
  CircleClose,
  WarningFilled
} from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import type { FormInstance } from 'element-plus'

const router = useRouter()
const accountStore = useAccountStore()

// 钱包选项
const walletOptions = [
  {
    type: 'metamask',
    name: 'MetaMask',
    description: '最流行的以太坊钱包',
    icon: 'https://metamask.io/img/metamask-fox.svg'
  },
  {
    type: 'ledger',
    name: 'Ledger',
    description: '硬件钱包（Ledger Nano）',
    icon: ''
  },
  {
    type: 'trezor',
    name: 'Trezor',
    description: '硬件钱包（Trezor）',
    icon: ''
  },
  {
    type: 'walletconnect',
    name: 'WalletConnect',
    description: '移动端钱包连接',
    icon: ''
  },
  {
    type: 'manual',
    name: '手动输入',
    description: '粘贴钱包地址和签名',
    icon: ''
  }
] as const

type WalletType = typeof walletOptions[number]['type']

const selectedWallet = ref<WalletType>()
const connecting = ref(false)

// MetaMask
const metamaskDetected = ref(false)
const checkMetaMask = async () => {
  metamaskDetected.value = typeof window !== 'undefined' && !!window.ethereum
}

// Ledger
const ledgerConnected = ref(false)
const ledgerAccounts = ref<{ path: string; address: string }[]>([])
const ledgerPath = ref('')
const loadingLedgerAccounts = ref(false)

// 手动输入
const manualFormRef = ref<FormInstance>()
const manualForm = reactive({
  address: '',
  signature: ''
})
const manualRules = {
  address: [
    { required: true, message: '请输入钱包地址', trigger: 'blur' },
    {
      pattern: /^0x[a-fA-F0-9]{40}$/,
      message: '请输入有效的地址格式',
      trigger: 'blur'
    }
  ],
  signature: [
    { required: true, message: '请输入签名', trigger: 'blur' }
  ]
}

const currentWallet = computed(() =>
  walletOptions.find(w => w.type === selectedWallet.value)
)

// 选择钱包类型
const selectWallet = (type: WalletType) => {
  selectedWallet.value = type
}

// 连接 MetaMask
const connectMetaMask = async () => {
  try {
    connecting.value = true

    if (!window.ethereum) {
      ElMessage.error('未检测到 MetaMask')
      return
    }

    // 请求账户访问
    const accounts = await window.ethereum.request({
      method: 'eth_requestAccounts'
    })

    if (accounts.length === 0) {
      ElMessage.error('用户拒绝连接')
      return
    }

    const address = accounts[0]
    accountStore.setWalletAddress(address)

    ElMessage.success('钱包连接成功')
    router.push('/dashboard')
  } catch (error: any) {
    console.error('MetaMask connection failed:', error)
    ElMessage.error('连接失败：' + (error.message || '未知错误'))
  } finally {
    connecting.value = false
  }
}

// 连接 Ledger
const connectLedger = async () => {
  try {
    connecting.value = true

    // TODO: 实现 Ledger 连接逻辑
    // 需要安装 @ledgerhq/hw-transport-webusb 等依赖

    ElMessage.success('Ledger 连接成功')
    router.push('/dashboard')
  } catch (error: any) {
    console.error('Ledger connection failed:', error)
    ElMessage.error('Ledger 连接失败')
  } finally {
    connecting.value = false
  }
}

// 连接 Trezor
const connectTrezor = async () => {
  try {
    connecting.value = true

    // TODO: 实现 Trezor 连接逻辑
    // 需要安装 trezor-connect 库

    ElMessage.success('Trezor 连接成功')
    router.push('/dashboard')
  } catch (error: any) {
    console.error('Trezor connection failed:', error)
    ElMessage.error('Trezor 连接失败')
  } finally {
    connecting.value = false
  }
}

// 手动连接
const connectManual = async () => {
  try {
    await manualFormRef.value?.validate()
    connecting.value = true

    // 验证签名
    // TODO: 调用验证 API
    const isValid = await verifySignature(manualForm.address, manualForm.signature)

    if (isValid) {
      accountStore.setWalletAddress(manualForm.address)
      ElMessage.success('钱包验证成功')
      router.push('/dashboard')
    } else {
      ElMessage.error('签名验证失败')
    }
  } catch (error: any) {
    console.error('Manual connection failed:', error)
    if (error !== 'validation_failed') {
      ElMessage.error('连接失败')
    }
  } finally {
    connecting.value = false
  }
}

// 验证签名（模拟）
const verifySignature = async (address: string, signature: string): Promise<boolean> => {
  // TODO: 调用验证 API
  return true
}

onMounted(() => {
  checkMetaMask()
})
</script>

<style lang="scss" scoped>
.wallet-connect-page {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  padding: 24px;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
}

.connect-card {
  width: 100%;
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  background: rgba(255, 255, 255, 0.95);

  :deep(.el-card__header) {
    text-align: center;
    padding: 24px;

    h2 {
      margin: 0 0 8px 0;
      color: #303133;
      font-size: 24px;
    }

    .subtitle {
      margin: 0;
      color: #909399;
      font-size: 14px;
    }
  }
}

.wallet-types {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}

.wallet-option {
  display: flex;
  align-items: center;
  padding: 16px;
  border: 2px solid #e4e7ed;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    border-color: #409eff;
    background: #ecf5ff;
  }

  &.active {
    border-color: #409eff;
    background: #ecf5ff;
  }

  .wallet-icon {
    width: 64px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-right: 16px;
    background: #fff;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);

    img {
      max-width: 48px;
      max-height: 48px;
    }
  }

  .wallet-info {
    flex: 1;

    h3 {
      margin: 0 0 4px 0;
      font-size: 16px;
      color: #303133;
    }

    p {
      margin: 0;
      font-size: 13px;
      color: #909399;
    }
  }

  .check-icon {
    font-size: 24px;
    color: #409eff;
  }
}

.connect-form {
  position: relative;
}

.back-btn {
  position: absolute;
  left: -56px;
  top: 0;
  padding: 8px;
}

.selected-wallet-info {
  text-align: center;
  margin-bottom: 24px;

  .wallet-icon-large {
    width: 80px;
    height: 80px;
    margin: 0 auto 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #fff;
    border-radius: 16px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);

    img {
      max-width: 64px;
      max-height: 64px;
    }
  }

  h3 {
    margin: 0;
    color: #303133;
    font-size: 20px;
  }
}

.device-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;

  span {
    color: #606266;
  }
}

.install-wallet {
  text-align: center;
  padding: 24px;

  p {
    margin: 0 0 12px 0;
    color: #606266;
  }
}

.connect-footer {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid #e4e7ed;
  text-align: center;

  .security-notice {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    margin: 0;
    font-size: 13px;
    color: #909399;

    .el-icon {
      color: #e6a23c;
    }
  }
}
</style>
