<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Lock /></el-icon> {{ t('account.permissions') || 'Permissions' }}</h2>
    </div>
    <el-card shadow="hover">
      <el-table :data="permissions" stripe size="small" style="width:100%">
        <el-table-column prop="code" :label="t('account.permissionCode') || 'Code'" width="200" />
        <el-table-column prop="name" :label="t('account.permissionName') || 'Name'" width="200" />
        <el-table-column prop="resource" :label="t('account.resource') || 'Resource'" width="180" />
        <el-table-column :label="t('account.action') || 'Action'" width="100" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="actionType(row.action)">{{ row.action }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="description" :label="t('common.description')" min-width="200" />
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
/**
 * 权限管理页面（静态参考）
 *
 * 注：NRCS 链上无独立"权限"系统，此页面为权限模型参考表，
 * 展示 NRCS 账户操作对应的权限码/资源/动作。
 */
import { Lock } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
const { t } = useI18n()

function actionType(action: string): 'primary' | 'success' | 'warning' | 'info' | 'danger' {
  const map: Record<string, 'primary' | 'success' | 'warning' | 'info' | 'danger'> = { view: 'info', send: 'warning', issue: 'success', start: 'primary', settings: 'danger' }
  return map[action] || 'info'
}

const permissions = [
  { code: 'account.view', name: t('account.permViewAccount') || 'View Account', resource: 'account', action: 'view', description: t('account.permViewAccountDesc') || 'View account details and balances' },
  { code: 'account.send', name: t('account.permSendTx') || 'Send Transaction', resource: 'account', action: 'send', description: t('account.permSendTxDesc') || 'Send NRC, assets, and messages' },
  { code: 'asset.issue', name: t('account.permIssueAsset') || 'Issue Asset', resource: 'asset', action: 'issue', description: t('account.permIssueAssetDesc') || 'Create new assets on the blockchain' },
  { code: 'currency.issue', name: t('account.permIssueCurrency') || 'Issue Currency', resource: 'currency', action: 'issue', description: t('account.permIssueCurrencyDesc') || 'Create new MS currencies' },
  { code: 'forge.start', name: t('account.permForge') || 'Start Forging', resource: 'forge', action: 'start', description: t('account.permForgeDesc') || 'Participate in block forging' },
  { code: 'admin.settings', name: t('account.permAdminSettings') || 'Admin Settings', resource: 'admin', action: 'settings', description: t('account.permAdminSettingsDesc') || 'Modify node settings' },
]
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container {
  .page-header { margin-bottom: 16px;
    .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; }
  }
}
</style>
