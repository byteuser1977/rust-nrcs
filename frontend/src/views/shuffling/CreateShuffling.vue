<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Plus /></el-icon> {{ t('menu.createShuffling') }}</h2>
    </div>
    <el-card shadow="hover" v-loading="isSubmitting">
      <el-form :model="form" label-width="140px" @submit.prevent="handleSubmit">
        <el-form-item label="Amount">
          <el-input v-model="form.amount" placeholder="Enter amount" />
        </el-form-item>
        <el-form-item label="Participants">
          <el-input-number v-model="form.participantCount" :min="2" :max="100" />
        </el-form-item>
        <el-form-item label="Registration Period">
          <el-input-number v-model="form.registrationPeriod" :min="1" />
        </el-form-item>
        <el-form-item label="Fee (NQT)">
          <el-input v-model="form.feeNQT" placeholder="Fee" />
        </el-form-item>
        <el-form-item label="Deadline (min)">
          <el-input-number v-model="form.deadline" :min="1" :max="1440" />
        </el-form-item>
        <el-form-item label="Secret Phrase">
          <el-input v-model="form.secretPhrase" type="password" placeholder="Enter secret phrase" show-password />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSubmit" :loading="isSubmitting">
            {{ t('common.submit') }}
          </el-button>
          <el-button @click="resetForm">{{ t('common.reset') }}</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const isSubmitting = ref(false)

const form = reactive({
  amount: '0',
  participantCount: 3,
  registrationPeriod: 10,
  feeNQT: '100000000',
  deadline: 1440,
  secretPhrase: ''
})

function resetForm() {
  form.amount = '0'
  form.participantCount = 3
  form.registrationPeriod = 10
  form.feeNQT = '100000000'
  form.deadline = 1440
  form.secretPhrase = ''
}

async function handleSubmit() {
  if (!form.secretPhrase) {
    ElMessage.warning('Please enter your secret phrase')
    return
  }
  isSubmitting.value = true
  try {
    await nrcsApi.shufflingCreate({
      secretPhrase: form.secretPhrase,
      amount: form.amount,
      participantCount: form.participantCount,
      registrationPeriod: form.registrationPeriod,
      holdingType: 0,
      holding: '',
      feeNQT: form.feeNQT,
      deadline: form.deadline
    })
    ElMessage.success(t('common.operationSuccess'))
    resetForm()
  } catch (error: any) {
    ElMessage.error(error?.message || t('common.operationFailed'))
  } finally {
    isSubmitting.value = false
  }
}
</script>

<style scoped lang="scss">
.page-container {
  max-width: 600px;
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
</style>
