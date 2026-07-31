<template>
  <el-dialog
    v-model="visible"
    :title="t('token.title')"
    width="640px"
    :close-on-click-modal="false"
    destroy-on-close
    class="token-modal"
    @close="handleClose"
  >
    <el-tabs v-model="activeTab" class="token-modal-tabs">
      <!-- 生成 Token 标签页（对标 nrs.modals.token.js:93-124 generate_token_button） -->
      <el-tab-pane :label="t('token.generate')" name="generate">
        <el-form :model="generateForm" label-position="top">
          <el-form-item :label="t('token.website')" required>
            <el-input
              v-model="generateForm.website"
              :placeholder="t('token.websitePlaceholder')"
              clearable
            />
          </el-form-item>

          <el-form-item :label="t('token.secretPhrase')" required>
            <el-input
              v-model="generateForm.secretPhrase"
              type="password"
              :placeholder="t('token.secretPhrasePlaceholder')"
              show-password
              clearable
            >
              <template #prefix>
                <el-icon><Lock /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-button type="primary" @click="onGenerateToken">
            {{ t('token.generateButton') }}
          </el-button>

          <!-- 生成结果（对标 :119-122） -->
          <el-alert
            v-if="generateResult.error"
            :title="generateResult.error"
            type="error"
            :closable="false"
            show-icon
            class="token-modal-output"
          />
          <div v-else-if="generateResult.token" class="token-modal-output token-modal-output-success">
            <div class="token-modal-output-label">{{ t('token.generatedToken') }}</div>
            <el-input
              :model-value="generateResult.token"
              type="textarea"
              :rows="3"
              readonly
            />
          </div>
        </el-form>
      </el-tab-pane>

      <!-- 解码 Token 标签页（对标 nrs.modals.token.js:39-62 decodeToken） -->
      <el-tab-pane :label="t('token.decode')" name="decode">
        <el-form :model="decodeForm" label-position="top">
          <el-form-item :label="t('token.website')" required>
            <el-input
              v-model="decodeForm.website"
              :placeholder="t('token.websitePlaceholder')"
              clearable
            />
          </el-form-item>

          <el-form-item :label="t('token.token')" required>
            <el-input
              v-model="decodeForm.token"
              :placeholder="t('token.tokenPlaceholder')"
              type="textarea"
              :rows="3"
              clearable
            />
          </el-form-item>

          <el-button type="primary" :loading="decoding" @click="onDecodeToken">
            {{ t('token.validateButton') }}
          </el-button>

          <!-- 解码结果（对标 :48-62） -->
          <el-alert
            v-if="decodeResult.error"
            :title="decodeResult.error"
            type="error"
            :closable="false"
            show-icon
            class="token-modal-output"
          />
          <el-alert
            v-else-if="decodeResult.valid === true"
            :title="t('token.validToken', { account: decodeResult.account, timestamp: decodeResult.timestamp })"
            type="success"
            :closable="false"
            show-icon
            class="token-modal-output"
          />
          <el-alert
            v-else-if="decodeResult.valid === false"
            :title="t('token.invalidToken', { account: decodeResult.account, timestamp: decodeResult.timestamp })"
            type="warning"
            :closable="false"
            show-icon
            class="token-modal-output"
          />
        </el-form>
      </el-tab-pane>
    </el-tabs>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * TokenModal 组件 —— 令牌生成与验证弹窗。
 *
 * 对标 nrs.modals.token.js（131 行）的完整实现。
 *
 * 主要功能：
 *   1. 生成 Token（对标 :93-124 generate_token_button）
 *      - 输入 website + secretPhrase
 *      - 本地调用 generateToken(website, secretPhrase)（对标 NRS.generateToken）
 *      - 展示生成的 token 文本
 *   2. 解码 Token（对标 :39-62 decodeToken）
 *      - 输入 website + token
 *      - 调 decodeToken API 验证
 *      - 展示验证结果（valid/invalid + account + timestamp）
 *
 * 安全模型：secretPhrase 仅用于本地生成 token，不出客户端。
 *
 * 对标参考：NRS.forms.decodeToken / generate_token_button click handler
 */
import { ref, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { Lock } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { generateToken } from '@/utils/nrcs-crypto'
import { formatTimestamp } from '@/utils/format'

const { t } = useI18n()

const visible = defineModel<boolean>('visible', { default: false })

const activeTab = ref<'generate' | 'decode'>('generate')

/** 生成 Token 表单 */
const generateForm = reactive({
  website: '',
  secretPhrase: '',
})

/** 解码 Token 表单 */
const decodeForm = reactive({
  website: '',
  token: '',
})

/** 生成结果 */
const generateResult = reactive<{
  token: string
  error: string
}>({
  token: '',
  error: '',
})

/** 解码结果 */
const decodeResult = reactive<{
  valid: boolean | null
  account: string
  timestamp: string
  error: string
}>({
  valid: null,
  account: '',
  timestamp: '',
  error: '',
})

const decoding = ref(false)

// ----------------------------------------------------------------
// 生成 Token（对标 nrs.modals.token.js:93-124）
// ----------------------------------------------------------------

/**
 * 生成 Token。
 *
 * 步骤：
 *   1. 校验 website 非空
 *   2. 调 generateToken(website, secretPhrase) 本地生成
 *   3. 展示生成的 token
 *
 * 对标 NRS.generateToken(website, secretPhrase)。
 */
function onGenerateToken(): void {
  generateResult.token = ''
  generateResult.error = ''

  if (!generateForm.website) {
    generateResult.error = t('token.websiteRequired')
    return
  }
  if (!generateForm.secretPhrase) {
    generateResult.error = t('token.secretPhraseRequired')
    return
  }

  try {
    const token = generateToken(generateForm.website, generateForm.secretPhrase)
    generateResult.token = token
  } catch (e) {
    generateResult.error = e instanceof Error ? e.message : String(e)
  }
}

// ----------------------------------------------------------------
// 解码 Token（对标 nrs.modals.token.js:39-62）
// ----------------------------------------------------------------

/**
 * 解码并验证 Token。
 *
 * 调用 decodeToken API 验证 token 是否有效，
 * 返回 valid/account/timestamp 等信息。
 *
 * 对标 NRS.forms.decodeTokenComplete。
 */
async function onDecodeToken(): Promise<void> {
  decodeResult.valid = null
  decodeResult.account = ''
  decodeResult.timestamp = ''
  decodeResult.error = ''

  if (!decodeForm.website) {
    decodeResult.error = t('token.websiteRequired')
    return
  }
  if (!decodeForm.token) {
    decodeResult.error = t('token.tokenRequired')
    return
  }

  decoding.value = true
  try {
    const resp: any = await nrcsApi.decodeToken(decodeForm.website, decodeForm.token)
    if (!resp || (resp as any).errorCode) {
      decodeResult.error = (resp as any)?.errorDescription || t('token.decodeFailed')
    } else {
      decodeResult.valid = !!resp.valid
      decodeResult.account = resp.accountRS || resp.account || ''
      decodeResult.timestamp = resp.timestamp ? formatTimestamp(resp.timestamp) : ''
    }
  } catch (e) {
    decodeResult.error = e instanceof Error ? e.message : String(e)
  } finally {
    decoding.value = false
  }
}

// ----------------------------------------------------------------
// 关闭/重置
// ----------------------------------------------------------------

/**
 * 关闭 modal 并清空所有状态。
 */
function handleClose(): void {
  visible.value = false
  activeTab.value = 'generate'
  generateForm.website = ''
  generateForm.secretPhrase = ''
  decodeForm.website = ''
  decodeForm.token = ''
  generateResult.token = ''
  generateResult.error = ''
  decodeResult.valid = null
  decodeResult.account = ''
  decodeResult.timestamp = ''
  decodeResult.error = ''
}
</script>

<style scoped lang="scss">
.token-modal-tabs {
  margin-bottom: 12px;
}
.token-modal-output {
  margin-top: 16px;
}
.token-modal-output-success {
  padding: 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 4px;
}
.token-modal-output-label {
  margin-bottom: 8px;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
