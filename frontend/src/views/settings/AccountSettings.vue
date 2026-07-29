<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><UserFilled /></el-icon> {{ t('settings.accountSettings') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-tabs v-model="activeTab">
        <!-- Set Account Info -->
        <el-tab-pane :label="t('settings.setAccountInfo')" name="info">
          <el-form
            ref="infoFormRef"
            :model="infoForm"
            label-width="140px"
            style="max-width: 620px"
            :rules="infoRules"
          >
            <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
              <el-input v-model="infoForm.secretPhrase" type="password" show-password placeholder="Enter your secret phrase" />
            </el-form-item>
            <el-form-item :label="t('common.name')" prop="name">
              <el-input v-model="infoForm.name" maxlength="100" show-word-limit placeholder="Account name" />
            </el-form-item>
            <el-form-item :label="t('common.description')" prop="description">
              <el-input
                v-model="infoForm.description"
                type="textarea"
                :rows="4"
                maxlength="1000"
                show-word-limit
                placeholder="Account description"
              />
            </el-form-item>
            <el-form-item :label="t('common.fee') + ' (NQT)'">
              <el-input v-model="infoForm.feeNQT" placeholder="100000000" />
            </el-form-item>
            <el-form-item :label="t('settings.deadline') + ' (min)'">
              <el-input-number v-model="infoForm.deadline" :min="1" :max="1440" controls-position="right" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="submitAccountInfo" :loading="isInfoSubmitting">
                {{ t('common.save') }}
              </el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- Account Properties -->
        <el-tab-pane :label="t('settings.setAccountProperty')" name="property">
          <el-form
            ref="propertyFormRef"
            :model="propertyForm"
            label-width="140px"
            style="max-width: 620px"
            :rules="propertyRules"
          >
            <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
              <el-input v-model="propertyForm.secretPhrase" type="password" show-password placeholder="Enter your secret phrase" />
            </el-form-item>
            <el-form-item :label="t('common.recipient')" prop="recipient">
              <el-input v-model="propertyForm.recipient" placeholder="NRCS-XXXX-XXXX-XXXX-XXXXX" />
            </el-form-item>
            <el-form-item :label="t('settings.property')" prop="property">
              <el-input v-model="propertyForm.property" placeholder="Property name" />
            </el-form-item>
            <el-form-item :label="t('common.value')">
              <el-input v-model="propertyForm.value" placeholder="Property value" />
            </el-form-item>
            <el-form-item :label="t('common.fee') + ' (NQT)'">
              <el-input v-model="propertyForm.feeNQT" placeholder="100000000" />
            </el-form-item>
            <el-form-item :label="t('settings.deadline') + ' (min)'">
              <el-input-number v-model="propertyForm.deadline" :min="1" :max="1440" controls-position="right" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="submitProperty" :loading="isPropertySubmitting">
                {{ t('common.submit') }}
              </el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const { submitForm } = useNrcsForm()
const accountStore = useAccountStore()

const activeTab = ref('info')

// --- Set Account Info ---
const infoFormRef = ref<FormInstance>()
const isInfoSubmitting = ref(false)
const infoForm = reactive({
  secretPhrase: accountStore.secretPhrase || '',
  name: accountStore.name || '',
  description: accountStore.description || '',
  feeNQT: '100000000',
  deadline: 1440,
})

const infoRules: FormRules = {
  secretPhrase: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  name: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
}

async function submitAccountInfo() {
  const valid = await infoFormRef.value?.validate().catch(() => false)
  if (!valid) return
  isInfoSubmitting.value = true
  try {
    await submitForm('setAccountInfo', {
      secretPhrase: infoForm.secretPhrase,
      name: infoForm.name,
      description: infoForm.description,
      feeNQT: infoForm.feeNQT,
      deadline: infoForm.deadline,
    }, {
      successMessage: t('settings.accountInfoUpdated'),
    })
    // Update store
    accountStore.name = infoForm.name
    accountStore.description = infoForm.description
  } catch {
    // Error handled by useNrcsForm
  } finally {
    isInfoSubmitting.value = false
  }
}

// --- Set Account Property ---
const propertyFormRef = ref<FormInstance>()
const isPropertySubmitting = ref(false)
const propertyForm = reactive({
  secretPhrase: accountStore.secretPhrase || '',
  recipient: '',
  property: '',
  value: '',
  feeNQT: '100000000',
  deadline: 1440,
})

const propertyRules: FormRules = {
  secretPhrase: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  recipient: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  property: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
}

async function submitProperty() {
  const valid = await propertyFormRef.value?.validate().catch(() => false)
  if (!valid) return
  isPropertySubmitting.value = true
  try {
    await submitForm('setAccountProperty', {
      secretPhrase: propertyForm.secretPhrase,
      recipient: propertyForm.recipient,
      property: propertyForm.property,
      value: propertyForm.value,
      feeNQT: propertyForm.feeNQT,
      deadline: propertyForm.deadline,
    }, {
      successMessage: t('common.operationSuccess'),
    })
  } catch {
    // Error handled by useNrcsForm
  } finally {
    isPropertySubmitting.value = false
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
    .header-actions {
      display: flex;
      gap: 8px;
    }
  }
}
</style>
