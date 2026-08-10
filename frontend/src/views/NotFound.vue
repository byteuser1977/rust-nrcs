<template>
  <div class="not-found-page">
    <div class="error-container">
      <h1 class="error-code">404</h1>
      <h2 class="error-message">{{ t('error.notFound') }}</h2>
      <p class="error-description">{{ t('error.notFoundDesc') }}</p>
      <el-button type="primary" @click="goHome">{{ t('error.backHome') }}</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const router = useRouter()
const goHome = () => router.push('/')
</script>

<style lang="scss" scoped>
.not-found-page {
  display: flex; justify-content: center; align-items: center; width: 100%; height: 100vh; background: #f5f7fa;
  .error-container { text-align: center; padding: 40px;
    .error-code { font-size: 120px; font-weight: bold; color: #409eff; margin: 0; line-height: 1; }
    .error-message { font-size: 24px; color: #303133; margin: 16px 0; }
    .error-description { font-size: 16px; color: #909399; margin-bottom: 32px; }
  }
}
</style>
