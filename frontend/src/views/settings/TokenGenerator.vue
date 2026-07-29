<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Key /></el-icon> {{ t('settings.tokenGenerator') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-tabs v-model="mode">
        <el-tab-pane :label="t('settings.generate')" name="generate">
          <el-form label-width="140px" style="max-width: 520px">
            <el-form-item :label="t('settings.website')">
              <el-input v-model="website" placeholder="https://example.com" />
            </el-form-item>
            <el-form-item :label="t('common.secretPhrase')">
              <el-input v-model="secretPhrase" type="password" show-password />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="generate" :loading="generating">
                {{ t('settings.generate') }}
              </el-button>
            </el-form-item>
          </el-form>
          <div v-if="tokenResult" style="margin-top: 16px">
            <el-form-item :label="t('settings.generatedToken')" label-width="140px">
              <div class="token-display">
                <code class="token-code">{{ tokenResult }}</code>
                <el-button size="small" type="primary" plain @click="copyToken">
                  <el-icon><CopyDocument /></el-icon> {{ t('common.copy') }}
                </el-button>
              </div>
            </el-form-item>
          </div>
        </el-tab-pane>
        <el-tab-pane :label="t('settings.decodeToken')" name="decode">
          <el-form label-width="140px" style="max-width: 520px">
            <el-form-item :label="t('settings.token')">
              <el-input v-model="tokenToDecode" type="textarea" :rows="3" placeholder="Paste token string to decode" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="decode" :loading="decoding">
                {{ t('settings.decode') }}
              </el-button>
            </el-form-item>
          </el-form>
          <el-descriptions v-if="decoded" :column="2" border style="margin-top: 16px">
            <el-descriptions-item :label="t('common.account')">
              {{ decoded.accountRS || decoded.account }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('settings.website')">
              {{ decoded.website || '-' }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.timestamp')">
              {{ formatTimestamp(decoded.timestamp) }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('settings.valid')">
              <el-tag :type="decoded.valid ? 'success' : 'danger'" size="small">
                {{ decoded.valid ? t('common.yes') : t('common.no') }}
              </el-tag>
            </el-descriptions-item>
          </el-descriptions>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp } from '@/utils/format'

const { t } = useI18n()

const mode = ref('generate')
const website = ref('')
const secretPhrase = ref('')
const tokenResult = ref('')
const generating = ref(false)

const tokenToDecode = ref('')
const decoded = ref<any>(null)
const decoding = ref(false)

async function generate() {
  if (!website.value || !secretPhrase.value) {
    ElMessage.warning(t('validation.required'))
    return
  }
  generating.value = true
  try {
    const result = await nrcsApi.generateToken({
      secretPhrase: secretPhrase.value,
      website: website.value,
    })
    tokenResult.value = result?.token || ''
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    generating.value = false
  }
}

async function copyToken() {
  try {
    await navigator.clipboard.writeText(tokenResult.value)
    ElMessage.success(t('common.copied'))
  } catch {
    ElMessage.error(t('common.copyFailed'))
  }
}

async function decode() {
  if (!tokenToDecode.value.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  decoding.value = true
  try {
    decoded.value = await nrcsApi.decodeToken(tokenToDecode.value.trim())
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    decoding.value = false
  }
}
</script>

<style scoped lang="scss">
.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 0;
    }
  }
}
.token-display {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.token-code {
  display: block;
  padding: 12px;
  background: #f5f7fa;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  font-family: 'Roboto Mono', monospace;
  font-size: 13px;
  word-break: break-all;
  color: #303133;
  line-height: 1.5;
}
</style>
