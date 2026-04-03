<template>
  <div class="forbidden-page">
    <div class="error-container">
      <h1 class="error-code">403</h1>
      <h2 class="error-message">权限不足</h2>
      <p class="error-description">
        抱歉，您没有访问此页面的权限。
        <br v-if="!isLoggedIn">
        请先登录或联系管理员获取相应权限。
      </p>
      <div class="actions">
        <el-button type="primary" @click="goHome">
          返回首页
        </el-button>
        <el-button v-if="!isLoggedIn" @click="goLogin">
          去登录
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAccountStore } from '@/stores/account'

const router = useRouter()
const accountStore = useAccountStore()

const isLoggedIn = computed(() => accountStore.isLoggedIn)

const goHome = () => {
  router.push('/')
}

const goLogin = () => {
  router.push('/login')
}
</script>

<style lang="scss" scoped>
.forbidden-page {
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
      color: #f56c6c;
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
      line-height: 1.6;
    }

    .actions {
      display: flex;
      gap: 12px;
      justify-content: center;
    }
  }
}
</style>
