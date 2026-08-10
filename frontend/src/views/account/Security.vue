<template>
  <div class="security">
    <h1 class="page-title">{{ t('account.security') }}</h1>

    <el-card shadow="never" v-if="isLoggedIn">
      <el-descriptions :column="2" border>
        <el-descriptions-item :label="t('account.sessionStatus')">
          <el-tag type="success" size="small">{{ t('account.loggedIn') }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item :label="t('account.loginType')">
          {{ loginType === 'password' ? t('account.passwordLogin') : t('account.readonlyLogin') }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('account.lockStatus')">
          <el-tag :type="isLocked ? 'warning' : 'success'" size="small">
            {{ isLocked ? t('account.locked') : t('account.unlocked') }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item :label="t('account.secretPhraseStatus')">
          {{ hasSecretPhrase ? t('account.secretPhraseInMemory') : t('account.secretPhraseNotInMemory') }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('account.publicKey')" :span="2">
          <code class="mono">{{ publicKey || '—' }}</code>
        </el-descriptions-item>
      </el-descriptions>

      <el-alert
        type="info"
        :closable="false"
        show-icon
        style="margin-top: 16px"
        :title="t('account.securityTips')"
      />

      <div class="action-row" style="margin-top: 20px">
        <!-- 解锁输入（仅在锁定状态显示） -->
        <template v-if="isLocked">
          <el-input
            v-model="unlockInput"
            type="password"
            show-password
            :placeholder="t('login.secretPhrase')"
            style="max-width: 320px"
            @keyup.enter="handleUnlock"
          />
          <el-button type="primary" @click="handleUnlock">
            <el-icon><Unlock /></el-icon> {{ t('account.unlockSession') }}
          </el-button>
        </template>
        <el-button v-else @click="handleLock" :disabled="!hasSecretPhrase">
          <el-icon><Lock /></el-icon> {{ t('account.lockSession') }}
        </el-button>
        <el-button type="danger" plain @click="handleLogout">
          <el-icon><SwitchButton /></el-icon> {{ t('account.logout') }}
        </el-button>
      </div>
    </el-card>

    <el-empty v-else :description="t('account.notLoggedIn')" />
  </div>
</template>

<script setup lang="ts">
/**
 * 账户安全页面
 *
 * 对标 NRCS 客户端的账户会话安全能力（nrs.login.js showLockscreen/logout）：
 *   - 展示会话状态（登录方式 / 锁定状态 / secretPhrase 是否在内存）
 *   - 锁定会话：隐藏 secretPhrase 进入锁屏（内存保留）
 *   - 解锁会话：验证密码短语后恢复
 *   - 登出：清除当前会话状态
 */
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Lock, SwitchButton, Unlock } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()
const router = useRouter()

const isLoggedIn = computed(() => accountStore.isLoggedIn)
const loginType = computed(() => accountStore.loginType)
const isLocked = computed(() => accountStore.isLocked)
const hasSecretPhrase = computed(() => accountStore.hasSecretPhrase)
const publicKey = computed(() => accountStore.publicKey)

const unlockInput = ref('')

/**
 * 锁定会话（对标 nrs.login.js showLockscreen）
 */
function handleLock() {
  accountStore.lock()
}

/**
 * 解锁会话：验证密码短语后解除锁屏
 */
async function handleUnlock() {
  if (!unlockInput.value.trim()) {
    ElMessage.warning(t('login.secretPhraseRequired'))
    return
  }
  try {
    await accountStore.unlock(unlockInput.value.trim())
    unlockInput.value = ''
    ElMessage.success(t('common.operationSuccess'))
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  }
}

/**
 * 登出并返回首页（对标 nrs.login.js logout）
 */
function handleLogout() {
  accountStore.logout()
  router.push('/')
}
</script>

<style lang="scss" scoped>
.security {
  .page-title {
    margin-bottom: 24px;
    font-size: 24px;
    font-weight: 600;
  }

  .mono {
    font-family: 'Monaco', 'Consolas', monospace;
    word-break: break-all;
  }

  .action-row {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
  }
}
</style>
