<template>
  <el-dialog
    v-model="visible"
    :title="t('asset.issueAsset')"
    width="520px"
    :close-on-click-modal="false"
    destroy-on-close
    class="nrcs-modal"
    @close="handleClose"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('asset.name')" prop="name">
        <el-input v-model="form.name" :placeholder="t('asset.namePlaceholder')" maxlength="10" show-word-limit clearable />
      </el-form-item>
      <el-form-item :label="t('asset.description')" prop="description">
        <el-input v-model="form.description" type="textarea" :rows="3" :placeholder="t('asset.descriptionPlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('asset.quantity')" prop="quantityQNT">
        <el-input v-model="form.quantityQNT" type="number" :placeholder="t('asset.quantityPlaceholder')" clearable />
      </el-form-item>
      <el-form-item :label="t('asset.decimals')" prop="decimals">
        <el-input-number v-model="form.decimals" :min="0" :max="8" style="width:100%" />
        <span class="hint-text">{{ t('asset.decimalsHint') }}</span>
      </el-form-item>
      <el-form-item :label="t('asset.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1000" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>
      <el-form-item :label="t('asset.deadline')" prop="deadline">
        <el-input v-model="form.deadline" placeholder="1440" type="number" clearable>
          <template #append>{{ t('common.minutes') }}</template>
        </el-input>
      </el-form-item>
      <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('asset.issueAsset') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * IssueAssetModal 组件 —— 发行资产弹窗。
 *
 * 对标 nrs.assetexchange.js 的 issueAsset 流程。
 *
 * 安全模型：secretPhrase 不随请求外发，通过 useNrcsForm 三步本地签名流程提交。
 */
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const form = reactive({
  name: '',
  description: '',
  quantityQNT: '',
  decimals: 0,
  feeNQT: '1000',
  deadline: '1440',
  secretPhrase: '',
})

/** 是否需要显示 secretPhrase 输入框 */
const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

const rules = computed<FormRules>(() => ({
  name: [
    { required: true, message: t('asset.nameRequired'), trigger: 'blur' },
    { min: 3, max: 10, message: t('asset.nameLength'), trigger: 'blur' },
  ],
  quantityQNT: [{ required: true, message: t('asset.quantityRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('asset.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('asset.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const secretPhrase = accountStore.hasSecretPhrase
        ? accountStore.secretPhrase
        : form.secretPhrase

      if (!secretPhrase) {
        ElMessage.warning(t('common.secretPhraseRequired'))
        return
      }

      const data: Record<string, any> = {
        secretPhrase,
        name: form.name,
        description: form.description,
        quantityQNT: String(form.quantityQNT),
        decimals: form.decimals,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
      }

      await submitForm('issueAsset', data, {
        successMessage: t('asset.issueSuccess'),
      })

      emit('success')
      handleClose()
    } catch (err: any) {
      console.error('Issue asset failed:', err)
    } finally {
      loading.value = false
    }
  })
}

function handleClose() {
  formRef.value?.resetFields()
  Object.assign(form, {
    name: '',
    description: '',
    quantityQNT: '',
    decimals: 0,
    feeNQT: '1000',
    deadline: '1440',
    secretPhrase: '',
  })
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.hint-text { font-size: $font-size-xs; color: $text-muted; margin-top: 4px; display: block; }
</style>
