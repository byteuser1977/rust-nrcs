<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Lock /></el-icon> 权限管理</h2>
    </div>
    <el-card shadow="hover">
      <el-table :data="permissions" stripe size="small" style="width:100%">
        <el-table-column prop="code" label="权限码" width="200" />
        <el-table-column prop="name" label="权限名称" width="200" />
        <el-table-column prop="resource" label="资源" width="200" />
        <el-table-column prop="action" label="操作" width="120">
          <template #default="{ row }">
            <el-tag size="small">{{ row.action }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="description" label="描述" min-width="200" />
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
const permissions = [
  { code: 'account.view', name: '查看账户', resource: 'account', action: 'view', description: '查看账户信息' },
  { code: 'account.send', name: '发送交易', resource: 'account', action: 'send', description: '发送NRC和资产' },
  { code: 'asset.issue', name: '发行资产', resource: 'asset', action: 'issue', description: '创建新的资产' },
  { code: 'currency.issue', name: '发行积分', resource: 'currency', action: 'issue', description: '创建新积分' },
  { code: 'forge.start', name: '开始锻造', resource: 'forge', action: 'start', description: '开始区块锻造' },
  { code: 'admin.settings', name: '管理设置', resource: 'admin', action: 'settings', description: '修改系统设置' },
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
