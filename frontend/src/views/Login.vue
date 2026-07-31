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

      <!-- Password Strength Warning (对标 nrs.login.js:415-425 passwordNotice) -->
      <div v-if="passwordWarning" class="login-warning">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" stroke="#f59e0b" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <line x1="12" y1="9" x2="12" y2="13" stroke="#f59e0b" stroke-width="2" stroke-linecap="round"/>
          <circle cx="12" cy="17" r="1" fill="#f59e0b"/>
        </svg>
        <span>{{ passwordWarning }}</span>
      </div>

      <div class="login-card">
        <!-- Login Type Selector -->
        <div class="login-type-selector">
          <button
            class="type-btn"
            :class="{ 'is-active': loginType === 'account' }"
            @click="switchLoginType('account')"
            title="账号登录（只读）"
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
            title="密码短语登录（可签名）"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <rect x="5" y="11" width="14" height="10" rx="2" stroke="currentColor" stroke-width="1.8"/>
              <path d="M8 11V7a4 4 0 118 0v4" stroke="currentColor" stroke-width="1.8"/>
            </svg>
          </button>
        </div>

        <!-- Account Login Form (只读模式) -->
        <form v-if="loginType === 'account'" @submit.prevent="handleAccountLogin" class="login-form">
          <div class="form-group">
            <label class="form-label">账户地址（只读模式）</label>

            <!-- Saved Accounts Dropdown -->
            <div v-if="accountStore.savedAccounts.length > 0 && !showManualInput" class="input-wrapper">
              <select v-model="selectedAccount" class="form-select" @change="onAccountSelect">
                <option value="">选择已保存的账号</option>
                <option v-for="(account, index) in accountStore.savedAccounts" :key="index" :value="account">
                  {{ account }}
                </option>
                <option value="__other__">手动输入</option>
              </select>
            </div>

            <!-- Manual Account Input -->
            <div v-if="showManualInput || accountStore.savedAccounts.length === 0" class="input-wrapper">
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
            <div v-if="accountStore.savedAccounts.length > 0 && !showManualInput" class="toggle-input-mode">
              <button type="button" @click="showManualInput = true; selectedAccount = ''" class="toggle-link">
                或手动输入地址 →
              </button>
            </div>

            <div v-if="accountStore.savedAccounts.length > 0 && showManualInput" class="toggle-input-mode">
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
              v-if="showManualInput || accountStore.savedAccounts.length === 0"
              type="button"
              class="btn-login btn-login--full"
              @click="handleAccountLogin"
              :disabled="loading || !accountInput.trim()"
            >
              <span v-if="loading" class="spinner"></span>
              <span v-else>登 录</span>
            </button>

            <p class="form-hint">只读模式：可查看余额与历史，发送交易需切换密码短语登录</p>
          </div>
        </form>

        <!-- Password Login Form (助记词/密码短语) -->
        <form v-else-if="loginType === 'password'" @submit.prevent="handlePasswordLogin" class="login-form">
          <div class="form-group">
            <label class="form-label">密码短语 / 助记词</label>
            <div class="input-wrapper">
              <input
                v-model="secretPhrase"
                :type="showPassword ? 'text' : 'password'"
                class="form-input"
                :class="{ 'is-focused': isPasswordFocused }"
                placeholder="输入您的 12 词助记词或密码短语..."
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

            <!-- Strength Indicator -->
            <div v-if="secretPhrase" class="strength-indicator">
              <div class="strength-bar">
                <div class="strength-fill" :class="strengthLevel" :style="{ width: strength.score + '%' }"></div>
              </div>
              <span class="strength-label" :class="strengthLevel">{{ strengthText }}</span>
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
            <span class="checkbox-label">记住账户（仅存 RS 地址）</span>
          </label>
        </div>

        <!-- Registration Link -->
        <div class="registration-link">
          <a href="#" @click.prevent="goToRegister">还没有账号？点击创建新账号！</a>
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
import { ElMessage } from 'element-plus'
import { useAccountStore } from '@/stores/modules'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { checkPassphraseStrength } from '@/utils/mnemonic'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const accountStore = useAccountStore()

const loading = ref(false)
const rememberMe = ref(false)
const showPassword = ref(false)

const loginType = ref<'account' | 'password'>('account')
const accountInput = ref('')
const secretPhrase = ref('')
const selectedAccount = ref('')
const showManualInput = ref(false)

const errorMessage = ref('')
const passwordWarning = ref('')
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

/** 密码强度评估（对标 nrs.login.js:415-425） */
const strength = computed(() => checkPassphraseStrength(secretPhrase.value))

const strengthLevel = computed(() => strength.value.level)

const strengthText = computed(() => {
  switch (strength.value.level) {
    case 'strong':
      return '强度：高'
    case 'medium':
      return '强度：中'
    case 'weak':
      return '强度：弱'
    default:
      return '强度：极弱'
  }
})

/** 切换登录方式 */
function switchLoginType(type: 'account' | 'password'): void {
  loginType.value = type
  errorMessage.value = ''
  passwordWarning.value = ''
}

/** 切换密码可见性 */
function togglePasswordVisibility(): void {
  showPassword.value = !showPassword.value
}

/** 检测节点连接状态 */
async function checkNodeConnection(): Promise<void> {
  connectionStatus.value = 'checking'
  try {
    const status = await nrcsApi.getBlockchainStatus()
    if ((status as any).errorCode) {
      connectionStatus.value = 'error'
      console.error('Blockchain error:', (status as any).errorDescription)
    } else {
      connectionStatus.value = 'online'
      console.log('[login] 节点已连接, height:', (status as any).numberOfBlocks)
    }
  } catch (error: any) {
    connectionStatus.value = 'error'
    console.error('[login] 节点连接失败:', error)
  }
}

/** 选择已保存账户 */
function onAccountSelect(): void {
  errorMessage.value = ''
  if (selectedAccount.value === '__other__') {
    showManualInput.value = true
    selectedAccount.value = ''
  }
}

/**
 * 账户地址登录（只读模式）
 *
 * 对标 nrs.login.js:334-336 的 NRS.login(false, account)：
 * 调用 store.loginByAccount 仅查询链上信息，不存储 secretPhrase。
 */
async function handleAccountLogin(): Promise<void> {
  let account = ''

  if (showManualInput.value || accountStore.savedAccounts.length === 0) {
    account = accountInput.value
  } else if (selectedAccount.value === '__other__' || !selectedAccount.value) {
    account = accountInput.value
  } else {
    account = selectedAccount.value
  }

  if (!account || account.trim() === '') {
    errorMessage.value = '请输入您的 NRCS 账号地址'
    return
  }

  const trimmedAccount = account.trim()

  if (!trimmedAccount.startsWith('NRCS-') && !/^\d+$/.test(trimmedAccount)) {
    errorMessage.value = '无效的账号格式，应为 NRCS-XXXX-XXXX-XXXX-XXXXX 或数字账户 ID'
    return
  }

  try {
    loading.value = true
    errorMessage.value = ''

    console.log('[login] 只读账户登录:', trimmedAccount)

    await accountStore.loginByAccount(trimmedAccount, {
      rememberMe: rememberMe.value
    })

    ElMessage.success(`欢迎回来！已登录 ${accountStore.accountRS}`)

    const redirect = route.query.redirect as string
    router.push(redirect || '/')
  } catch (error: any) {
    console.error('[login] 账户登录失败:', error)
    if (error.code === 5) {
      errorMessage.value = '区块链上未找到该账户，请检查地址是否正确'
    } else if (error.message?.includes('网络') || error.message?.includes('Network')) {
      errorMessage.value = '网络错误，请检查您的网络连接'
    } else {
      errorMessage.value = error.message || error.description || '登录失败'
    }
  } finally {
    loading.value = false
  }
}

/**
 * 密码短语登录（可签名模式）
 *
 * 对标 nrs.login.js:292 的 NRS.login(true, id)：
 *   1. store.login 在本地派生 publicKey/accountId/accountRS（secretPhrase 不出客户端）
 *   2. 调 getAccountPublicKey 校验账户未被占用（error_account_taken）
 *   3. 拉取链上信息（余额、名称等）
 *   4. 评估密码强度并返回警告（passwordNotice）
 *
 * 注意：secretPhrase 仅内存暂存，rememberMe 只存 accountRS（阶段 0.5 安全要求）。
 */
async function handlePasswordLogin(): Promise<void> {
  if (!secretPhrase.value || secretPhrase.value.trim() === '') {
    errorMessage.value = '密码短语不能为空'
    return
  }

  // 对标 nrs.login.js:306：非测试网下密码短语长度 < 12 拒绝登录
  // 注意：12 词助记词长度通常 >= 60，此处只做最低长度校验
  const trimmedPhrase = secretPhrase.value.trim()
  if (trimmedPhrase.length < 12) {
    errorMessage.value = '密码短语过短（最低 12 字符）'
    return
  }

  try {
    loading.value = true
    errorMessage.value = ''
    passwordWarning.value = ''

    console.log('[login] 密码短语登录（本地派生）')

    const result = await accountStore.login(trimmedPhrase, {
      rememberMe: rememberMe.value
    })

    // 显示密码强度警告（对标 nrs.login.js:415-425）
    if (result.warning) {
      passwordWarning.value = result.warning
      ElMessage.warning(result.warning)
    }

    ElMessage.success(`欢迎！已登录 ${accountStore.accountRS}`)

    // 清空密码短语输入框（避免残留）
    secretPhrase.value = ''

    const redirect = route.query.redirect as string
    router.push(redirect || '/')
  } catch (error: any) {
    console.error('[login] 密码短语登录失败:', error)
    if (error.message?.includes('账户已被其他密码短语占用')) {
      errorMessage.value = '该账户已被其他密码短语占用（error_account_taken），请检查您的密码短语'
    } else if (error.code === 5) {
      errorMessage.value = '账户查询失败，请稍后重试'
    } else {
      errorMessage.value = error.message || error.description || '登录失败'
    }
  } finally {
    loading.value = false
  }
}

/** 跳转到注册页 */
function goToRegister(): void {
  router.push('/register')
}

onMounted(() => {
  // 从 store 加载已保存账户列表
  accountStore.listSavedAccounts()

  // 恢复 rememberMe 勾选状态（若上次登录时记住过账户）
  const remembered = localStorage.getItem('nrcs-remember-me')
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

.login-warning {
  display: flex;
  align-items: center;
  gap: $space-sm;
  margin-bottom: $space-lg;
  padding: $space-md;
  background: $warning-subtle;
  border: 1px solid rgba($warning, 0.25);
  border-radius: $radius-md;
  color: $warning;
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

  .form-hint {
    margin-top: $space-sm;
    font-size: $font-size-xs;
    color: $text-muted;
    line-height: 1.4;
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

  .strength-indicator {
    display: flex;
    align-items: center;
    gap: $space-sm;
    margin-top: $space-sm;

    .strength-bar {
      flex: 1;
      height: 4px;
      background: rgba($bg, 0.6);
      border-radius: 2px;
      overflow: hidden;

      .strength-fill {
        height: 100%;
        border-radius: 2px;
        transition: width $duration-normal $ease-out;

        &.strong {
          background: $success;
        }

        &.medium {
          background: $warning;
        }

        &.weak {
          background: $danger-muted;
        }

        &.very_weak {
          background: $danger;
        }
      }
    }

    .strength-label {
      font-size: $font-size-xs;
      white-space: nowrap;

      &.strong {
        color: $success;
      }

      &.medium {
        color: $warning;
      }

      &.weak {
        color: $danger-muted;
      }

      &.very_weak {
        color: $danger;
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
