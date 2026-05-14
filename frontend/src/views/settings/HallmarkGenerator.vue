<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Stamp /></el-icon> {{ t('settings.hallmarkGenerator') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-form label-width="140px" style="max-width:600px">
        <el-form-item :label="t('settings.hallmark')">
          <el-input v-model="hallmark" placeholder="Enter hallmark string" clearable />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="decode">{{ t('settings.decode') }}</el-button>
        </el-form-item>
      </el-form>
      <el-descriptions v-if="result" :column="2" border style="margin-top:16px">
        <el-descriptions-item :label="t('common.host')">{{ result.host }}</el-descriptions-item>
        <el-descriptions-item :label="t('common.weight')">{{ result.weight }}</el-descriptions-item>
        <el-descriptions-item :label="t('common.date')">{{ result.date }}</el-descriptions-item>
        <el-descriptions-item :label="'Valid'">{{ result.valid ? 'Yes' : 'No' }}</el-descriptions-item>
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
const hallmark = ref('')
const result = ref<any>(null)

async function decode() {
  if (!hallmark.value.trim()) return
  try {
    result.value = await nrcsApi.decodeHallmark(hallmark.value.trim())
  } catch (e: any) { ElMessage.error(e?.message || 'Failed to decode hallmark') }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } } }
</style>
