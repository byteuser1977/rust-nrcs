<template>
  <div class="user-profile-page">
    <el-card>
      <template #header>
        <span>{{ t('account.userProfile') }}</span>
      </template>

      <el-form ref="formRef" :model="form" label-width="120px" style="max-width: 640px">
        <el-form-item :label="t('account.rsAddress')">
          <el-input :model-value="accountStore.accountRS" disabled />
        </el-form-item>
        <el-form-item :label="t('common.name')">
          <el-input v-model="form.name" maxlength="100" clearable />
        </el-form-item>
        <el-form-item :label="t('common.description')">
          <el-input v-model="form.description" type="textarea" :rows="3" maxlength="1000" clearable />
        </el-form-item>
        <el-form-item :label="t('common.fee')">
          <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
            <template #append>NRC</template>
          </el-input>
        </el-form-item>
        <el-form-item :label="t('common.deadline')">
          <el-input v-model="form.deadline" placeholder="1440" type="number" clearable>
            <template #append>{{ t('common.minutes') }}</template>
          </el-input>
        </el-form-item>
        <el-form-item :label="t('common.secretPhrase')">
          <el-input v-model="form.secretPhrase" type="password" show-password clearable
            :placeholder="t('login.secretPhraseRequired')" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :loading="saving" @click="saveProfile">
            {{ t('common.save') }}
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
/**
 * 账户资料编辑页面
 *
 * 对标 SetAccountInfoModal（nrs.modals.accounts.js setAccountInfoModal）：
 * 修改当前账户的链上名称与描述，需提供 secretPhrase 在本地签名，
 * secretPhrase 不出客户端（安全模型：本地签名）。
 */
import { onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const accountStore = useAccountStore()

const formRef = ref()
const saving = ref(false)

const form = reactive({
  name: '',
  description: '',
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: ''
})

// 挂载时用当前账户数据填充表单
onMounted(() => {
  form.name = accountStore.name
  form.description = accountStore.description
})

/**
 * 保存账户资料（本地签名，secretPhrase 不出客户端）
 */
async function saveProfile() {
  if (!accountStore.isLoggedIn) {
    ElMessage.warning(t('account.notLoggedIn'))
    return
  }
  if (!form.secretPhrase.trim()) {
    ElMessage.warning(t('common.secretPhraseRequired'))
    return
  }
  saving.value = true
  try {
    await nrcsApi.setAccountInfo({
      secretPhrase: form.secretPhrase.trim(),
      name: form.name,
      description: form.description,
      feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)),
      deadline: Number(form.deadline)
    })
    ElMessage.success(t('common.operationSuccess'))
    form.secretPhrase = ''
    // 刷新链上账户信息
    await accountStore.refreshAccount()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}
</script>
