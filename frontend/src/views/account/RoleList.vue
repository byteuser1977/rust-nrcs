<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><UserFilled /></el-icon> {{ t('account.roles') || 'Roles' }}</h2>
    </div>
    <el-card shadow="hover">
      <el-table :data="roles" stripe size="small" style="width:100%">
        <el-table-column prop="name" :label="t('common.name')" width="180" />
        <el-table-column prop="description" :label="t('common.description')" min-width="300" />
        <el-table-column :label="t('account.permissions') || 'Permissions'" width="120" align="center">
          <template #default="{ row }">
            <el-tag v-for="p in row.permissions" :key="p" size="small" class="perm-tag">{{ p }}</el-tag>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
/**
 * 角色管理页面（静态参考）
 *
 * 注：NRCS 链上无独立"角色"系统，此页面为角色模型参考表。
 */
import { UserFilled } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
const { t } = useI18n()

const roles = [
  { name: 'admin', description: t('account.roleAdminDesc') || 'Full system access', permissions: ['view', 'send', 'issue', 'forge', 'settings'] },
  { name: 'user', description: t('account.roleUserDesc') || 'Standard user', permissions: ['view', 'send'] },
  { name: 'forger', description: t('account.roleForgerDesc') || 'Block forger', permissions: ['view', 'send', 'forge'] },
]
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container {
  .page-header { margin-bottom: 16px;
    .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; }
  }
  .perm-tag { margin-right: 4px; margin-bottom: 2px; }
}
</style>
