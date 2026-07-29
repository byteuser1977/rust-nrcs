<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Stamp /></el-icon> {{ t('settings.hallmarkGenerator') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-tabs v-model="mode">
        <el-tab-pane :label="t('settings.generate')" name="generate">
          <el-form label-width="140px" style="max-width: 520px">
            <el-form-item :label="t('common.secretPhrase')">
              <el-input v-model="hallmarkSecret" type="password" show-password placeholder="Enter your secret phrase" />
            </el-form-item>
            <el-form-item :label="t('common.host')">
              <el-input v-model="host" placeholder="e.g. example.com or 192.168.1.1" />
            </el-form-item>
            <el-form-item :label="t('common.weight')">
              <el-input-number v-model="weight" :min="0" :max="99999" controls-position="right" style="width: 200px" />
            </el-form-item>
            <el-form-item :label="t('common.date')">
              <el-date-picker
                v-model="hallmarkDate"
                type="date"
                placeholder="Select date"
                format="YYYY-MM-DD"
                value-format="YYYY-MM-DD"
                style="width: 200px"
              />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="generateHallmark" :loading="generating">
                {{ t('settings.generate') }}
              </el-button>
            </el-form-item>
          </el-form>
          <div v-if="hallmarkResult" style="margin-top: 16px">
            <el-form-item :label="t('settings.hallmark')" label-width="140px">
              <div class="hallmark-display">
                <code class="hallmark-code">{{ hallmarkResult }}</code>
                <el-button size="small" type="primary" plain @click="copyHallmark">
                  <el-icon><CopyDocument /></el-icon> {{ t('common.copy') }}
                </el-button>
              </div>
            </el-form-item>
          </div>
        </el-tab-pane>
        <el-tab-pane :label="t('settings.decode')" name="decode">
          <el-form label-width="140px" style="max-width: 520px">
            <el-form-item :label="t('settings.hallmark')">
              <el-input v-model="hallmarkInput" type="textarea" :rows="3" placeholder="Paste hallmark string to decode" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="decodeHallmarkAction" :loading="decoding">
                {{ t('settings.decode') }}
              </el-button>
            </el-form-item>
          </el-form>
          <el-descriptions v-if="decodeResult" :column="2" border style="margin-top: 16px">
            <el-descriptions-item :label="t('common.host')">
              {{ decodeResult.host || '-' }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.weight')">
              {{ decodeResult.weight ?? '-' }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.date')">
              {{ decodeResult.date || '-' }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('settings.valid')">
              <el-tag :type="decodeResult.valid ? 'success' : 'danger'" size="small">
                {{ decodeResult.valid ? t('common.yes') : t('common.no') }}
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

const { t } = useI18n()

const mode = ref('generate')

// Generate tab
const hallmarkSecret = ref('')
const host = ref('')
const weight = ref(100)
const hallmarkDate = ref('')
const hallmarkResult = ref('')
const generating = ref(false)

// Decode tab
const hallmarkInput = ref('')
const decodeResult = ref<any>(null)
const decoding = ref(false)

async function generateHallmark() {
  if (!hallmarkSecret.value || !host.value) {
    ElMessage.warning(t('validation.required'))
    return
  }
  generating.value = true
  try {
    const result = await nrcsApi.markHost(
      hallmarkSecret.value,
      host.value,
      weight.value,
      hallmarkDate.value || new Date().toISOString().slice(0, 10),
    )
    hallmarkResult.value = (result as any)?.hallmark || result?.hallmark || JSON.stringify(result)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    generating.value = false
  }
}

async function copyHallmark() {
  try {
    await navigator.clipboard.writeText(hallmarkResult.value)
    ElMessage.success(t('common.copied'))
  } catch {
    ElMessage.error(t('common.copyFailed'))
  }
}

async function decodeHallmarkAction() {
  if (!hallmarkInput.value.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  decoding.value = true
  try {
    decodeResult.value = await nrcsApi.decodeHallmark(hallmarkInput.value.trim())
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
.hallmark-display {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.hallmark-code {
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
