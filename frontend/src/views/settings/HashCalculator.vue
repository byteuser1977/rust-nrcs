<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><MagicStick /></el-icon> {{ t('settings.hashCalculator') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-form label-width="140px" style="max-width:600px">
        <el-form-item :label="t('settings.algorithm')">
          <el-select v-model="algorithm" style="width:180px">
            <el-option :value="2" label="SHA-256" />
            <el-option :value="3" label="SHA-512" />
            <el-option :value="6" label="RIPEMD160" />
            <el-option :value="62" label="Keccak256" />
          </el-select>
        </el-form-item>
        <el-form-item :label="t('settings.secret')">
          <el-input v-model="secret" type="textarea" :rows="3" />
        </el-form-item>
        <el-form-item :label="t('settings.isText')">
          <el-switch v-model="isText" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="calculate">{{ t('settings.calculate') }}</el-button>
        </el-form-item>
      </el-form>
      <el-descriptions v-if="hashResult" :column="1" border style="margin-top:16px">
        <el-descriptions-item :label="t('settings.hash')">
          <span class="text-mono">{{ hashResult }}</span>
        </el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const algorithm = ref(2)
const secret = ref('')
const isText = ref(true)
const hashResult = ref('')

async function calculate() {
  if (!secret.value) return
  try {
    const result = await nrcsApi.hash({ hashAlgorithm: algorithm.value, secret: secret.value, secretIsText: isText.value })
    hashResult.value = (result as any).hash || ''
  } catch (e: any) { ElMessage.error(e?.message || 'Failed to calculate hash') }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } } }
.text-mono { font-family: monospace; word-break: break-all; }
</style>
