<template>
  <div class="not-found-page">
    <div class="error-container">
      <h1 class="error-code">404</h1>
      <h2 class="error-message">页面不存在</h2>
      <p class="error-description">
        抱歉，您访问的页面不存在或已被移动。
      </p>
      <el-button type="primary" @click="goHome">
        返回首页
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const goHome = () => {
  router.push('/')
}
</script>

<style lang="scss" scoped>
.not-found-page {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100vh;
  background: #f5f7fa;

  .error-container {
    text-align: center;
    padding: 40px;

    .error-code {
      font-size: 120px;
      font-weight: bold;
      color: #409eff;
      margin: 0;
      line-height: 1;
    }

    .error-message {
      font-size: 24px;
      color: #303133;
      margin: 16px 0;
    }

    .error-description {
      font-size: 16px;
      color: #909399;
      margin-bottom: 32px;
    }
  }
}
</style>
