<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAccountStore } from '@/stores/modules/account.store'
import { ElCard, ElDescriptions, ElDescriptionsItem, ElTag } from 'element-plus'
import { formatAddress } from '@/utils/format'

const { t } = useI18n()
const accountStore = useAccountStore()

const user = computed(() => accountStore.userInfo)
const isLoggedIn = computed(() => accountStore.isLoggedIn)
</script>

<template>
  <div class="profile">
    <h1 class="page-title">{{ t('account.profile') }}</h1>

    <el-card shadow="never" v-if="isLoggedIn && user">
      <el-descriptions :column="2" border>
        <el-descriptions-item :label="t('account.name')">
          {{ user.name }}
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.email')">
          {{ user.email }}
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.address')">
          <div class="address-cell">
            <code>{{ formatAddress(user.wallet_address, 8) }}</code>
          </div>
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.role')">
          <el-tag>{{ user.role }}</el-tag>
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.createdAt')" :span="2">
          {{ user.created_at }}
        </el-descriptions-item>
      </el-descriptions>
    </el-card>

    <el-empty v-else description="未登录" />
  </div>
</template>

<style lang="scss" scoped>
.profile {
  .page-title {
    margin-bottom: 24px;
    font-size: 24px;
    font-weight: 600;
  }

  .address-cell {
    code {
      font-family: 'Monaco', 'Consolas', monospace;
      background: #f5f5f5;
      padding: 2px 6px;
      border-radius: 4px;
      color: #409eff;
    }
  }
}
</style>
