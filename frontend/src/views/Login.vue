<template>
  <div class="login-page">
    <div class="login-background">
      <div class="bg-gradient"></div>
      <div class="bg-grid"></div>
      <div class="bg-glow bg-glow--1"></div>
      <div class="bg-glow bg-glow--2"></div>
    </div>

    <div class="login-container">
      <div class="login-branding">
        <h1 class="brand-name">NRCS</h1>
        <p class="brand-tagline">{{ t('login.subtitle') }}</p>
      </div>

      <!-- Connection Status -->
      <div class="connection-status" :class="connectionStatusClass">
        <div class="status-indicator"></div>
        <span>{{ connectionStatusText }}</span>
        <button v-if="connectionStatus === 'error'" @click="checkNodeConnection" class="retry-btn">
          重试
        </button>
      </div>

      <!-- Error Message -->
      <div v-if="errorMessage" class="login-error">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="#ef4444" stroke-width="2"/>
          <line x1="12" y1="8" x2="12" y2="12" stroke="#ef4444" stroke-width="2" stroke-linecap="round"/>
          <circle cx="12" cy="16" r="1" fill="#ef4444"/>
        </svg>
        <span>{{ errorMessage }}</span>
      </div>

      <div class="login-card">
        <!-- Login Type Selector -->
        <div class="login-type-selector">
          <button
            class="type-btn"
            :class="{ 'is-active': loginType === 'account' }"
            @click="switchLoginType('account')"
            title="账号登录"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
              <circle cx="12" cy="7" r="4" stroke="currentColor" stroke-width="1.8"/>
            </svg>
          </button>
          <button
            class="type-btn"
            :class="{ 'is-active': loginType === 'password' }"
            @click="switchLoginType('password')"
            title="密码登录"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <rect x="5" y="11" width="14" height="10" rx="2" stroke="currentColor" stroke-width="1.8"/>
              <path d="M8 11V7a4 4 0 118 0v4" stroke="currentColor" stroke-width="1.8"/>
            </svg>
          </button>
        </div>

        <!-- Account Login Form -->
        <form v-if="loginType === 'account'" @submit.prevent="handleAccountLogin" class="login-form">
          <div class="form-group">
            <!-- Saved Accounts Dropdown -->
            <div v-if="savedAccounts.length > 0 && !showManualInput" class="input-wrapper">
              <select v-model="selectedAccount" class="form-select" @change="onAccountSelect">
                <option value="">选择已保存的账号</option>
                <option v-for="(account, index) in savedAccounts" :key="index" :value="account">
                  {{ account }}
                </option>
                <option value="__other__">手动输入</option>
              </select>
            </div>

            <!-- Manual Account Input -->
            <div v-if="showManualInput || savedAccounts.length === 0" class="input-wrapper">
              <input
                v-model="accountInput"
                type="text"
                class="form-input"
                :class="{ 'is-focused': isAccountFocused }"
                placeholder="NRCS-SM2H-LPVM-ES9M-94C92"
                @focus="isAccountFocused = true"
                @blur="isAccountFocused = false"
                @keyup.enter="handleAccountLogin"
              />
              <button type="submit" class="input-action-btn" title="登录">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
                  <path d="M5 12h14M12 5l7 7-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </button>
            </div>

            <!-- Toggle Button for Saved Accounts -->
            <div v-if="savedAccounts.length > 0 && !showManualInput" class="toggle-input-mode">
              <button type="button" @click="showManualInput = true; selectedAccount = ''" class="toggle-link">
                或手动输入地址 →
              </button>
            </div>

            <div v-if="savedAccounts.length > 0 && showManualInput" class="toggle-input-mode">
              <button type="button" @click="showManualInput = false; accountInput = ''" class="toggle-link">
                ← 返回已保存账号
              </button>
            </div>

            <!-- Quick Login Button for Selected Account -->
            <button
              v-if="selectedAccount && selectedAccount !== '__other__' && !showManualInput"
              type="button"
              class="btn-login btn-login--full"
              @click="handleAccountLogin"
              :disabled="loading"
            >
              <span v-if="loading" class="spinner"></span>
              <span v-else>登 录</span>
            </button>

            <!-- Login Button for Manual Input -->
            <button
              v-if="showManualInput || savedAccounts.length === 0"
              type="button"
              class="btn-login btn-login--full"
              @click="handleAccountLogin"
              :disabled="loading || !accountInput.trim()"
            >
              <span v-if="loading" class="spinner"></span>
              <span v-else>登 录</span>
            </button>
          </div>
        </form>

        <!-- Password Login Form -->
        <form v-else-if="loginType === 'password'" @submit.prevent="handlePasswordLogin" class="login-form">
          <div class="form-group">
            <label class="form-label">密码短语</label>
            <div class="input-wrapper">
              <input
                v-model="secretPhrase"
                type="password"
                class="form-input"
                :class="{ 'is-focused': isPasswordFocused }"
                placeholder="输入您的密码短语..."
                @focus="isPasswordFocused = true"
                @blur="isPasswordFocused = false"
                @keyup.enter="handlePasswordLogin"
              />
              <button
                type="button"
                class="toggle-password"
                @click="togglePasswordVisibility"
                title="切换可见性"
              >
                <svg v-if="!showPassword" width="18" height="18" viewBox="0 0 24 24" fill="none">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8S1 12 1 12z" stroke="currentColor" stroke-width="1.8"/>
                  <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.8"/>
                </svg>
                <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none">
                  <path d="M17.94 17.94A10.07 10.07 0 0112 20c-7 0-11-8-11-8a18.45 18.45 0 015.06-5.94M9.9 4.24A9.12 9.12 0 0112 4c7 0 11 8 11 8a18.5 18.5 0 01-2.16 3.19m-6.72-1.07a3 3 0 11-4.24-4.24" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
                  <line x1="1" y1="1" x2="23" y2="23" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
                </svg>
              </button>
              <button type="submit" class="input-action-btn" title="登录">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
                  <path d="M5 12h14M12 5l7 7-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </button>
            </div>
          </div>

          <button
            type="button"
            class="btn-login btn-login--full"
            @click="handlePasswordLogin"
            :disabled="loading"
          >
            <span v-if="loading" class="spinner"></span>
            <span v-else>登 录</span>
          </button>
        </form>

        <!-- Options -->
        <div class="login-options">
          <label class="checkbox-wrapper">
            <input type="checkbox" v-model="rememberMe" class="checkbox-input" />
            <span class="checkbox-custom"></span>
            <span class="checkbox-label">记住我</span>
          </label>
        </div>

        <!-- Registration Link -->
        <div class="registration-link">
          <a href="#" @click.prevent="showRegistrationInfo">还没有账号？点击创建新账号！</a>
        </div>
      </div>

      <div class="login-footer">
        <p>Powered by NRCS Blockchain</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()

const loading = ref(false)
const rememberMe = ref(false)
const showPassword = ref(false)

const loginType = ref<'account' | 'password'>('account')
const accountInput = ref('')
const secretPhrase = ref('')
const selectedAccount = ref('')
const savedAccounts = ref<string[]>([])
const showManualInput = ref(false)

const errorMessage = ref('')
const connectionStatus = ref<'checking' | 'online' | 'error'>('checking')

const isAccountFocused = ref(false)
const isPasswordFocused = ref(false)

const connectionStatusClass = computed(() => ({
  'is-online': connectionStatus.value === 'online',
  'is-error': connectionStatus.value === 'error',
  'is-checking': connectionStatus.value === 'checking'
}))

const connectionStatusText = computed(() => {
  switch (connectionStatus.value) {
    case 'online':
      return '节点已连接'
    case 'error':
      return '节点离线'
    case 'checking':
      return '正在检测节点...'
    default:
      return '未知状态'
  }
})

const switchLoginType = (type: 'account' | 'password') => {
  loginType.value = type
  errorMessage.value = ''
}

const togglePasswordVisibility = () => {
  showPassword.value = !showPassword.value
}

const checkNodeConnection = async () => {
  connectionStatus.value = 'checking'

  try {
    const status = await nrcsApi.getBlockchainStatus()
    if ((status as any).errorCode) {
      connectionStatus.value = 'error'
      console.error('Blockchain error:', (status as any).errorDescription)
    } else {
      connectionStatus.value = 'online'
      console.log('Blockchain connected, height:', (status as any).numberOfBlocks)
    }
  } catch (error: any) {
    connectionStatus.value = 'error'
    console.error('Connection failed:', error)
  }
}

const onAccountSelect = () => {
  errorMessage.value = ''
}

const loadSavedAccounts = () => {
  const accounts = localStorage.getItem('saved_nrcs_accounts')
  if (accounts) {
    savedAccounts.value = accounts.split(';').filter(a => a.trim() !== '')
  }
}

const saveAccount = (accountRS: string) => {
  if (!rememberMe.value) return

  let accounts = localStorage.getItem('saved_nrcs_accounts') || ''
  const accountList = accounts.split(';').filter(a => a.trim() !== '')

  if (!accountList.includes(accountRS)) {
    accountList.push(accountRS)
    localStorage.setItem('saved_nrcs_accounts', accountList.join(';'))
  }
}

const handleAccountLogin = async () => {
  let account = ''

  if (showManualInput.value || savedAccounts.value.length === 0) {
    account = accountInput.value
  } else {
    if (selectedAccount.value === '__other__' || !selectedAccount.value) {
      account = accountInput.value
    } else {
      account = selectedAccount.value
    }
  }

  console.log('Login attempt with account:', account, 'showManualInput:', showManualInput.value)

  if (!account || account.trim() === '') {
    errorMessage.value = '请输入您的 NRC 账号地址'
    return
  }

  const trimmedAccount = account.trim()

  if (!trimmedAccount.startsWith('NRCS-')) {
    errorMessage.value = '无效的 NRC 账号格式，应为 NRCS-XXXX-XXXX-XXXX-XXXXX'
    return
  }

  try {
    loading.value = true
    errorMessage.value = ''

    console.log('=== 账号登录开始 ===')
    console.log('账号:', trimmedAccount)
    console.log('登录模式:', showManualInput.value ? '手动' : '已保存')

    console.log('步骤 1: 调用 getBlockchainStatus...')

    let status
    try {
      status = await nrcsApi.getBlockchainStatus()
      console.log('步骤 1 成功: BlockchainStatus:', JSON.stringify(status).substring(0, 100))
    } catch (statusError: any) {
      console.error('步骤 1 失败:', statusError)
      errorMessage.value = '无法连接到 NRCS 节点，请确保节点在 http://localhost:17976 运行'
      return
    }

    if ((status as any).errorCode) {
      errorMessage.value = `区块链错误: ${(status as any).errorDescription || '未知错误'} (代码: ${(status as any).errorCode})`
      console.error('步骤 1 错误:', errorMessage.value)
      return
    }

    console.log('步骤 2: 调用 getAccount 查询', trimmedAccount)

    let result
    try {
      result = await nrcsApi.getAccount(trimmedAccount)
      console.log('步骤 2 成功: 账户数据:', JSON.stringify(result).substring(0, 200))
    } catch (accountError: any) {
      console.error('步骤 2 失败:', accountError)

      if (accountError.code === 5) {
        errorMessage.value = '区块链上未找到该账户，请检查地址是否正确'
      } else if (accountError.message?.includes('Network') || accountError.message?.includes('network')) {
        errorMessage.value = '网络错误，请检查您的网络连接'
      } else {
        errorMessage.value = `API 错误: ${accountError.message || accountError.description || '未知'}`
        console.error('完整错误对象:', accountError)
      }
      return
    }

    console.log('步骤 3: 处理登录响应...')

    if (result && result.accountRS) {
      console.log('✅ 账号登录成功:', result.accountRS)

      localStorage.setItem('access_token', 'nrcs_session')
      localStorage.setItem('nrcs_account_rs', result.accountRS)
      localStorage.setItem('nrcs_account_id', result.account)
      localStorage.setItem('nrcs_public_key', result.publicKey || '')
      localStorage.setItem('logged_in', 'true')
      localStorage.setItem('login_type', 'account')

      saveAccount(result.accountRS)

      try {
        if (result.balanceNQT) {
          localStorage.setItem('nrcs_balance_nqt', result.balanceNQT)
          console.log('💰 余额已保存:', result.balanceNQT, 'NQT')
        }
      } catch {}

      ElMessage.success(`欢迎回来！已登录 ${result.accountRS}`)
      const redirect = route.query.redirect as string

      console.log('步骤 4: 跳转到', redirect || '/')

      setTimeout(() => {
        router.push(redirect || '/')
      }, 500)
    } else {
      errorMessage.value = '服务器返回了无效的响应'
      console.error('❌ 无效的响应:', result)
    }
  } catch (error: any) {
    console.error('❌ handleAccountLogin 发生意外错误:', error)
    errorMessage.value = `意外错误: ${error.message || '发生了未知错误'}`
  } finally {
    loading.value = false
    console.log('=== 账号登录完成 ===')
  }
}

const handlePasswordLogin = async () => {
  if (!secretPhrase.value || secretPhrase.value.trim() === '') {
    errorMessage.value = '密码短语不能为空'
    return
  }

  const trimmedPhrase = secretPhrase.value.trim()

  if (trimmedPhrase.length < 35) {
    errorMessage.value = '警告：密码短语应至少包含 35 个字符以保证安全性'
    return
  }

  try {
    loading.value = true
    errorMessage.value = ''

    console.log('密码登录: 调用 getBlockchainStatus')

    let status
    try {
      status = await nrcsApi.getBlockchainStatus()
    } catch (statusError: any) {
      console.error('getBlockchainStatus 失败:', statusError)
      errorMessage.value = '无法连接到 NRCS 节点，请确保节点在 http://localhost:17976 运行'
      return
    }

    if ((status as any).errorCode) {
      errorMessage.value = `区块链错误: ${(status as any).errorDescription || '未知错误'} (代码: ${(status as any).errorCode})`
      return
    }

    console.log('区块链状态正常，调用 getAccountId')

    let result
    try {
      result = await nrcsApi.getAccountId(trimmedPhrase)
    } catch (idError: any) {
      console.error('getAccountId 失败:', idError)
      errorMessage.value = idError.message || idError.description || '无法从密码短语生成账号 ID'
      return
    }

    console.log('getAccountId 响应:', result)

    if (result && result.accountRS) {
      console.log('密码登录成功:', result.accountRS)

      localStorage.setItem('access_token', 'nrcs_session')
      localStorage.setItem('nrcs_account_rs', result.accountRS)
      localStorage.setItem('nrcs_account_id', result.account)
      localStorage.setItem('nrcs_public_key', result.publicKey || '')
      localStorage.setItem('logged_in', 'true')
      localStorage.setItem('login_type', 'password')

      if (rememberMe.value) {
        localStorage.setItem('nrcs_remember', 'true')
        localStorage.setItem('saved_passphrase', trimmedPhrase)
      } else {
        localStorage.removeItem('nrcs_remember')
        localStorage.removeItem('saved_passphrase')
      }

      saveAccount(result.accountRS)

      try {
        const account = await nrcsApi.getAccount(result.accountRS)
        if (account && account.balanceNQT) {
          localStorage.setItem('nrcs_balance_nqt', account.balanceNQT)
        }
      } catch (balanceError: any) {
        console.warn('获取余额失败:', balanceError)
      }

      ElMessage.success(`欢迎！账号已创建: ${result.accountRS}`)
      const redirect = route.query.redirect as string

      setTimeout(() => {
        router.push(redirect || '/')
      }, 500)
    } else {
      errorMessage.value = '无法从密码短语生成账号 ID，请检查您的密码短语'
    }
  } catch (error: any) {
    console.error('密码登录发生意外错误:', error)
    errorMessage.value = error.message || '登录过程中发生了未知错误'
  } finally {
    loading.value = false
  }
}

const showRegistrationInfo = () => {
  ElMessageBox.alert(
    '要创建新的 NRCS 账号，您需要生成一个安全的密码短语。此密码短语将用于访问您的账号。请务必安全保存！',
    '创建新账号',
    {
      confirmButtonText: '确定',
      type: 'info',
    }
  )
}

onMounted(() => {
  loadSavedAccounts()

  const remembered = localStorage.getItem('nrcs_remember')
  if (remembered === 'true') {
    rememberMe.value = true
  }

  checkNodeConnection()
})
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables' as *;

.login-page {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: $bg;
  overflow: hidden;
  font-family: $font-body;
}

.login-background {
  position: absolute;
  inset: 0;
  z-index: 0;

  .bg-gradient {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(ellipse at 20% 50%, rgba($primary, 0.08) 0%, transparent 50%),
      radial-gradient(ellipse at 80% 80%, rgba($accent-2, 0.06) 0%, transparent 50%),
      radial-gradient(ellipse at 50% 20%, rgba($primary-dark, 0.05) 0%, transparent 40%);
  }

  .bg-grid {
    position: absolute;
    inset: 0;
    background-image:
      linear-gradient(rgba(255, 255, 255, 0.02) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.02) 1px, transparent 1px);
    background-size: 60px 60px;
    mask-image: radial-gradient(ellipse at center, black 30%, transparent 70%);
  }

  .bg-glow {
    position: absolute;
    border-radius: 50%;
    filter: blur(120px);
    animation: float 15s ease-in-out infinite;

    &--1 {
      width: 500px;
      height: 500px;
      top: -200px;
      left: -150px;
      background: rgba($primary, 0.08);
      animation-delay: 0s;
    }

    &--2 {
      width: 400px;
      height: 400px;
      bottom: -150px;
      right: -100px;
      background: rgba($accent-2, 0.06);
      animation-delay: -7s;
    }
  }

  @keyframes float {
    0%, 100% { transform: translate(0, 0); }
    33% { transform: translate(30px, -30px); }
    66% { transform: translate(-20px, 20px); }
  }
}

.login-container {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 420px;
  padding: $space-lg;
}

.login-branding {
  text-align: center;
  margin-bottom: $space-xl;

  .brand-name {
    font-size: $font-size-3xl;
    font-weight: 700;
    letter-spacing: 12px;
    color: #fff;
    margin: 0 0 $space-xs 0;
    background: linear-gradient(135deg, #fff 0%, rgba($primary-hover, 0.9) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .brand-tagline {
    font-size: $font-size-base;
    color: $text-secondary;
    margin: $space-sm 0 0;
    letter-spacing: $letter-spacing-normal;
    max-width: 600px;
    line-height: $line-height-normal;
  }
}

.connection-status {
  display: flex;
  align-items: center;
  gap: $space-sm;
  margin-bottom: $space-lg;
  padding: $space-sm $space-md;
  background: rgba($card, 0.6);
  border-radius: $radius-md;
  font-size: $font-size-xs;
  color: $text-secondary;
  transition: all $duration-normal;

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: $text-muted;

    &::before {
      content: '';
      position: absolute;
      inset: -2px;
      border-radius: 50%;
      animation: none;
    }
  }

  &.is-online {
    background: $success-subtle;
    color: $success;

    .status-indicator {
      background: $success;
      box-shadow: 0 0 8px $success-muted;

      &::before {
        background: $success;
        opacity: 0.3;
        animation: pulse 2s ease-in-out infinite;
      }
    }
  }

  &.is-error {
    background: $danger-subtle;
    color: $danger;

    .status-indicator {
      background: $danger;
      box-shadow: 0 0 8px $danger-muted;
    }

    .retry-btn {
      margin-left: auto;
      padding: 2px $space-sm;
      background: rgba($danger, 0.15);
      border: 1px solid rgba($danger, 0.3);
      border-radius: $radius-sm;
      color: $danger;
      font-size: $font-size-xs;
      cursor: pointer;
      transition: all $duration-fast;

      &:hover {
        background: rgba($danger, 0.25);
        border-color: rgba($danger, 0.5);
      }
    }
  }

  &.is-checking {
    color: $warning;

    .status-indicator {
      background: $warning;
      animation: blink 1s ease-in-out infinite;
    }
  }

  span {
    flex: 1;
  }
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.3; }
  50% { transform: scale(1.5); opacity: 0; }
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.login-error {
  display: flex;
  align-items: center;
  gap: $space-sm;
  margin-bottom: $space-lg;
  padding: $space-md;
  background: $danger-subtle;
  border: 1px solid rgba($danger, 0.25);
  border-radius: $radius-md;
  color: $danger;
  font-size: $font-size-sm;

  svg {
    flex-shrink: 0;
  }

  span {
    line-height: 1.5;
  }
}

.login-card {
  background: linear-gradient(135deg, rgba($card, 0.9), rgba($panel-strong, 0.95));
  backdrop-filter: blur(20px);
  border-radius: $radius-xl;
  border: 1px solid $border-default;
  padding: $space-xl;
  box-shadow: $shadow-lg;
  transition: all $duration-slow $ease-out;

  &:hover {
    box-shadow: 0 32px 64px rgba(0, 0, 0, 0.4);
    border-color: rgba($primary, 0.2);
  }
}

.login-type-selector {
  display: flex;
  gap: $space-sm;
  margin-bottom: $space-xl;
  justify-content: center;

  .type-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 48px;
    background: rgba($bg, 0.6);
    border: 1.5px solid $border-default;
    border-radius: $radius-md;
    color: $text-secondary;
    cursor: pointer;
    transition: all $duration-normal $ease-out;

    &:hover {
      color: $text-primary;
      border-color: $border-hover;
      background: rgba($bg, 0.8);
    }

    &.is-active {
      background: linear-gradient(135deg, rgba($primary, 0.15), rgba($primary-dark, 0.08));
      border-color: $primary;
      color: $primary;
      box-shadow: 0 0 20px $primary-glow;
    }
  }
}

.login-form {
  .form-group {
    margin-bottom: $space-lg;
  }

  .form-label {
    display: block;
    font-size: $font-size-sm;
    font-weight: 500;
    color: $text-secondary;
    margin-bottom: $space-sm;
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;

    .form-input,
    .form-select {
      flex: 1;
      height: 48px;
      padding: 0 $space-lg;
      padding-right: 52px;
      background: rgba($bg, 0.6);
      border: 1.5px solid $border-default;
      border-radius: $radius-md;
      color: $text-primary;
      font-size: $font-size-base;
      outline: none;
      transition: all $duration-normal $ease-out;
      appearance: none;
      cursor: pointer;
      font-family: inherit;

      &::placeholder {
        color: $text-muted;
      }

      &:focus,
      &.is-focused {
        border-color: $primary;
        background: rgba($bg, 0.9);
        box-shadow: $focus-ring;

        + .toggle-password svg path,
        + .toggle-password svg line,
        + .toggle-password svg circle,
        + .input-action-btn {
          color: $primary;
        }
      }

      &:hover:not(:focus):not(.is-focused) {
        border-color: $border-hover;
        background: rgba($bg, 0.75);
      }
    }

    .form-select {
      option {
        background: $panel-strong;
        color: $text-primary;
      }
    }

    .toggle-password,
    .input-action-btn {
      position: absolute;
      right: $space-sm;
      top: 50%;
      transform: translateY(-50%);
      display: flex;
      align-items: center;
      justify-content: center;
      width: 36px;
      height: 36px;
      background: transparent;
      border: none;
      cursor: pointer;
      color: $text-secondary;
      transition: all $duration-fast;
      border-radius: $radius-sm;

      &:hover {
        color: $primary;
        background: $primary-subtle;
      }
    }

    .input-action-btn {
      color: $primary;

      &:hover {
        background: rgba($primary, 0.15);
      }
    }
  }

  .btn-login--full {
    width: 100%;
    margin-top: $space-md;
  }

  .toggle-input-mode {
    margin-top: $space-sm;
    text-align: center;

    .toggle-link {
      background: none;
      border: none;
      color: $primary;
      font-size: $font-size-sm;
      cursor: pointer;
      padding: $space-xs 0;
      transition: color $duration-fast;

      &:hover {
        color: $primary-hover;
        text-decoration: underline;
      }
    }
  }
}

.btn-login {
  position: relative;
  height: 48px;
  background: linear-gradient(135deg, $primary, $primary-dark);
  border: none;
  border-radius: $radius-md;
  color: $primary-foreground;
  font-size: $font-size-base;
  font-weight: 600;
  cursor: pointer;
  overflow: hidden;
  transition: all $duration-normal $ease-out;
  box-shadow: 0 4px 16px rgba($primary, 0.35);

  &::before {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(135deg, transparent, rgba(255, 255, 255, 0.2));
    opacity: 0;
    transition: opacity $duration-fast;
  }

  &:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba($primary, 0.45);

    &::before {
      opacity: 1;
    }
  }

  &:active:not(:disabled) {
    transform: translateY(0);
    box-shadow: 0 4px 16px rgba($primary, 0.35);
  }

  &:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  &.is-loading {
    pointer-events: none;

    .spinner {
      display: inline-block;
      width: 20px;
      height: 20px;
      border: 2.5px solid rgba($primary-foreground, 0.2);
      border-top-color: $primary-foreground;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
    }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
}

.login-options {
  margin-top: $space-lg;
  padding-top: $space-lg;
  border-top: 1px solid $border-default;
}

.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: $space-sm;
  cursor: pointer;
  user-select: none;

  .checkbox-input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;

    &:checked + .checkbox-custom {
      background: $primary;
      border-color: $primary;

      &::after {
        opacity: 1;
        transform: scale(1);
      }
    }

    &:focus-visible + .checkbox-custom {
      box-shadow: 0 0 0 3px $primary-subtle;
    }
  }

  .checkbox-custom {
    position: relative;
    width: 18px;
    height: 18px;
    background: rgba($bg, 0.6);
    border: 1.5px solid $border-default;
    border-radius: 4px;
    transition: all $duration-fast;

    &::after {
      content: '';
      position: absolute;
      left: 5px;
      top: 2px;
      width: 5px;
      height: 9px;
      border: solid white;
      border-width: 0 2px 2px 0;
      transform: scale(0) rotate(45deg);
      opacity: 0;
      transition: all $duration-fast $ease-spring;
    }
  }

  .checkbox-label {
    font-size: $font-size-sm;
    color: $text-secondary;
    transition: color $duration-fast;

    &:hover {
      color: $text-primary;
    }
  }

  &:hover .checkbox-custom {
    border-color: $primary;
  }
}

.registration-link {
  margin-top: $space-lg;
  text-align: center;

  a {
    font-size: $font-size-sm;
    color: $primary;
    text-decoration: none;
    transition: color $duration-fast;

    &:hover {
      color: $primary-hover;
      text-decoration: underline;
    }
  }
}

.login-footer {
  text-align: center;
  margin-top: $space-xl;

  p {
    font-size: $font-size-xs;
    color: $text-muted;
    margin: 0;
  }
}

@media (max-width: 480px) {
  .login-container {
    max-width: 100%;
    padding: $space-md;
  }

  .login-branding {
    .brand-name {
      font-size: 28px;
      letter-spacing: 8px;
    }
  }

  .login-card {
    padding: $space-lg;
  }
}
</style>
