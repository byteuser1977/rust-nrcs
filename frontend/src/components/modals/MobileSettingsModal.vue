<template>
  <el-dialog
    v-model="visible"
    :title="t('mobileSettings.deviceSettings')"
    width="520px"
    :close-on-click-modal="false"
    destroy-on-close
    class="nrcs-modal"
    @open="onOpen"
  >
    <!-- 离线模式提示 -->
    <el-alert
      v-if="isOffline"
      :title="t('mobileSettings.workingOffline')"
      type="warning"
      :closable="false"
      show-icon
      class="offline-alert"
    />
    <el-alert
      v-else
      :title="t('mobileSettings.remoteNodeUrl', { url: remoteNodeUrl })"
      type="info"
      :closable="false"
      show-icon
      class="offline-alert"
    />

    <!-- 离线操作快捷入口 -->
    <div v-if="isOffline" class="offline-actions">
      <h4 class="section-title">{{ t('mobileSettings.offlineActions') }}</h4>
      <div class="offline-links">
        <el-button text @click="openTokenModal">
          <el-icon><Key /></el-icon>
          {{ t('mobileSettings.generateToken') }}
        </el-button>
        <el-button text @click="openSignTransactionModal">
          <el-icon><EditPen /></el-icon>
          {{ t('mobileSettings.signTransaction') }}
        </el-button>
      </div>
    </div>

    <el-divider content-position="left">{{ t('mobileSettings.settings') }}</el-divider>

    <el-form
      ref="formRef"
      :model="form"
      label-position="top"
      class="mobile-settings-form"
      @submit.prevent
    >
      <!-- 记住我复选框 -->
      <el-form-item>
        <el-checkbox v-model="form.is_check_remember_me">
          {{ t('mobileSettings.isCheckRememberMe') }}
        </el-checkbox>
      </el-form-item>

      <!-- 记住密码短语 -->
      <el-form-item>
        <el-checkbox v-model="form.is_store_remembered_passphrase">
          {{ t('mobileSettings.isStoreRememberedPassphrase') }}
        </el-checkbox>
      </el-form-item>

      <!-- 模拟移动应用 -->
      <el-form-item v-if="enableMobileAppSimulation">
        <el-checkbox v-model="form.is_simulate_app">
          {{ t('mobileSettings.simulateMobileApp') }}
        </el-checkbox>
      </el-form-item>

      <!-- 连接到测试网 -->
      <el-form-item>
        <el-checkbox v-model="form.is_testnet">
          {{ t('mobileSettings.connectToTestnet') }}
        </el-checkbox>
      </el-form-item>

      <!-- 远程节点地址 -->
      <el-form-item :label="t('mobileSettings.remoteNodeAddress')">
        <el-input
          v-model="form.remote_node_address"
          :placeholder="t('mobileSettings.remoteNodeAddressPlaceholder')"
          clearable
        />
      </el-form-item>

      <!-- 远程节点端口 -->
      <el-form-item :label="t('mobileSettings.remoteNodePort')">
        <el-input
          v-model="form.remote_node_port"
          :placeholder="t('mobileSettings.remoteNodePortPlaceholder')"
          clearable
        />
      </el-form-item>

      <!-- 使用 HTTPS -->
      <el-form-item>
        <el-checkbox v-model="form.is_remote_node_ssl">
          {{ t('mobileSettings.isRemoteNodeSsl') }}
        </el-checkbox>
      </el-form-item>

      <!-- 数据验证者数量 -->
      <el-form-item :label="t('mobileSettings.validatorsCount')">
        <el-input-number
          v-model="form.validators_count"
          :min="0"
          :max="3"
          controls-position="right"
          style="width: 100%"
        />
        <span class="form-hint">{{ t('mobileSettings.validatorsHint') }}</span>
      </el-form-item>

      <!-- 引导节点数量 -->
      <el-form-item :label="t('mobileSettings.bootstrapNodesCount')">
        <el-input-number
          v-model="form.bootstrap_nodes_count"
          :min="0"
          :max="5"
          controls-position="right"
          style="width: 100%"
        />
        <span class="form-hint">{{ t('mobileSettings.bootstrapHint') }}</span>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">{{ t('common.cancel') }}</el-button>
      <el-button
        type="primary"
        :loading="submitting"
        @click="handleSubmit"
      >
        {{ t('common.save') }}
      </el-button>
    </template>

    <!-- 子弹窗：令牌生成 / 离线签名 -->
    <TokenModal v-model:visible="showTokenModal" />
    <RawTransactionModal v-model:visible="showSignTransactionModal" />
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Key, EditPen } from '@element-plus/icons-vue'
import TokenModal from '@/components/modals/TokenModal.vue'
import RawTransactionModal from '@/components/modals/RawTransactionModal.vue'
import { setFeatureContext } from '@/utils/feature-detection'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  saved: []
}>()

const { t } = useI18n()

/** 弹窗可见性 */
const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val),
})

/** 表单引用 */
const formRef = ref()

/** 是否正在提交 */
const submitting = ref(false)

/** 是否离线模式（对标 is_offline 隐藏字段） */
const isOffline = ref(false)

/** 是否启用移动应用模拟（对标 NRS.isEnableMobileAppSimulation） */
const enableMobileAppSimulation = ref(true)

/** 远程节点 URL（用于提示展示） */
const remoteNodeUrl = computed(() => {
  const proto = form.is_remote_node_ssl ? 'https' : 'http'
  const port = form.remote_node_port ? `:${form.remote_node_port}` : ''
  return `${proto}://${form.remote_node_address || 'localhost'}${port}`
})

/** 表单数据（对标 NRS.mobileSettings） */
const form = reactive({
  is_check_remember_me: false,
  is_store_remembered_passphrase: false,
  is_simulate_app: false,
  is_testnet: false,
  remote_node_address: '',
  remote_node_port: '' as string | number,
  is_remote_node_ssl: false,
  validators_count: 0,
  bootstrap_nodes_count: 0,
})

/** 子弹窗可见性 */
const showTokenModal = ref(false)
const showSignTransactionModal = ref(false)

/**
 * 从 localStorage 加载 mobile_settings（对标 nrs.mobile.js:22 的 show.bs.modal 回调）。
 */
function loadSettings(): void {
  const stored = localStorage.getItem('mobile_settings')
  if (stored) {
    try {
      const parsed = JSON.parse(stored)
      form.is_check_remember_me = !!parsed.is_check_remember_me
      form.is_store_remembered_passphrase = !!parsed.is_store_remembered_passphrase
      form.is_simulate_app = !!parsed.is_simulate_app
      form.is_testnet = !!parsed.is_testnet
      form.remote_node_address = parsed.remote_node_address || ''
      form.remote_node_port = parsed.remote_node_port ?? ''
      form.is_remote_node_ssl = !!parsed.is_remote_node_ssl
      form.validators_count = Number(parsed.validators_count) || 0
      form.bootstrap_nodes_count = Number(parsed.bootstrap_nodes_count) || 0
    } catch {
      // 解析失败则保留默认值
    }
  }
}

/**
 * 弹窗打开时加载设置。
 */
function onOpen(): void {
  loadSettings()
}

/**
 * 打开令牌生成弹窗。
 */
function openTokenModal(): void {
  showTokenModal.value = true
}

/**
 * 打开离线签名弹窗。
 */
function openSignTransactionModal(): void {
  showSignTransactionModal.value = true
}

/**
 * 提交保存（对标 nrs.mobile.js:66 NRS.forms.setMobileSettings）。
 */
async function handleSubmit(): Promise<void> {
  submitting.value = true
  try {
    // 端口校验（对标 nrs.mobile.js:73-76）
    const portStr = String(form.remote_node_port)
    if (portStr && !/^\d+$/.test(portStr)) {
      ElMessage.error(t('mobileSettings.portNotNumeric'))
      return
    }

    // 验证者数量范围校验（0-3，由 el-input-number 保证，但仍防御性校验）
    if (form.validators_count < 0 || form.validators_count > 3) {
      ElMessage.error(t('mobileSettings.validatorsRangeError', { from: 0, to: 3 }))
      return
    }

    // 引导节点数量范围校验（0-5）
    if (form.bootstrap_nodes_count < 0 || form.bootstrap_nodes_count > 5) {
      ElMessage.error(t('mobileSettings.bootstrapRangeError', { from: 0, to: 5 }))
      return
    }

    // 持久化到 localStorage（对标 nrs.mobile.js:99 NRS.setJSONItem）
    const settings = {
      is_check_remember_me: form.is_check_remember_me,
      is_store_remembered_passphrase: form.is_store_remembered_passphrase,
      is_simulate_app: form.is_simulate_app,
      is_testnet: form.is_testnet,
      remote_node_address: form.remote_node_address,
      remote_node_port: Number(form.remote_node_port) || 0,
      is_remote_node_ssl: form.is_remote_node_ssl,
      validators_count: form.validators_count,
      bootstrap_nodes_count: form.bootstrap_nodes_count,
    }
    localStorage.setItem('mobile_settings', JSON.stringify(settings))

    // 同步更新特性检测上下文（对标 NRS.isEnableMobileAppSimulation / isTestNet 影响）
    setFeatureContext({
      isTestNet: form.is_testnet,
      mobileSettings: { is_simulate_app: form.is_simulate_app },
    })

    ElMessage.success(t('mobileSettings.saveSuccess'))
    emit('saved')
    visible.value = false
  } finally {
    submitting.value = false
  }
}

// 弹窗首次打开时触发加载
watch(visible, (val) => {
  if (val) {
    onOpen()
  }
})
</script>

<style scoped lang="scss">
.mobile-settings-modal {
  .offline-alert {
    margin-bottom: 12px;
  }

  .offline-actions {
    margin-bottom: 12px;

    .section-title {
      margin: 0 0 8px 0;
      font-size: 13px;
      color: $text-secondary;
      font-weight: 600;
    }

    .offline-links {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
    }
  }

  .mobile-settings-form {
    .form-hint {
      display: block;
      margin-top: 4px;
      font-size: 11px;
      color: $text-muted;
    }
  }
}
</style>
