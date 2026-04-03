<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { ElCard, ElForm, ElFormItem, ElInput, ElButton } from 'element-plus'

const { t } = useI18n()

// 简化实现
</script>

<template>
  <div class="security">
    <h1 class="page-title">{{ t('account.security') }}</h1>

    <el-card shadow="never">
      <el-form label-width="120px">
        <el-form-item :label="t('login.password')">
          <el-input type="password" show-password />
        </el-form-item>
        <el-form-item :label="t('login.confirmPassword')">
          <el-input type="password" show-password />
        </el-form-item>
        <el-form-item>
          <el-button type="primary">{{ t('account.changePassword') }}</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<style lang="scss" scoped>
.security {
  .page-title {
    margin-bottom: 24px;
    font-size: 24px;
    font-weight: 600;
  }
}
</style>
