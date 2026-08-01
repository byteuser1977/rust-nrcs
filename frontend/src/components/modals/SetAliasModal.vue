<template>
  <el-dialog v-model="visible" :title="isEdit ? t('alias.editAlias') : t('alias.registerAlias')" width="520px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <!-- 别名名称：编辑模式下禁用（对标 nrs.aliases.js:357 register_alias_alias 隐藏 + alias_update=1） -->
      <el-form-item :label="t('alias.aliasName')" prop="aliasName">
        <el-input v-model="form.aliasName" :placeholder="t('alias.aliasNamePlaceholder')" clearable :disabled="isEdit" />
      </el-form-item>

      <!-- 别名类型（对标 nrs.aliases.js:421 setAliasType：uri/account/general） -->
      <el-form-item :label="t('alias.type')" prop="aliasType">
        <el-select v-model="form.aliasType" style="width:100%" @change="onAliasTypeChange">
          <el-option value="uri" label="URI" />
          <el-option value="account" :label="t('common.account')" />
          <el-option value="general" :label="t('common.other')" />
        </el-select>
      </el-form-item>

      <!-- URI/账户/数据 输入框（label 和 placeholder 随类型变化，对标 setAliasType） -->
      <el-form-item :label="aliasURILabel" prop="aliasURI">
        <el-input v-model="form.aliasURI" :placeholder="aliasURIPlaceholder" clearable />
        <div v-if="form.aliasType === 'account'" class="alias-help">{{ t('alias.aliasAccountHelp') }}</div>
        <div v-else-if="form.aliasType === 'general'" class="alias-help">{{ t('alias.aliasDataHelp') }}</div>
      </el-form-item>

      <el-row :gutter="16">
        <el-col :span="12"><el-form-item :label="t('common.fee')" prop="feeNQT"><el-input v-model="form.feeNQT" placeholder="2" type="number" clearable><template #append>NRC</template></el-input></el-form-item></el-col>
        <el-col :span="12"><el-form-item :label="t('common.deadline')" prop="deadline"><el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input></el-form-item></el-col>
      </el-row>
      <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ isEdit ? t('alias.editAlias') : t('alias.registerAlias') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * SetAliasModal 组件 —— 注册/编辑别名弹窗。
 *
 * 对标 nrs.aliases.js 的 setAlias 表单与 setAliasType 函数（nrs.aliases.js:385-490）。
 *
 * 别名类型（type）：
 *   - uri：URI（http:// 自动补全，对标 setAliasType:425-440）
 *   - account：账户 ID（自动转换为 acct:NRCS-xxx@nxt 格式，对标 setAliasType:441-474）
 *   - general：任意数据（对标 setAliasType:475-489）
 *
 * 安全模型：secretPhrase 不随请求外发，由 useNrcsForm 在本地签名。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

/** 编辑模式下传入的别名对象（含 aliasName 和 aliasURI） */
const props = defineProps<{ editAlias?: { aliasName?: string; aliasURI?: string } }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const isEdit = computed(() => !!props.editAlias?.aliasName)

const form = reactive({
  aliasType: 'uri',
  aliasName: '',
  aliasURI: '',
  feeNQT: '2',
  deadline: '1440',
  secretPhrase: ''
})

/** URI 输入框 label（对标 setAliasType 动态切换 label） */
const aliasURILabel = computed(() => {
  if (form.aliasType === 'account') return t('common.account')
  if (form.aliasType === 'general') return t('alias.aliasUri') // "数据" 标签
  return t('alias.uri')
})

/** URI 输入框 placeholder */
const aliasURIPlaceholder = computed(() => {
  if (form.aliasType === 'account') return t('contacts.accountRSPlaceholder')
  if (form.aliasType === 'general') return t('alias.uriPlaceholder')
  return 'https://'
})

const rules = computed<FormRules>(() => ({
  aliasName: [{ required: true, message: t('alias.nameRequired'), trigger: 'blur' }],
  aliasURI: [{ required: true, message: t('alias.uriRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('alias.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('alias.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
}))

/**
 * 弹窗打开时根据 editAlias 预填表单（对标 nrs.aliases.js:328-377 register_alias_modal show.bs.modal）。
 *
 * 编辑模式：根据 aliasURI 自动识别类型（http:// → uri，acct:xxx@nxt / nacc:xxx → account，其他 → general）。
 */
watch(visible, (open) => {
  if (!open) return
  // 预填 secretPhrase（若 accountStore 已有）
  form.secretPhrase = accountStore.secretPhrase || ''
  if (props.editAlias?.aliasName) {
    form.aliasName = props.editAlias.aliasName
    detectAliasType(props.editAlias.aliasURI || '')
  } else {
    form.aliasName = ''
    form.aliasType = 'uri'
    form.aliasURI = 'https://'
  }
})

/**
 * 根据现有 aliasURI 自动识别类型（对标 nrs.aliases.js:345-353）。
 *
 * - http:// 或 https:// → uri
 * - acct:xxx@nxt 或 nacc:xxx → account（提取账户 RS）
 * - 其他 → general
 */
function detectAliasType(uri: string): void {
  if (/http:\/\//i.test(uri) || /https:\/\//i.test(uri)) {
    form.aliasType = 'uri'
    form.aliasURI = uri
  } else {
    const accountMatch = /acct:(.*)@nxt/i.exec(uri) || /nacc:(.*)/i.exec(uri)
    if (accountMatch && accountMatch[1]) {
      form.aliasType = 'account'
      form.aliasURI = accountMatch[1].toUpperCase()
    } else {
      form.aliasType = 'general'
      form.aliasURI = uri
    }
  }
}

/**
 * 类型切换时处理 URI（对标 nrs.aliases.js:421 setAliasType）。
 *
 * - uri：自动补 http:// 前缀
 * - account：默认填当前账户 RS
 * - general：清空（若为账户 RS 或 http://）
 */
function onAliasTypeChange(): void {
  if (form.aliasType === 'uri') {
    if (!form.aliasURI || form.aliasURI === accountStore.accountRS) {
      form.aliasURI = 'https://'
    } else if (!/https?:\/\//i.test(form.aliasURI)) {
      form.aliasURI = 'http://' + form.aliasURI
    }
  } else if (form.aliasType === 'account') {
    // 若当前 URI 不是 RS 格式，则填入当前账户 RS（对标 setAliasType:472）
    if (!/^NRCS-/i.test(form.aliasURI)) {
      form.aliasURI = accountStore.accountRS || ''
    } else {
      form.aliasURI = form.aliasURI.toUpperCase()
    }
  } else {
    // general：若为账户 RS 或 http:// 则清空（对标 setAliasType:481-484）
    if (form.aliasURI === accountStore.accountRS || form.aliasURI === 'http://' || form.aliasURI === 'https://') {
      form.aliasURI = ''
    }
  }
}

/**
 * 提交注册/编辑别名（对标 nrs.aliases.js:385 setAlias 表单）。
 *
 * 类型为 account 时，将 RS 转换为 acct:NRCS-xxx@nxt 格式（对标 nrs.aliases.js:390-411）。
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      let aliasURI = form.aliasURI.trim()

      // account 类型：转换为 acct:xxx@nxt 格式（对标 nrs.aliases.js:390-411）
      if (form.aliasType === 'account') {
        if (!/^acct:.*@nxt/i.test(aliasURI) && !/^nacc:/i.test(aliasURI)) {
          if (/^NRCS-/i.test(aliasURI)) {
            aliasURI = `acct:${aliasURI}@nxt`
          } else if (/^\d+$/.test(aliasURI)) {
            ElMessage.error(t('error.numericIdsNotAllowed'))
            return
          } else {
            ElMessage.error(t('error.invalidAccountId'))
            return
          }
        }
      }

      await submitForm('setAlias', {
        aliasName: form.aliasName,
        aliasURI,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
        secretPhrase: form.secretPhrase
      }, {
        successMessage: isEdit.value ? t('alias.editAlias') : t('alias.registerSuccess')
      })

      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('alias.registerError'))
    } finally {
      loading.value = false
    }
  })
}

function handleClose(): void {
  formRef.value?.resetFields()
  Object.assign(form, {
    aliasType: 'uri',
    aliasName: '',
    aliasURI: '',
    feeNQT: '2',
    deadline: '1440',
    secretPhrase: ''
  })
  visible.value = false
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.alias-help {
  margin-top: 4px;
  font-size: $font-size-xs;
  color: $text-muted;
  line-height: 1.4;
}
</style>
