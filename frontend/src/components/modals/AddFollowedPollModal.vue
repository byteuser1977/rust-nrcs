<template>
  <el-dialog v-model="visible" :title="t('modal.addPollBookmark')" width="450px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-alert
      :title="t('voting.bookmarkThisPoll')"
      type="info"
      :closable="false"
      show-icon
      class="mb-4"
    />
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top" @submit.prevent="handleSubmit">
      <el-form-item :label="t('voting.pollId')" prop="pollId">
        <el-input v-model="form.pollId" placeholder="e.g. 12345678901234567890" clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('voting.follow') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const FOLLOWED_POLLS_KEY = 'nrcs-followedPolls'

const { t } = useI18n()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)

const form = reactive({
  pollId: ''
})

const rules = computed<FormRules>(() => ({
  pollId: [{ required: true, message: t('error.fieldNotANumber', { field: t('voting.pollId') }), trigger: 'blur' }]
}))

function getFollowedPolls(): string[] {
  try {
    const stored = localStorage.getItem(FOLLOWED_POLLS_KEY)
    return stored ? JSON.parse(stored) : []
  } catch {
    return []
  }
}

function setFollowedPolls(polls: string[]): void {
  localStorage.setItem(FOLLOWED_POLLS_KEY, JSON.stringify(polls))
}

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      // Validate poll exists on the blockchain
      await nrcsApi.getPoll(form.pollId.trim())

      const followed = getFollowedPolls()
      if (followed.includes(form.pollId.trim())) {
        ElMessage.warning(t('error.pollAlreadyBookmarked'))
        return
      }

      followed.push(form.pollId.trim())
      setFollowedPolls(followed)

      ElMessage.success(t('success.addPollBookmark'))
      emit('success')
      handleClose()
    } catch (err: any) {
      if (err?.message?.includes('Unknown') || err?.message?.includes('not found') || err?.message?.includes('does not exist')) {
        ElMessage.error(t('error.pollNotExist'))
      } else {
        ElMessage.error(err?.message || t('error.unknownError'))
      }
    } finally {
      loading.value = false
    }
  })
}

function handleClose() {
  formRef.value?.resetFields()
  visible.value = false
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.mb-4 { margin-bottom: $space-lg; }
</style>
