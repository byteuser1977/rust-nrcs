<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { useUiStore } from '@/stores/modules/ui.store'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const accountStore = useAccountStore()
const uiStore = useUiStore()

// 表单数据
const walletAddress = ref('')
const isConnecting = ref(false)

// 计算属性
const redirectPath = computed(() => (route.query.redirect as string) || '/')

// 模拟连接钱包（实际项目中需要集成 Web3 库）
async function handleConnectWallet() {
  if (isConnecting.value) return

  isConnecting.value = true

  try {
    // TODO: 实际应集成 ethers.js / web3.js
    // const accounts = await ethereum.request({ method: 'eth_requestAccounts' })
    // const address = accounts[0]
    // const signature = await signMessage(address)

    // 模拟延迟
    await new Promise(resolve => setTimeout(resolve, 1000))

    // 模拟数据（测试用）
    const mockAddress = '0x' + '1234567890abcdef'.repeat(4)
    walletAddress.value = mockAddress

    // 生成模拟签名
    const mockSignature = '0x' + 'abcd1234'.repeat(20)
    const mockMessage = `Login to NRCS Platform at ${new Date().toISOString()}`

    // 调用登录
    await accountStore.login({
      wallet_address: mockAddress,
      signature: mockSignature,
      message: mockMessage
    })

    ElMessage.success(t('common.success'))
    router.push(redirectPath.value)
  } catch (error: any) {
    ElMessage.error(error.message || t('login.loginFailed'))
  } finally {
    isConnecting.value = false
  }
}

// 检查是否已登录
if (accountStore.isLoggedIn) {
  router.push(redirectPath.value)
}
</script>

<template>
  <div class="login-container">
    <div class="login-card">
      <div class="logo-section">
        <img src="@/assets/images/logo.svg" alt="NRCS" class="logo" />
        <h1 class="title">{{ t('login.title') }}</h1>
        <p class="subtitle">{{ t('login.subtitle') }}</p>
      </div>

      <div class="form-section">
        <el-form @submit.prevent="handleConnectWallet">
          <el-form-item>
            <div class="wallet-input-wrapper">
              <el-input
                v-model="walletAddress"
                :placeholder="t('login.walletAddress')"
                disabled
                class="wallet-input"
              />
              <el-button
                type="primary"
                :loading="isConnecting"
                @click="handleConnectWallet"
                class="connect-btn"
              >
                {{ isConnecting ? t('common.loading') : t('login.connectWallet') }}
              </el-button>
            </div>
          </el-form-item>
        </el-form>

        <div class="divider">
          <span>或</span>
        </div>

        <el-button class="wallet-option" disabled>
          <img src="https://metamask.io/img/metamask-icon.svg" alt="MetaMask" width="24" />
          MetaMask
        </el-button>

        <el-button class="wallet-option" disabled>
          <img src="https://wallet.trezor.io/icon/apple-touch-icon-120x120.png" alt="Trezor" width="24" />
          Trezor
        </el-button>

        <div class="links">
          <router-link to="/register">{{ t('login.register') }}</router-link>
        </div>
      </div>

      <div class="footer">
        <p>{{ t('login.signPrompt') }}</p>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 100%);
  padding: 20px;
}

.login-card {
  width: 100%;
  max-width: 420px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  padding: 40px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.logo-section {
  text-align: center;
  margin-bottom: 32px;

  .logo {
    width: 80px;
    height: 80px;
    margin-bottom: 16px;
  }

  .title {
    font-size: 24px;
    font-weight: 600;
    margin: 0 0 8px;
    color: #fff;
  }

  .subtitle {
    font-size: 14px;
    color: #909399;
    margin: 0;
  }
}

.form-section {
  .wallet-input-wrapper {
    display: flex;
    gap: 8px;

    .wallet-input {
      flex: 1;
    }

    .connect-btn {
      min-width: 100px;
    }
  }

  .divider {
    display: flex;
    align-items: center;
    margin: 24px 0;
    color: #909399;

    &::before,
    &::after {
      content: '';
      flex: 1;
      height: 1px;
      background: rgba(255, 255, 255, 0.1);
    }

    span {
      padding: 0 16px;
      font-size: 12px;
    }
  }

  .wallet-option {
    width: 100%;
    justify-content: flex-start;
    margin-bottom: 12px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #fff;

    img {
      margin-right: 8px;
    }

    &:hover {
      background: rgba(255, 255, 255, 0.1);
    }

    &:disabled {
      opacity: 0.6;
    }
  }

  .links {
    text-align: center;
    margin-top: 24px;

    a {
      color: #409eff;
      text-decoration: none;
      font-size: 14px;

      &:hover {
        text-decoration: underline;
      }
    }
  }
}

.footer {
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  text-align: center;

  p {
    font-size: 12px;
    color: #909399;
    margin: 0;
    line-height: 1.6;
  }
}
</style>
