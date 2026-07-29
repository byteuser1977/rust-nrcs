<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><MagicStick /></el-icon> {{ t('settings.hashCalculator') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-form label-width="140px" style="max-width: 620px">
        <el-form-item :label="t('settings.algorithm')">
          <el-select v-model="algorithm" style="width: 200px">
            <el-option v-for="a in algorithms" :key="a.value" :value="a.value" :label="a.label" />
          </el-select>
        </el-form-item>
        <el-form-item :label="t('settings.secret')">
          <el-input v-model="secret" type="textarea" :rows="4" placeholder="Enter secret / data to hash" />
        </el-form-item>
        <el-form-item :label="t('settings.isText')">
          <el-switch v-model="isText" />
          <span class="switch-label">{{ isText ? t('common.text') : t('settings.hex') }}</span>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="calculate" :loading="calculating">
            {{ t('settings.calculate') }}
          </el-button>
        </el-form-item>
      </el-form>
      <div v-if="hashResult" style="margin-top: 16px">
        <el-form-item :label="t('settings.hash')" label-width="140px">
          <div class="hash-display">
            <el-input :model-value="hashResult" readonly class="hash-input">
              <template #append>
                <el-button @click="copyHash">
                  <el-icon><CopyDocument /></el-icon>
                </el-button>
              </template>
            </el-input>
          </div>
        </el-form-item>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const algorithms = [
  { value: 2, label: 'SHA-256' },
  { value: 3, label: 'SHA-512' },
  { value: 6, label: 'RIPEMD160' },
  { value: 62, label: 'Keccak256' },
  { value: 17, label: 'SHA3-256' },
  { value: 18, label: 'SHA3-512' },
]

const algorithm = ref(2)
const secret = ref('')
const isText = ref(true)
const hashResult = ref('')
const calculating = ref(false)

async function calculate() {
  if (!secret.value.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  calculating.value = true
  try {
    const result = await nrcsApi.hash({
      hashAlgorithm: algorithm.value,
      secret: secret.value,
      secretIsText: isText.value,
    })
    hashResult.value = result?.hash || ''
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    calculating.value = false
  }
}

async function copyHash() {
  try {
    await navigator.clipboard.writeText(hashResult.value)
    ElMessage.success(t('common.copied'))
  } catch {
    ElMessage.error(t('common.copyFailed'))
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
.switch-label {
  margin-left: 8px;
  font-size: 13px;
  color: #909399;
}
.hash-display {
  width: 100%;
  .hash-input {
    :deep(.el-input__inner) {
      font-family: 'Roboto Mono', monospace;
      font-size: 13px;
    }
  }
}
</style>
