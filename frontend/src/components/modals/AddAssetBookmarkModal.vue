<template>
  <el-dialog
    v-model="visible"
    :title="t('asset.addBookmark')"
    width="480px"
    :close-on-click-modal="false"
    destroy-on-close
    class="add-asset-bookmark-modal"
    @close="handleClose"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('asset.assetOrIssuer')" prop="input">
        <el-input
          v-model="form.input"
          :placeholder="t('asset.assetOrIssuerPlaceholder')"
          clearable
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <div class="hint-text">{{ t('asset.assetOrIssuerHint') }}</div>
      </el-form-item>
    </el-form>

    <!-- 添加结果预览 -->
    <div v-if="previewAssets.length > 0" class="preview-section">
      <div class="preview-header">{{ t('asset.foundAssets') }} ({{ previewAssets.length }})</div>
      <div class="preview-list">
        <div v-for="asset in previewAssets" :key="asset.asset" class="preview-item">
          <div class="preview-name">{{ asset.name }}</div>
          <div class="preview-id text-mono">{{ truncateHash(asset.asset, 10) }}</div>
          <div class="preview-issuer text-mono">{{ asset.accountRS }}</div>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" :disabled="!form.input" @click="handleSubmit">
        {{ t('asset.addBookmark') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * AddAssetBookmarkModal 组件 —— 添加资产收藏弹窗。
 *
 * 对标 nrs.assetexchange.js:153 NRS.forms.addAssetBookmark：
 *   支持两种输入：
 *     1. 资产 ID（纯数字）→ 调 getAsset 查询并加入收藏
 *     2. 发行方 RS 地址（NRCS-XXXX-...）→ 调 getAssetsByIssuer 批量查询并加入收藏
 *
 * 添加前先预览找到的资产列表，确认后写入 IndexedDB。
 */
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { useAssetBookmarks, type AssetBookmark } from '@/composables/useAssetBookmarks'
import { truncateHash } from '@/utils/format'

const { t } = useI18n()
const { addBookmark } = useAssetBookmarks()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success', added: AssetBookmark[]): void }>()

const formRef = ref<FormInstance>()
const loading = ref(false)
const form = reactive({ input: '' })

/** 预览找到的资产列表 */
const previewAssets = ref<AssetBookmark[]>([])

const rules = computed<FormRules>(() => ({
  input: [
    { required: true, message: t('asset.assetOrIssuerRequired'), trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: (err?: Error) => void) => {
        const trimmed = value.trim()
        if (!trimmed) {
          callback(new Error(t('asset.assetOrIssuerRequired')))
        } else if (!/^\d+$/.test(trimmed) && !/^NRCS-/i.test(trimmed)) {
          callback(new Error(t('asset.assetOrIssuerInvalid')))
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
}))

/**
 * 提交添加收藏。
 *
 * 对标 NRS.forms.addAssetBookmark（:153-209）的完整流程：
 *   - RS 地址 → getAssetsByIssuer 批量
 *   - 纯数字 → getAsset 单个
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const added = await addBookmark(form.input)
      if (added.length === 0) {
        ElMessage.warning(t('asset.bookmarkAlreadyExists'))
      } else {
        ElMessage.success(
          t('asset.bookmarkAddedSuccess', { count: added.length }),
        )
        previewAssets.value = added
        emit('success', added)
        // 延迟关闭，让用户看到添加结果
        setTimeout(() => handleClose(), 800)
      }
    } catch (e: any) {
      ElMessage.error(e?.message || t('asset.bookmarkAddError'))
    } finally {
      loading.value = false
    }
  })
}

function handleClose(): void {
  formRef.value?.resetFields()
  form.input = ''
  previewAssets.value = []
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.hint-text {
  font-size: $font-size-xs;
  color: $text-muted;
  margin-top: 4px;
}

.preview-section {
  margin-top: $space-md;
  padding: $space-md;
  background: var(--el-fill-color-light);
  border-radius: $radius-sm;

  .preview-header {
    font-size: $font-size-sm;
    color: $text-muted;
    margin-bottom: $space-sm;
  }

  .preview-list {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
  }

  .preview-item {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: $space-sm;
    padding: $space-xs $space-sm;
    background: var(--el-bg-color);
    border-radius: $radius-sm;
    font-size: $font-size-sm;
    align-items: center;

    .preview-name {
      font-weight: 500;
      color: $text-primary;
    }

    .preview-id,
    .preview-issuer {
      color: $text-muted;
      font-size: $font-size-xs;
    }
  }
}

.text-mono {
  font-family: $font-mono;
}
</style>
