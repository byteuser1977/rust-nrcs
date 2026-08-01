<template>
  <el-dialog v-model="visible" :title="t('alias.deleteAlias')" width="500px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('alias.aliasName')">
        <el-input :model-value="aliasName" disabled />
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('common.fee')" prop="feeNQT">
            <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable><template #append>NRC</template></el-input>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item :label="t('common.deadline')" prop="deadline">
            <el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input>
          </el-form-item>
        </el-col>
      </el-row>
      <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="danger" :loading="loading" @click="handleSubmit">{{ t('alias.deleteAlias') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * DeleteAliasModal 组件 —— 删除别名弹窗。
 *
 * 对标 nrs.aliases.js 的 deleteAlias 表单（nrs.aliases.js:219-244）。
 *
 * 安全模型：secretPhrase 不随请求外发，由 useNrcsForm 在本地签名。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { type FormInstance, type FormRules } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const props = defineProps<{ alias?: { aliasName?: string } }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const aliasName = computed(() => props.alias?.aliasName || '')

const form = reactive({
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: ''
})

const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

/** 弹窗打开时预填 secretPhrase */
watch(visible, (val) => {
  if (val) {
    form.secretPhrase = accountStore.secretPhrase || ''
  }
})

const rules = computed<FormRules>(() => ({
  feeNQT: [{ required: true, message: t('alias.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('alias.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : []
}))

/**
 * 提交删除别名（通过 useNrcsForm 三步本地签名流程）。
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await submitForm('deleteAlias', {
        aliasName: aliasName.value,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
        secretPhrase: form.secretPhrase
      }, {
        successMessage: t('alias.deleteSuccess')
      })
      emit('success')
      handleClose()
    } catch (err: any) {
      console.error('Delete alias failed:', err)
    } finally {
      loading.value = false
    }
  })
}

function handleClose(): void {
  formRef.value?.resetFields()
  Object.assign(form, { feeNQT: '1', deadline: '1440', secretPhrase: '' })
  visible.value = false
}
</script>
<style scoped lang="scss">@use '@/assets/styles/variables' as *;</style>
