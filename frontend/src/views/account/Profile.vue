<script setup lang="ts">
/**
 * 账户资料页面
 *
 * 展示当前登录账户的链上信息（对标 NRCS getAccount 返回的字段）：
 * 账户名 / 描述 / RS 地址 / 数字账户 ID / 公钥 / 余额 / 有效余额。
 * 数据全部来自 account store，可手动刷新。
 */
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { formatNrc, truncateHash } from '@/utils/format'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)

const isLoggedIn = computed(() => accountStore.isLoggedIn)
const name = computed(() => accountStore.name || '—')
const description = computed(() => accountStore.description || '—')
const accountRS = computed(() => accountStore.accountRS)
const accountId = computed(() => accountStore.accountId)
const publicKey = computed(() => accountStore.publicKey)
const balanceNQT = computed(() => accountStore.balanceNQT)
const effectiveBalance = computed(() => accountStore.effectiveBalance)

// 挂载时拉取一次完整账户信息（失败不阻断，展示已有状态）
onMounted(() => {
  if (accountStore.isLoggedIn) {
    refresh()
  }
})

/**
 * 刷新账户链上信息（完整模式：含租赁/控制/资产状态）
 */
async function refresh() {
  if (!accountStore.accountRS) return
  loading.value = true
  try {
    await accountStore.getAccountInfo(accountStore.accountRS)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="profile">
    <div class="page-header">
      <h1 class="page-title">{{ t('account.profile') }}</h1>
      <div class="header-actions">
        <el-button size="small" :loading="loading" @click="refresh">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="never" v-if="isLoggedIn" v-loading="loading">
      <el-descriptions :column="2" border>
        <el-descriptions-item :label="t('account.name')">
          {{ name }}
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.balance')">
          {{ formatNrc(balanceNQT) }} NRC
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.rsAddress')" :span="2">
          <code class="mono">{{ accountRS }}</code>
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.accountId')">
          <code class="mono">{{ accountId }}</code>
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.effectiveBalance')">
          {{ effectiveBalance }} NRC
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.publicKey')" :span="2">
          <code class="mono">{{ truncateHash(publicKey, 16) }}</code>
        </el-descriptions-item>

        <el-descriptions-item :label="t('account.description')" :span="2">
          {{ description }}
        </el-descriptions-item>
      </el-descriptions>
    </el-card>

    <el-empty v-else :description="t('account.notLoggedIn')" />
  </div>
</template>

<style lang="scss" scoped>
.profile {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;

    .page-title {
      margin: 0;
      font-size: 24px;
      font-weight: 600;
    }

    .header-actions {
      display: flex;
      gap: 8px;
    }
  }

  .mono {
    font-family: 'Monaco', 'Consolas', monospace;
    word-break: break-all;
  }
}
</style>
