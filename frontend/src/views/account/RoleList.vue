<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><UserFilled /></el-icon> 角色管理</h2>
    </div>
    <el-card shadow="hover">
      <el-table :data="roles" stripe size="small" style="width:100%">
        <el-table-column prop="name" label="角色名称" width="200" />
        <el-table-column prop="description" label="描述" min-width="300" />
        <el-table-column label="权限数" width="100" align="center">
          <template #default="{ row }">{{ row.permissions?.length || 0 }}</template>
        </el-table-column>
        <el-table-column label="用户数" width="100" align="center">
          <template #default="{ row }">{{ row.userCount || 0 }}</template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
const roles = [
  { name: 'admin', description: '系统管理员，拥有全部权限', permissions: ['*'], userCount: 1 },
  { name: 'user', description: '普通用户', permissions: ['view', 'send'], userCount: 0 },
  { name: 'forger', description: '锻造者角色', permissions: ['view', 'send', 'forge'], userCount: 0 },
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
