<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><UserFilled /></el-icon> {{ t('settings.accountSettings') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-form :model="form" label-width="140px" style="max-width:600px">
        <el-form-item :label="t('common.name')">
          <el-input v-model="form.name" maxlength="100" show-word-limit />
        </el-form-item>
        <el-form-item :label="t('common.description')">
          <el-input v-model="form.description" type="textarea" :rows="3" maxlength="1000" show-word-limit />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="saveSettings">{{ t('common.save') }}</el-button>
        </el-form-item>
      </el-form>
    </el-card>
    <SetAccountInfoModal v-model:visible="showInfo" @success="loadAccount" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import SetAccountInfoModal from '@/components/modals/SetAccountInfoModal.vue'

const { t } = useI18n()
const form = ref({ name: '', description: '' })
const showInfo = ref(false)

onMounted(() => loadAccount())

async function loadAccount() {
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    if (!accountId) return
    const result = await nrcsApi.getAccount(accountId)
    const acct = result as any
    form.value.name = acct.name || ''
    form.value.description = acct.description || ''
  } catch {}
}

function saveSettings() { showInfo.value = true }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } }
</style>
