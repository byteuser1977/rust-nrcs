<script setup lang="ts">
/**
 * @deprecated 此组件为以太坊风格钱包登录，与 NRCS 安全模型（本地派生 secretPhrase）冲突。
 * 路由实际使用 `@/views/Login.vue`（NRCS 风格双模式登录）。本文件保留仅供历史参考，
 * 将在阶段 3.x 统一清理。新代码请勿使用。
 *
 * 对标参考：nrs.login.js（NRCS Java 前端）—— secretPhrase 仅在客户端本地派生
 * publicKey/accountId/accountRS，绝不外发。请使用 useAccountStore.login(password)。
 */
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const accountStore = useAccountStore()

// 表单数据
const walletAddress = ref('')
const isConnecting = ref(false)

// 计算属性
const redirectPath = computed(() => (route.query.redirect as string) || '/')

/**
 * @deprecated 以太坊钱包连接登录已废弃。
 * NRCS 使用本地派生的 secretPhrase 登录，请改用：
 *   await accountStore.login(secretPhrase, { rememberMe: true })
 * 或只读账户登录：
 *   await accountStore.loginByAccount(accountRS, { rememberMe: true })
 */
async function handleConnectWallet() {
  if (isConnecting.value) return

  isConnecting.value = true
  try {
    // ⛔ 已废弃：以太坊钱包连接登录逻辑与 NRCS 模型冲突，已屏蔽。
    // 原实现调用 accountStore.login({wallet_address, signature, message})，
    // 但 NRCS accountStore.login 签名为 (password: string, options?)。
    // 如需钱包登录，请在阶段 3.x 基于 NRCS 模型重新设计。
    ElMessage.warning('此登录方式已废弃，请使用 NRCS 密码短语登录（/login）')
    router.push('/login')
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
