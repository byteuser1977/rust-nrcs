<template>
  <el-dialog v-model="visible" :title="t('alias.sellAlias')" width="520px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('alias.aliasName')">
        <el-input v-model="aliasName" disabled />
      </el-form-item>

      <!-- 出售方式切换（对标 nrs.aliases.js:246-272 sell_alias_to_anyone / sell_alias_to_specific_account） -->
      <el-form-item :label="t('alias.sellToAnyone')">
        <el-radio-group v-model="sellMode" @change="onSellModeChange">
          <el-radio value="anyone">{{ t('alias.sellToAnyone') }}</el-radio>
          <el-radio value="specific">{{ t('alias.sellToSpecificAccount') }}</el-radio>
        </el-radio-group>
      </el-form-item>

      <el-form-item v-if="sellMode === 'specific'" :label="t('alias.buyer')" prop="buyer">
        <el-input v-model="form.buyer" :placeholder="t('alias.buyerPlaceholder')" clearable />
      </el-form-item>

      <el-form-item :label="t('alias.price')" prop="priceNQT">
        <el-input v-model="form.priceNQT" type="number" placeholder="0.00" clearable><template #append>NRC</template></el-input>
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12"><el-form-item :label="t('common.fee')" prop="feeNQT"><el-input v-model="form.feeNQT" placeholder="1" type="number" clearable><template #append>NRC</template></el-input></el-form-item></el-col>
        <el-col :span="12"><el-form-item :label="t('common.deadline')" prop="deadline"><el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input></el-form-item></el-col>
      </el-row>
      <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('alias.sellAlias') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * SellAliasModal 组件 —— 出售别名弹窗。
 *
 * 对标 nrs.aliases.js 的 sellAlias 表单（nrs.aliases.js:129-182）。
 *
 * 支持两种出售方式（对标 nrs.aliases.js:246-272）：
 *   - sellToAnyone：出售给任何人（buyer 设为创世账户）
 *   - sellToSpecificAccount：出售给指定账户
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

const props = defineProps<{ aliasName?: string }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const aliasName = computed(() => props.aliasName || '')

/** 出售方式：anyone=出售给任何人，specific=指定买方 */
const sellMode = ref<'anyone' | 'specific'>('anyone')

const form = reactive({
  priceNQT: '',
  buyer: '',
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: ''
})

/** 弹窗打开时预填 secretPhrase */
watch(visible, (open) => {
  if (open) {
    form.secretPhrase = accountStore.secretPhrase || ''
    sellMode.value = 'anyone'
    form.buyer = ''
  }
})

const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

const rules = computed<FormRules>(() => ({
  priceNQT: [{ required: true, message: t('alias.priceRequired'), trigger: 'blur' }],
  buyer: sellMode.value === 'specific'
    ? [{ required: true, message: t('validation.invalidAddress'), trigger: 'blur' }]
    : [],
  feeNQT: [{ required: true, message: t('alias.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('alias.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : []
}))

/**
 * 出售方式切换处理（对标 nrs.aliases.js:246-272）。
 *
 * - anyone：清空 buyer
 * - specific：清空 buyer 让用户输入
 */
function onSellModeChange(): void {
  form.buyer = ''
}

/**
 * 提交出售别名（通过 useNrcsForm 三步本地签名流程）。
 *
 * 出售给任何人时，buyer 留空（服务端解释为间接出售）。
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const data: Record<string, any> = {
        aliasName: aliasName.value,
        priceNXT: form.priceNQT,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
        secretPhrase: form.secretPhrase
      }
      if (sellMode.value === 'specific' && form.buyer) {
        data.recipient = form.buyer
      }
      await submitForm('sellAlias', data, {
        successMessage: t('alias.sellSuccess')
      })
      emit('success')
      handleClose()
    } catch (err: any) {
      console.error('Sell alias failed:', err)
    } finally {
      loading.value = false
    }
  })
}

function handleClose(): void {
  formRef.value?.resetFields()
  Object.assign(form, {
    priceNQT: '',
    buyer: '',
    feeNQT: '1',
    deadline: '1440',
    secretPhrase: ''
  })
  visible.value = false
}
</script>
<style scoped lang="scss">@use '@/assets/styles/variables' as *;</style>
