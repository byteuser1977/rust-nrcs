<template>
  <el-dialog
    :model-value="visible"
    :title="title"
    :width="width"
    :top="top"
    :modal="modal"
    :modal-class="modalClass"
    :append-to-body="appendToBody"
    :close-on-click-modal="closeOnClickModal"
    :close-on-press-escape="closeOnPressEscape"
    :show-close="showClose"
    :before-close="handleBeforeClose"
    @update:model-value="handleClose"
  >
    <template #header>
      <slot name="header" />
    </template>

    <slot />

    <template #footer>
      <slot name="footer">
        <span class="dialog-footer">
          <BaseButton @click="handleCancel">
            {{ cancelText }}
          </BaseButton>
          <BaseButton
            type="primary"
            :loading="confirmLoading"
            @click="handleConfirm"
          >
            {{ confirmText }}
          </BaseButton>
        </span>
      </slot>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import BaseButton from './BaseButton.vue'

interface Props {
  // 对话框显隐
  visible: boolean
  // 标题
  title?: string
  // 宽度
  width?: string | number
  // 顶部距离
  top?: string
  // 是否遮罩
  modal?: boolean
  // 遮罩样式类
  modalClass?: string
  // 是否插入到 body
  appendToBody?: boolean
  // 是否可通过点击遮罩关闭
  closeOnClickModal?: boolean
  // 是否可按 ESC 关闭
  closeOnPressEscape?: boolean
  // 是否显示关闭按钮
  showClose?: boolean
  // 取消按钮文本
  cancelText?: string
  // 确认按钮文本
  confirmText?: string
  // 确认按钮加载状态
  confirmLoading?: boolean
  // 关闭前回调（返回 false 阻止关闭）
  beforeClose?: (done: () => void) => void | Promise<void>
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  width: '50%',
  top: '10vh',
  modal: true,
  appendToBody: true,
  closeOnClickModal: true,
  closeOnPressEscape: true,
  showClose: true,
  cancelText: '取消',
  confirmText: '确认',
  confirmLoading: false,
  beforeClose: undefined
})

const emit = defineEmits<{
  'update:visible': [value: boolean]
  confirm: []
  cancel: []
  close: []
}>()

// 内部确认加载状态（如果外部未控制）
const innerConfirmLoading = ref(false)
const finalLoading = ref(false)

watch(() => props.confirmLoading, (val) => {
  finalLoading.value = val
}, { immediate: true })

const handleBeforeClose = (done: () => void) => {
  if (props.beforeClose) {
    props.beforeClose(done)
  } else {
    done()
  }
}

const handleConfirm = async () => {
  try {
    emit('confirm')
    // 注意：loading 状态由父组件控制
  } catch (error: any) {
    if (error !== 'prevent_close') {
      ElMessage.error(error.message || '操作失败')
    }
    throw error
  }
}

const handleCancel = () => {
  emit('cancel')
  emit('update:visible', false)
}

const handleClose = () => {
  emit('close')
  emit('update:visible', false)
}

// 暴露方法给父组件
defineExpose({
  setLoading: (loading: boolean) => {
    finalLoading.value = loading
  }
})
</script>

<style scoped lang="scss">
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
