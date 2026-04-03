<template>
  <div class="user-profile-page">
    <el-card>
      <template #header>
        <span>个人资料</span>
      </template>

      <el-form :model="form" label-width="100px">
        <el-form-item label="用户名">
          <el-input v-model="form.username" disabled />
        </el-form-item>
        <el-form-item label="邮箱">
          <el-input v-model="form.email" />
        </el-form-item>
        <el-form-item label="姓名">
          <el-input v-model="form.fullName" />
        </el-form-item>
        <el-form-item label="角色">
          <el-tag v-for="role in form.roles" :key="role" style="margin-right: 8px;">
            {{ role }}
          </el-tag>
        </el-form-item>
        <el-form-item>
          <el-button type="primary">保存修改</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'

const form = reactive({
  username: 'admin',
  email: 'admin@example.com',
  fullName: '管理员',
  roles: ['admin']
})
</script>
