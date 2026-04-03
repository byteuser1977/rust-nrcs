<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElButton } from 'element-plus'

const { t } = useI18n()
const router = useRouter()

function goHome() {
  router.push('/')
}
</script>

<template>
  <div class="error-page not-found">
    <div class="error-content">
      <div class="error-code">404</div>
      <h1 class="error-title">{{ t('error.notFound') }}</h1>
      <p class="error-desc">{{ t('error.notFoundDesc') }}</p>
      <el-button type="primary" @click="goHome">
        {{ t('error.backHome') }}
      </el-button>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.error-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #0a0a0a;

  .error-content {
    text-align: center;
    padding: 40px;

    .error-code {
      font-size: 120px;
      font-weight: 700;
      color: #409eff;
      line-height: 1;
      margin-bottom: 24px;
      text-shadow: 0 4px 20px rgba(64, 158, 255, 0.3);
    }

    .error-title {
      font-size: 24px;
      color: #fff;
      margin: 0 0 12px;
    }

    .error-desc {
      font-size: 14px;
      color: #909399;
      margin: 0 0 32px;
    }
  }
}
</style>
