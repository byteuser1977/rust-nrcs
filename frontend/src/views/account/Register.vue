<template>
  <div class="register-page">
    <div class="register-background">
      <div class="bg-gradient"></div>
      <div class="bg-grid"></div>
      <div class="bg-glow bg-glow--1"></div>
      <div class="bg-glow bg-glow--2"></div>
    </div>

    <div class="register-container">
      <div class="register-branding">
        <h1 class="brand-name">NRCS</h1>
        <p class="brand-tagline">创建新账户</p>
      </div>

      <!-- Error Message -->
      <div v-if="errorMessage" class="register-error">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="#ef4444" stroke-width="2"/>
          <line x1="12" y1="8" x2="12" y2="12" stroke="#ef4444" stroke-width="2" stroke-linecap="round"/>
          <circle cx="12" cy="16" r="1" fill="#ef4444"/>
        </svg>
        <span>{{ errorMessage }}</span>
      </div>

      <div class="register-card">
        <!-- Stepper -->
        <div class="stepper">
          <div
            v-for="(step, index) in stepLabels"
            :key="index"
            class="step"
            :class="{
              'is-active': currentStep === index,
              'is-completed': currentStep > index
            }"
          >
            <div class="step-circle">
              <svg v-if="currentStep > index" width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M20 6L9 17l-5-5" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              <span v-else>{{ index + 1 }}</span>
            </div>
            <span class="step-label">{{ step }}</span>
            <div v-if="index < stepLabels.length - 1" class="step-connector"></div>
          </div>
        </div>

        <!-- Step 0: Choose registration method -->
        <div v-if="currentStep === 0" class="step-content">
          <h2 class="step-title">选择注册方式</h2>
          <p class="step-desc">NRCS 账户通过密码短语（助记词）管理，无中心化注册。请选择适合您的方式：</p>

          <div class="method-options">
            <button
              class="method-card"
              :class="{ 'is-selected': method === 'generated' }"
              @click="selectMethod('generated')"
            >
              <div class="method-icon">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none">
                  <path d="M21 2v6h-6M3 12a9 9 0 0115-6.7L21 8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                  <path d="M3 22v-6h6M21 12a9 9 0 01-15 6.7L3 16" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </div>
              <h3 class="method-title">随机生成助记词</h3>
              <p class="method-desc">系统为您生成 12 词安全助记词（推荐）</p>
              <p class="method-hint">128 位熵 · 银行级安全</p>
            </button>

            <button
              class="method-card"
              :class="{ 'is-selected': method === 'custom' }"
              @click="selectMethod('custom')"
            >
              <div class="method-icon">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none">
                  <rect x="4" y="8" width="16" height="12" rx="2" stroke="currentColor" stroke-width="1.8"/>
                  <path d="M8 8V6a4 4 0 118 0v2" stroke="currentColor" stroke-width="1.8"/>
                  <line x1="12" y1="13" x2="12" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                </svg>
              </div>
              <h3 class="method-title">自定义密码短语</h3>
              <p class="method-desc">自行输入密码短语（≥35 字符）</p>
              <p class="method-hint">需含大写字母或数字</p>
            </button>
          </div>

          <div class="step-actions">
            <button class="btn-secondary" @click="goToLogin">取消</button>
            <button class="btn-primary" :disabled="!method" @click="goToStep(1)">下一步</button>
          </div>
        </div>

        <!-- Step 1: Generated - Display mnemonic / Custom - Input passphrase -->
        <div v-else-if="currentStep === 1" class="step-content">
          <!-- Generated Mnemonic Display -->
          <template v-if="method === 'generated'">
            <h2 class="step-title">备份您的助记词</h2>
            <div class="alert alert-warning">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
                <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <line x1="12" y1="9" x2="12" y2="13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                <circle cx="12" cy="17" r="1" fill="currentColor"/>
              </svg>
              <span>这是您账户的唯一凭证，请务必离线保存！切勿截图或通过网络传输。</span>
            </div>

            <div class="mnemonic-grid">
              <div v-for="(word, index) in generatedWords" :key="index" class="mnemonic-item">
                <span class="word-index">{{ index + 1 }}</span>
                <span class="word-text">{{ word }}</span>
              </div>
            </div>

            <div class="mnemonic-actions">
              <button class="btn-ghost" @click="copyMnemonic">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <rect x="9" y="9" width="13" height="13" rx="2" stroke="currentColor" stroke-width="1.8"/>
                  <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" stroke="currentColor" stroke-width="1.8"/>
                </svg>
                复制助记词
              </button>
              <button class="btn-ghost" @click="regenerateMnemonic">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <path d="M21 2v6h-6M3 12a9 9 0 0115-6.7L21 8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                  <path d="M3 22v-6h6M21 12a9 9 0 01-15 6.7L3 16" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
                重新生成
              </button>
            </div>

            <label class="checkbox-wrapper">
              <input type="checkbox" v-model="backupConfirmed" class="checkbox-input" />
              <span class="checkbox-custom"></span>
              <span class="checkbox-label">我已安全备份助记词</span>
            </label>
          </template>

          <!-- Custom Passphrase Input -->
          <template v-else>
            <h2 class="step-title">设置密码短语</h2>
            <p class="step-desc">密码短语是您账户的唯一凭证，请妥善保管。建议使用 12 词助记词或 ≥35 字符的强密码。</p>

            <div class="form-group">
              <label class="form-label">密码短语</label>
              <div class="input-wrapper">
                <input
                  v-model="customPassphrase"
                  :type="showPassword ? 'text' : 'password'"
                  class="form-input"
                  placeholder="输入密码短语（≥35 字符）..."
                />
                <button type="button" class="toggle-password" @click="showPassword = !showPassword">
                  <svg v-if="!showPassword" width="18" height="18" viewBox="0 0 24 24" fill="none">
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8S1 12 1 12z" stroke="currentColor" stroke-width="1.8"/>
                    <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.8"/>
                  </svg>
                  <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none">
                    <path d="M17.94 17.94A10.07 10.07 0 0112 20c-7 0-11-8-11-8a18.45 18.45 0 015.06-5.94M9.9 4.24A9.12 9.12 0 0112 4c7 0 11 8 11 8a18.5 18.5 0 01-2.16 3.19m-6.72-1.07a3 3 0 11-4.24-4.24" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
                    <line x1="1" y1="1" x2="23" y2="23" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
                  </svg>
                </button>
              </div>
              <div v-if="customPassphrase" class="strength-indicator">
                <div class="strength-bar">
                  <div class="strength-fill" :class="customStrength.level" :style="{ width: customStrength.score + '%' }"></div>
                </div>
                <span class="strength-label" :class="customStrength.level">{{ customStrengthText }}</span>
              </div>
            </div>

            <div class="form-group">
              <label class="form-label">确认密码短语</label>
              <div class="input-wrapper">
                <input
                  v-model="customPassphraseRepeat"
                  :type="showPassword ? 'text' : 'password'"
                  class="form-input"
                  placeholder="再次输入密码短语..."
                />
              </div>
              <p v-if="customPassphraseRepeat && customPassphrase !== customPassphraseRepeat" class="field-error">
                两次输入不一致
              </p>
            </div>
          </template>

          <div class="step-actions">
            <button class="btn-secondary" @click="goToStep(0)">上一步</button>
            <button
              v-if="method === 'generated'"
              class="btn-primary"
              :disabled="!backupConfirmed"
              @click="goToStep(2)"
            >
              下一步
            </button>
            <button
              v-else
              class="btn-primary"
              :disabled="!isCustomValid"
              @click="completeCustomRegistration"
            >
              完成注册
            </button>
          </div>
        </div>

        <!-- Step 2: Verify mnemonic (generated only) -->
        <div v-else-if="currentStep === 2" class="step-content">
          <h2 class="step-title">确认助记词</h2>
          <p class="step-desc">请按顺序输入您刚才备份的 12 词助记词，以确认备份正确。</p>

          <div class="verify-input">
            <textarea
              v-model="verifyInput"
              class="verify-textarea"
              rows="3"
              placeholder="按顺序输入 12 词助记词，用空格分隔..."
            ></textarea>
          </div>

          <div v-if="verifyError" class="alert alert-error">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
              <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <circle cx="12" cy="16" r="1" fill="currentColor"/>
            </svg>
            <span>{{ verifyError }}</span>
          </div>

          <div class="step-actions">
            <button class="btn-secondary" @click="goToStep(1)">上一步</button>
            <button class="btn-primary" :disabled="loading" @click="completeGeneratedRegistration">
              <span v-if="loading" class="spinner"></span>
              <span v-else>完成注册</span>
            </button>
          </div>
        </div>

        <!-- Step 3: Success -->
        <div v-else-if="currentStep === 3" class="step-content step-success">
          <div class="success-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none">
              <path d="M22 11.08V12a10 10 0 11-5.93-9.14" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M22 4L12 14.01l-3-3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <h2 class="step-title">账户创建成功！</h2>
          <p class="step-desc">欢迎加入 NRCS 网络</p>

          <div class="account-info">
            <div class="info-row">
              <span class="info-label">账户地址</span>
              <span class="info-value">{{ accountStore.accountRS }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">账户 ID</span>
              <span class="info-value">{{ accountStore.accountId }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">公钥</span>
              <span class="info-value mono">{{ accountStore.publicKey.substring(0, 32) }}...</span>
            </div>
          </div>

          <div class="step-actions">
            <button class="btn-primary btn-full" @click="goToDashboard">进入控制台</button>
          </div>
        </div>
      </div>

      <div class="register-footer">
        <p>已有账户？<a href="#" @click.prevent="goToLogin">立即登录</a></p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useAccountStore } from '@/stores/modules'
import {
  generateMnemonic,
  verifyPassphraseMatch,
  checkPassphraseStrength,
  validateMnemonic,
} from '@/utils/mnemonic'

const router = useRouter()
const accountStore = useAccountStore()

/** 注册方式：'generated'（随机生成）/ 'custom'（自定义密码短语） */
const method = ref<'generated' | 'custom' | ''>('')

/** 当前步骤：0 选择方式 / 1 备份或输入 / 2 验证助记词 / 3 成功 */
const currentStep = ref(0)

const stepLabels = computed(() => {
  if (method.value === 'custom') {
    return ['选择方式', '设置密码', '完成']
  }
  return ['选择方式', '备份助记词', '确认助记词', '完成']
})

// Step 1 - Generated
const generatedWords = ref<string[]>([])
const backupConfirmed = ref(false)

// Step 1 - Custom
const customPassphrase = ref('')
const customPassphraseRepeat = ref('')
const showPassword = ref(false)

// Step 2 - Verify
const verifyInput = ref('')
const verifyError = ref('')

// Common
const loading = ref(false)
const errorMessage = ref('')

/** 自定义密码短语强度评估 */
const customStrength = computed(() => checkPassphraseStrength(customPassphrase.value))

const customStrengthText = computed(() => {
  switch (customStrength.value.level) {
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

/** 自定义密码短语是否通过校验（对标 nrs.login.js:152-158） */
const isCustomValid = computed(() => {
  const p = customPassphrase.value
  const r = customPassphraseRepeat.value
  if (!p || !r) return false
  if (p !== r) return false
  // 对标 nrs.login.js:152-155：长度 < 35 或 < 50 且无大写/数字 → 不通过
  if (p.length < 35) return false
  if (p.length < 50 && (!/[A-Z]/.test(p) || !/[0-9]/.test(p))) return false
  return true
})

/** 选择注册方式 */
function selectMethod(m: 'generated' | 'custom'): void {
  method.value = m
  errorMessage.value = ''
}

/** 跳转到指定步骤 */
function goToStep(step: number): void {
  errorMessage.value = ''

  // 进入步骤 1 时按需初始化
  if (step === 1 && method.value === 'generated' && generatedWords.value.length === 0) {
    regenerateMnemonic()
  }

  currentStep.value = step
}

/** 重新生成助记词（对标 nrs.login.js:120 PassPhraseGenerator.generatePassPhrase） */
function regenerateMnemonic(): void {
  const result = generateMnemonic()
  generatedWords.value = result.words
  backupConfirmed.value = false
  console.log('[register] 已生成新助记词（仅显示，不外发）')
}

/** 复制助记词到剪贴板 */
async function copyMnemonic(): Promise<void> {
  try {
    const mnemonic = generatedWords.value.join(' ')
    await navigator.clipboard.writeText(mnemonic)
    ElMessage.warning('已复制！请尽快离线保存，避免剪贴板泄露')
  } catch {
    ElMessage.error('无法访问剪贴板，请手动抄写')
  }
}

/**
 * 完成自定义密码短语注册
 *
 * 对标 nrs.login.js:164 NRS.login(true, password)：
 * 直接用密码短语登录（本地派生账户），无需服务端注册接口。
 */
async function completeCustomRegistration(): Promise<void> {
  if (!isCustomValid.value) {
    errorMessage.value = '密码短语不符合安全要求'
    return
  }

  try {
    loading.value = true
    errorMessage.value = ''

    // 对标 nrs.login.js:492 newlyCreatedAccount = true → 自动 rememberAccount
    await accountStore.login(customPassphrase.value, { rememberMe: true })

    // 清空敏感输入
    customPassphrase.value = ''
    customPassphraseRepeat.value = ''

    currentStep.value = 3
    ElMessage.success('账户创建成功！')
  } catch (error: any) {
    console.error('[register] 自定义注册失败:', error)
    errorMessage.value = error.message || error.description || '注册失败'
  } finally {
    loading.value = false
  }
}

/**
 * 完成随机助记词注册（验证用户输入与原始助记词匹配后登录）
 *
 * 对标 nrs.login.js:129-142 verifyGeneratedPassphrase：
 *   - 用户输入与 PassPhraseGenerator.passPhrase 比对
 *   - 匹配 → newlyCreatedAccount = true; NRS.login(true, password)
 *   - 不匹配 → 显示错误
 */
async function completeGeneratedRegistration(): Promise<void> {
  const input = verifyInput.value.trim()

  if (!input) {
    verifyError.value = '请输入助记词'
    return
  }

  const original = generatedWords.value.join(' ')

  // 对标 nrs.login.js:133：与原始助记词比对
  if (!verifyPassphraseMatch(input, original)) {
    verifyError.value = '助记词不匹配，请检查顺序与拼写'
    return
  }

  // 额外校验：必须是有效的 12 词助记词
  const validation = validateMnemonic(input)
  if (!validation.valid) {
    verifyError.value = validation.errors[0] || '助记词无效'
    return
  }

  try {
    loading.value = true
    verifyError.value = ''

    // 对标 nrs.login.js:136-137：登录并标记为新账户
    await accountStore.login(original, { rememberMe: true })

    // 清空敏感数据
    verifyInput.value = ''
    generatedWords.value = []

    currentStep.value = 3
    ElMessage.success('账户创建成功！')
  } catch (error: any) {
    console.error('[register] 助记词注册失败:', error)
    if (error.message?.includes('账户已被其他密码短语占用')) {
      verifyError.value = '该助记词对应的账户已被占用，请重新生成'
    } else {
      verifyError.value = error.message || error.description || '注册失败'
    }
  } finally {
    loading.value = false
  }
}

/** 跳转到登录页 */
function goToLogin(): void {
  router.push('/login')
}

/** 跳转到控制台 */
function goToDashboard(): void {
  router.push('/')
}
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables' as *;

.register-page {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: $bg;
  overflow: hidden;
  font-family: $font-body;
}

.register-background {
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

.register-container {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 540px;
  padding: $space-lg;
}

.register-branding {
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
  }
}

.register-error {
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

  svg { flex-shrink: 0; }
  span { line-height: 1.5; }
}

.register-card {
  background: linear-gradient(135deg, rgba($card, 0.9), rgba($panel-strong, 0.95));
  backdrop-filter: blur(20px);
  border-radius: $radius-xl;
  border: 1px solid $border-default;
  padding: $space-xl;
  box-shadow: $shadow-lg;
}

.stepper {
  display: flex;
  align-items: center;
  margin-bottom: $space-xl;
  padding: 0 $space-sm;

  .step {
    display: flex;
    align-items: center;
    flex: 1;
    position: relative;

    &:last-child {
      flex: 0;
    }

    .step-circle {
      width: 32px;
      height: 32px;
      border-radius: 50%;
      background: rgba($bg, 0.6);
      border: 2px solid $border-default;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: $font-size-sm;
      font-weight: 600;
      color: $text-secondary;
      transition: all $duration-normal $ease-out;
      flex-shrink: 0;
    }

    .step-label {
      margin-left: $space-sm;
      font-size: $font-size-xs;
      color: $text-secondary;
      white-space: nowrap;
    }

    .step-connector {
      flex: 1;
      height: 2px;
      background: $border-default;
      margin: 0 $space-sm;
      transition: background $duration-normal;
    }

    &.is-active {
      .step-circle {
        background: linear-gradient(135deg, $primary, $primary-dark);
        border-color: $primary;
        color: $primary-foreground;
        box-shadow: 0 0 12px $primary-glow;
      }

      .step-label {
        color: $text-primary;
        font-weight: 500;
      }
    }

    &.is-completed {
      .step-circle {
        background: $success;
        border-color: $success;
        color: #fff;
      }

      .step-connector {
        background: $success;
      }
    }
  }
}

.step-content {
  min-height: 280px;

  &.step-success {
    text-align: center;
  }
}

.step-title {
  font-size: $font-size-xl;
  font-weight: 600;
  color: $text-primary;
  margin: 0 0 $space-sm;
}

.step-desc {
  font-size: $font-size-sm;
  color: $text-secondary;
  line-height: 1.6;
  margin: 0 0 $space-lg;
}

.method-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: $space-md;
  margin-bottom: $space-xl;

  .method-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: $space-lg $space-md;
    background: rgba($bg, 0.6);
    border: 1.5px solid $border-default;
    border-radius: $radius-md;
    cursor: pointer;
    transition: all $duration-normal $ease-out;
    text-align: center;

    &:hover {
      border-color: $border-hover;
      background: rgba($bg, 0.8);
      transform: translateY(-2px);
    }

    &.is-selected {
      background: linear-gradient(135deg, rgba($primary, 0.12), rgba($primary-dark, 0.06));
      border-color: $primary;
      box-shadow: 0 0 20px $primary-glow;
    }

    .method-icon {
      color: $primary;
      margin-bottom: $space-sm;
    }

    .method-title {
      font-size: $font-size-base;
      font-weight: 600;
      color: $text-primary;
      margin: 0 0 $space-xs;
    }

    .method-desc {
      font-size: $font-size-sm;
      color: $text-secondary;
      margin: 0 0 $space-xs;
    }

    .method-hint {
      font-size: $font-size-xs;
      color: $text-muted;
      margin: 0;
    }
  }
}

.alert {
  display: flex;
  align-items: flex-start;
  gap: $space-sm;
  padding: $space-md;
  border-radius: $radius-md;
  font-size: $font-size-sm;
  margin-bottom: $space-lg;

  svg { flex-shrink: 0; margin-top: 2px; }

  &.alert-warning {
    background: $warning-subtle;
    border: 1px solid rgba($warning, 0.25);
    color: $warning;
  }

  &.alert-error {
    background: $danger-subtle;
    border: 1px solid rgba($danger, 0.25);
    color: $danger;
  }
}

.mnemonic-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: $space-sm;
  margin-bottom: $space-lg;

  .mnemonic-item {
    display: flex;
    align-items: center;
    gap: $space-sm;
    padding: $space-sm $space-md;
    background: rgba($bg, 0.6);
    border: 1px solid $border-default;
    border-radius: $radius-sm;

    .word-index {
      font-size: $font-size-xs;
      color: $text-muted;
      min-width: 18px;
    }

    .word-text {
      font-size: $font-size-sm;
      color: $text-primary;
      font-weight: 500;
      font-family: 'Courier New', monospace;
    }
  }
}

.mnemonic-actions {
  display: flex;
  gap: $space-sm;
  margin-bottom: $space-lg;

  .btn-ghost {
    display: flex;
    align-items: center;
    gap: $space-xs;
    padding: $space-sm $space-md;
    background: transparent;
    border: 1px solid $border-default;
    border-radius: $radius-sm;
    color: $text-secondary;
    font-size: $font-size-sm;
    cursor: pointer;
    transition: all $duration-fast;

    &:hover {
      color: $primary;
      border-color: $primary;
      background: $primary-subtle;
    }
  }
}

.form-group {
  margin-bottom: $space-lg;

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

    .form-input {
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
      font-family: inherit;

      &::placeholder { color: $text-muted; }

      &:focus {
        border-color: $primary;
        background: rgba($bg, 0.9);
        box-shadow: $focus-ring;
      }
    }

    .toggle-password {
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
  }

  .field-error {
    margin-top: $space-xs;
    font-size: $font-size-xs;
    color: $danger;
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

      &.strong { background: $success; }
      &.medium { background: $warning; }
      &.weak { background: $danger-muted; }
      &.very_weak { background: $danger; }
    }
  }

  .strength-label {
    font-size: $font-size-xs;
    white-space: nowrap;

    &.strong { color: $success; }
    &.medium { color: $warning; }
    &.weak { color: $danger-muted; }
    &.very_weak { color: $danger; }
  }
}

.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: $space-sm;
  cursor: pointer;
  user-select: none;
  margin-top: $space-md;

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
  }
}

.verify-input {
  margin-bottom: $space-lg;

  .verify-textarea {
    width: 100%;
    padding: $space-md;
    background: rgba($bg, 0.6);
    border: 1.5px solid $border-default;
    border-radius: $radius-md;
    color: $text-primary;
    font-size: $font-size-base;
    font-family: 'Courier New', monospace;
    outline: none;
    transition: all $duration-normal $ease-out;
    resize: vertical;
    min-height: 80px;

    &::placeholder { color: $text-muted; }

    &:focus {
      border-color: $primary;
      background: rgba($bg, 0.9);
      box-shadow: $focus-ring;
    }
  }
}

.step-success {
  .success-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: $success-subtle;
    color: $success;
    margin-bottom: $space-lg;
  }
}

.account-info {
  background: rgba($bg, 0.6);
  border: 1px solid $border-default;
  border-radius: $radius-md;
  padding: $space-md;
  margin-bottom: $space-xl;
  text-align: left;

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: $space-sm 0;
    border-bottom: 1px solid $border-default;

    &:last-child { border-bottom: none; }

    .info-label {
      font-size: $font-size-sm;
      color: $text-secondary;
    }

    .info-value {
      font-size: $font-size-sm;
      color: $text-primary;
      font-weight: 500;
      word-break: break-all;

      &.mono {
        font-family: 'Courier New', monospace;
      }
    }
  }
}

.step-actions {
  display: flex;
  gap: $space-md;
  margin-top: $space-lg;

  .btn-full {
    width: 100%;
  }
}

.btn-primary {
  position: relative;
  height: 44px;
  flex: 1;
  background: linear-gradient(135deg, $primary, $primary-dark);
  border: none;
  border-radius: $radius-md;
  color: $primary-foreground;
  font-size: $font-size-base;
  font-weight: 600;
  cursor: pointer;
  transition: all $duration-normal $ease-out;
  box-shadow: 0 4px 16px rgba($primary, 0.35);
  display: inline-flex;
  align-items: center;
  justify-content: center;

  &:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba($primary, 0.45);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .spinner {
    display: inline-block;
    width: 18px;
    height: 18px;
    border: 2.5px solid rgba($primary-foreground, 0.2);
    border-top-color: $primary-foreground;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
}

.btn-secondary {
  height: 44px;
  flex: 1;
  background: rgba($bg, 0.6);
  border: 1.5px solid $border-default;
  border-radius: $radius-md;
  color: $text-secondary;
  font-size: $font-size-base;
  font-weight: 500;
  cursor: pointer;
  transition: all $duration-normal $ease-out;

  &:hover {
    border-color: $border-hover;
    color: $text-primary;
    background: rgba($bg, 0.8);
  }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.register-footer {
  text-align: center;
  margin-top: $space-xl;

  p {
    font-size: $font-size-sm;
    color: $text-muted;
    margin: 0;
  }

  a {
    color: $primary;
    text-decoration: none;
    margin-left: $space-xs;

    &:hover {
      text-decoration: underline;
    }
  }
}

@media (max-width: 540px) {
  .register-container {
    max-width: 100%;
    padding: $space-md;
  }

  .method-options {
    grid-template-columns: 1fr;
  }

  .mnemonic-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .stepper {
    .step-label {
      display: none;
    }
  }
}
</style>
