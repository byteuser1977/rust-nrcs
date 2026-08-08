<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Setting /></el-icon>
        {{ t('settings.generalSettings') }}
      </h2>
      <div class="header-actions">
        <el-button size="small" @click="resetToDefaults">
          <el-icon><RefreshLeft /></el-icon>
          {{ t('settings.resetDefaults') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" class="settings-card">
      <el-form
        label-position="right"
        label-width="200px"
        class="settings-form"
      >
        <!-- 每页显示条数 -->
        <el-form-item :label="t('settings.itemsPerPage')">
          <el-select
            :model-value="settingsStore.settings.items_page"
            @change="(val: string) => settingsStore.updateSetting('items_page', val)"
            style="width: 120px"
          >
            <el-option label="10" value="10" />
            <el-option label="15" value="15" />
            <el-option label="25" value="25" />
            <el-option label="50" value="50" />
            <el-option label="100" value="100" />
          </el-select>
        </el-form-item>

        <!-- 回车提交表单 -->
        <el-form-item :label="t('settings.submitOnEnter')">
          <el-switch
            :model-value="settingsStore.submitOnEnter"
            @change="(val: boolean) => settingsStore.updateSetting('submit_on_enter', val ? '1' : '0')"
          />
        </el-form-item>

        <!-- 锻造动画 -->
        <el-form-item :label="t('settings.animateForging')">
          <el-switch
            :model-value="settingsStore.animateForging"
            @change="(val: boolean) => settingsStore.updateSetting('animate_forging', val ? '1' : '0')"
          />
        </el-form-item>

        <!-- 24小时格式 -->
        <el-form-item :label="t('settings.hourFormat')">
          <el-select
            :model-value="settingsStore.settings['24_hour_format']"
            @change="(val: string) => settingsStore.updateSetting('24_hour_format', val)"
            style="width: 160px"
          >
            <el-option :label="t('settings.format24')" value="1" />
            <el-option :label="t('settings.format12')" value="0" />
          </el-select>
        </el-form-item>

        <!-- 小数位数 -->
        <el-form-item :label="t('settings.maxDecimals')">
          <el-input-number
            :model-value="parseInt(settingsStore.settings.max_nxt_decimals, 10)"
            @change="(val: number | undefined) => settingsStore.updateSetting('max_nxt_decimals', String(val ?? 2))"
            :min="0"
            :max="8"
            controls-position="right"
            style="width: 120px"
          />
        </el-form-item>

        <!-- 记住解密密码 -->
        <el-form-item :label="t('settings.rememberDecryptionPassphrase')">
          <el-switch
            :model-value="settingsStore.rememberDecryptionPassphrase"
            @change="(val: boolean) => settingsStore.updateSetting('remember_decryption_passphrase', val ? '1' : '0')"
          />
        </el-form-item>

        <!-- 启用插件 -->
        <el-form-item :label="t('settings.enablePlugins')">
          <el-switch
            :model-value="settingsStore.enablePlugins"
            @change="(val: boolean) => settingsStore.updateSetting('enable_plugins', val ? '1' : '0')"
          />
        </el-form-item>

        <!-- 虚假实体警告 -->
        <el-form-item :label="t('settings.fakeEntityWarning')">
          <el-switch
            :model-value="settingsStore.settings.fake_entity_warning === '1'"
            @change="(val: boolean) => settingsStore.updateSetting('fake_entity_warning', val ? '1' : '0')"
          />
        </el-form-item>

        <!-- 控制台日志 -->
        <el-form-item :label="t('settings.consoleLog')">
          <el-switch
            :model-value="settingsStore.consoleLog"
            @change="(val: boolean) => settingsStore.updateSetting('console_log', val ? '1' : '0')"
          />
        </el-form-item>

        <el-divider>{{ t('settings.warningThresholds') }}</el-divider>

        <!-- 费用警告阈值 -->
        <el-form-item :label="t('settings.feeWarning')">
          <el-input
            :model-value="feeWarningNXT"
            @change="(val: string) => settingsStore.updateSetting('fee_warning', nxtToNqt(val))"
            style="width: 200px"
          >
            <template #append>NRC</template>
          </el-input>
        </el-form-item>

        <!-- 金额警告阈值 -->
        <el-form-item :label="t('settings.amountWarning')">
          <el-input
            :model-value="amountWarningNXT"
            @change="(val: string) => settingsStore.updateSetting('amount_warning', nxtToNqt(val))"
            style="width: 200px"
          >
            <template #append>NRC</template>
          </el-input>
        </el-form-item>

        <!-- 资产转移警告 -->
        <el-form-item :label="t('settings.assetTransferWarning')">
          <el-input
            :model-value="settingsStore.settings.asset_transfer_warning"
            @change="(val: string) => settingsStore.updateSetting('asset_transfer_warning', val)"
            style="width: 200px"
          >
            <template #append>QNT</template>
          </el-input>
        </el-form-item>

        <!-- 积分转移警告 -->
        <el-form-item :label="t('settings.currencyTransferWarning')">
          <el-input
            :model-value="settingsStore.settings.currency_transfer_warning"
            @change="(val: string) => settingsStore.updateSetting('currency_transfer_warning', val)"
            style="width: 200px"
          >
            <template #append>QNT</template>
          </el-input>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
/**
 * 全局设置页面 —— 对标 nrs.settings.js NRS.pages.settings
 *
 * 功能：
 *   - 显示所有配置项（对标 defaultSettings 22+ 配置项）
 *   - 实时保存到 localStorage（对标 NRS.updateSettings）
 *   - 重置为默认值
 *   - 费用/金额警告阈值的 NXT ↔ NQT 转换
 */
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Setting, RefreshLeft } from '@element-plus/icons-vue'
import { useSettingsStore } from '@/stores/modules/settings.store'
import { formatNqtToNrc, nxtToNqt } from '@/utils/format'

const { t } = useI18n()
const settingsStore = useSettingsStore()

/** 费用警告阈值（NQT → NXT 显示） */
const feeWarningNXT = computed(() => {
  return formatNqtToNrc(settingsStore.settings.fee_warning)
})

/** 金额警告阈值（NQT → NXT 显示） */
const amountWarningNXT = computed(() => {
  return formatNqtToNrc(settingsStore.settings.amount_warning)
})

/**
 * 重置所有设置为默认值。
 */
async function resetToDefaults() {
  try {
    await ElMessageBox.confirm(
      t('settings.resetDefaultsConfirm'),
      t('settings.resetDefaults'),
      { type: 'warning' }
    )
    settingsStore.resetSettings()
    ElMessage.success(t('settings.resetSuccess'))
  } catch {
    // 用户取消
  }
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-lg;
    .page-title {
      font-size: $font-size-lg;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
      color: $text-primary;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
    }
  }
}

.settings-card {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: rgba($surface-800, 0.8) !important;
}

.settings-form {
  max-width: 640px;

  :deep(.el-form-item) {
    margin-bottom: $space-lg;
  }

  :deep(.el-form-item__label) {
    color: $text-secondary;
    font-size: $font-size-sm;
  }
}

:deep(.el-divider__text) {
  color: $text-muted;
  font-size: $font-size-sm;
}
</style>
