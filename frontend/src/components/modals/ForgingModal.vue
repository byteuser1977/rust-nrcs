<template>
  <el-dialog
    v-model="visible"
    :title="mode === 'start' ? t('forging.startTitle') : t('forging.stopTitle')"
    width="460px"
    :close-on-click-modal="false"
    destroy-on-close
    class="forging-modal"
    @close="handleClose"
  >
    <!-- 前置校验错误提示（对标 nrs.modals.forging.js:64-89 的 6 种错误状态） -->
    <el-alert
      v-if="preconditionError"
      :title="preconditionError"
      type="error"
      :closable="false"
      show-icon
      class="forging-modal-error"
    />

    <template v-else>
      <!-- 确认提示 -->
      <el-alert
        :title="mode === 'start' ? t('forging.confirmStart') : t('forging.confirmStop')"
        :type="mode === 'start' ? 'success' : 'warning'"
        :closable="false"
        show-icon
        class="forging-modal-info"
      />

      <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
        <!-- 密码短语（仅当内存中无 secretPhrase 时显示，对标 nrs.modals.forging.js:106-117） -->
        <el-form-item v-if="!accountStore.hasSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
          <el-input
            v-model="form.secretPhrase"
            type="password"
            show-password
            :placeholder="t('forging.enterSecretPhrase')"
            clearable
            @keyup.enter="handleSubmit"
          >
            <template #prefix>
              <el-icon><Lock /></el-icon>
            </template>
          </el-input>
        </el-form-item>

        <!-- 当前锻造状态信息 -->
        <div v-if="mode === 'stop'" class="forging-modal-status">
          <span class="status-label">{{ t('forging.forging') }}:</span>
          <span class="status-value">{{ forgingStatusLabel }}</span>
        </div>
      </el-form>
    </template>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
        <el-button
          v-if="!preconditionError"
          :type="mode === 'start' ? 'success' : 'danger'"
          :loading="submitting"
          @click="handleSubmit"
        >
          {{ mode === 'start' ? t('forging.startForging') : t('forging.stopForging') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * ForgingModal 组件 —— 开始/停止锻造弹窗。
 *
 * 对标 nrs.modals.forging.js 中的 start_forging_modal / stop_forging_modal。
 *
 * 主要功能：
 *   1. 根据 mode（start/stop）显示对应标题与按钮
 *   2. 弹窗显示时执行前置校验（checkForgingPreconditions，6 种错误状态）
 *   3. 若内存中已有 secretPhrase（password 登录）则直接使用，否则提示输入
 *   4. 提交后调用 useForging.startForging/stopForging，更新锻造状态
 *
 * 对标参考：
 *   - startForgingComplete（:22-37）：成功 → forging 状态 + tooltip
 *   - stopForgingComplete（:39-58）：成功 → not_forging 状态
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { Lock } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { useForging } from '@/composables/useForging'

const props = withDefaults(
  defineProps<{
    /** 模式：start（开始锻造）/ stop（停止锻造） */
    mode?: 'start' | 'stop'
  }>(),
  { mode: 'start' },
)

const emit = defineEmits<{
  success: []
  close: []
}>()

const { t } = useI18n()
const accountStore = useAccountStore()
const {
  forgingStatusLabel,
  checkForgingPreconditions,
  startForging,
  stopForging,
} = useForging()

const visible = defineModel<boolean>('visible', { default: false })

const formRef = ref<FormInstance>()
const submitting = ref(false)
const form = reactive({ secretPhrase: '' })

/** 前置校验错误（非空时表示无法锻造，对标 nrs.modals.forging.js:64-89） */
const preconditionError = ref<string>('')

const rules = computed<FormRules>(() => ({
  secretPhrase: [
    {
      required: !accountStore.hasSecretPhrase,
      message: t('forging.secretPhraseRequired'),
      trigger: 'blur',
    },
  ],
}))

/**
 * 执行前置校验（对标 forgingIndicator.click 的 6 条件检查）。
 *
 * 弹窗显示时调用，若返回错误则禁用提交按钮并展示错误。
 */
function runPreconditions(): void {
  if (visible.value) {
    preconditionError.value = checkForgingPreconditions() || ''
  }
}

/**
 * 提交锻造操作（对标 startForgingComplete / stopForgingComplete）。
 */
async function handleSubmit(): Promise<void> {
  if (preconditionError.value) return

  // 校验表单（仅当需要密码短语时）
  if (!accountStore.hasSecretPhrase && formRef.value) {
    const valid = await formRef.value.validate().catch(() => false)
    if (!valid) return
  }

  submitting.value = true
  try {
    const secret = accountStore.hasSecretPhrase ? accountStore.secretPhrase : form.secretPhrase
    if (props.mode === 'start') {
      const ok = await startForging(secret)
      if (ok) {
        ElMessage.success(t('forging.startSuccess'))
        emit('success')
        handleClose()
      } else {
        ElMessage.error(t('forging.startError'))
      }
    } else {
      const ok = await stopForging(accountStore.hasSecretPhrase ? secret : secret)
      if (ok) {
        ElMessage.success(t('forging.stopSuccess'))
        emit('success')
        handleClose()
      } else {
        ElMessage.error(t('forging.stopError'))
      }
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('forging.startError'))
  } finally {
    submitting.value = false
  }
}

/**
 * 关闭 modal 并重置状态。
 */
function handleClose(): void {
  visible.value = false
  form.secretPhrase = ''
  preconditionError.value = ''
  formRef.value?.resetFields()
  emit('close')
}

// 弹窗显示时执行前置校验
watch(visible, (val) => {
  if (val) {
    form.secretPhrase = ''
    preconditionError.value = ''
    runPreconditions()
  }
})
</script>

<style scoped lang="scss">
.forging-modal-error {
  margin-bottom: 12px;
}
.forging-modal-info {
  margin-bottom: 16px;
}
.forging-modal-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  font-size: 14px;
  .status-label {
    color: var(--el-text-color-secondary);
  }
  .status-value {
    font-weight: 600;
  }
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
