<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Key /></el-icon> {{ t('settings.tokenGenerator') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-tabs v-model="mode">
        <el-tab-pane :label="t('settings.generate')" name="generate">
          <el-form label-width="140px" style="max-width:500px">
            <el-form-item :label="t('settings.website')">
              <el-input v-model="website" placeholder="https://example.com" />
            </el-form-item>
            <el-form-item :label="t('common.secretPhrase')">
              <el-input v-model="secretPhrase" type="password" show-password />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="generate">{{ t('settings.generate') }}</el-button>
            </el-form-item>
          </el-form>
          <el-alert v-if="tokenResult" type="success" :closable="false" style="margin-top:16px">
            <template #title><span class="text-mono">{{ tokenResult }}</span></template>
          </el-alert>
        </el-tab-pane>
        <el-tab-pane :label="t('settings.decode')" name="decode">
          <el-form label-width="140px" style="max-width:500px">
            <el-form-item :label="t('settings.token')">
              <el-input v-model="decodeToken" type="textarea" :rows="3" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="decode">{{ t('settings.decode') }}</el-button>
            </el-form-item>
          </el-form>
          <el-descriptions v-if="decoded" :column="2" border style="margin-top:16px">
            <el-descriptions-item :label="t('common.account')">{{ decoded.accountRS }}</el-descriptions-item>
            <el-descriptions-item :label="t('settings.website')">{{ decoded.website }}</el-descriptions-item>
            <el-descriptions-item :label="t('common.timestamp')">{{ formatDate(decoded.timestamp) }}</el-descriptions-item>
            <el-descriptions-item :label="'Valid'">{{ decoded.valid ? 'Yes' : 'No' }}</el-descriptions-item>
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

const { t } = useI18n()
const mode = ref('generate')
const website = ref('')
const secretPhrase = ref('')
const tokenResult = ref('')
const decodeToken = ref('')
const decoded = ref<any>(null)

function formatDate(ts?: number) { if (!ts) return ''; return new Date(new Date(Date.UTC(2013, 10, 24, 12, 0, 0)).getTime() + ts * 1000).toLocaleString() }

async function generate() {
  if (!website.value || !secretPhrase.value) return
  try {
    const result = await nrcsApi.generateToken({ secretPhrase: secretPhrase.value, website: website.value })
    tokenResult.value = (result as any).token || ''
  } catch (e: any) { ElMessage.error(e?.message || 'Failed to generate token') }
}

async function decode() {
  if (!decodeToken.value) return
  try {
    decoded.value = await nrcsApi.decodeToken(decodeToken.value)
  } catch (e: any) { ElMessage.error(e?.message || 'Failed to decode token') }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } } }
.text-mono { font-family: monospace; word-break: break-all; }
</style>
