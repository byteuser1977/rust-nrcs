<template>
  <div class="issue-asset-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Coin /></el-icon> {{ t('asset.issueAsset') }}</h2>
    </div>

    <!-- Introduction -->
    <el-card shadow="hover" class="intro-card">
      <div class="intro-content">
        <el-icon class="intro-icon" :size="48" color="var(--el-color-primary)">
          <Coin />
        </el-icon>
        <h3>{{ t('asset.issueAssetTitle') }}</h3>
        <p class="intro-text">{{ t('asset.issueAssetDescription') }}</p>
        <ul class="intro-list">
          <li>{{ t('asset.issueAssetInfo1') }}</li>
          <li>{{ t('asset.issueAssetInfo2') }}</li>
          <li>{{ t('asset.issueAssetInfo3') }}</li>
          <li>{{ t('asset.issueAssetInfo4') }}</li>
        </ul>
        <el-button type="primary" size="large" @click="showIssueModal = true">
          <el-icon><Plus /></el-icon> {{ t('asset.issueAsset') }}
        </el-button>
      </div>
    </el-card>

    <IssueAssetModal v-model:visible="showIssueModal" @success="onIssueSuccess" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Coin, Plus } from '@element-plus/icons-vue'
import IssueAssetModal from '@/components/modals/IssueAssetModal.vue'

const { t } = useI18n()

const showIssueModal = ref(false)

function onIssueSuccess() {
  ElMessage.success(t('asset.issueAssetSuccess'))
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.issue-asset-page {
  .page-header {
    margin-bottom: 24px;
    .page-title {
      font-size: 20px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 0;
    }
  }

  .intro-card {
    max-width: 680px;
    .intro-content {
      display: flex;
      flex-direction: column;
      align-items: center;
      text-align: center;
      padding: 24px 16px;

      .intro-icon {
        margin-bottom: 16px;
      }

      h3 {
        font-size: 20px;
        font-weight: 600;
        margin: 0 0 12px 0;
        color: $text-primary;
      }

      .intro-text {
        font-size: 14px;
        color: $text-secondary;
        max-width: 480px;
        line-height: 1.6;
        margin: 0 0 20px 0;
      }

      .intro-list {
        text-align: left;
        margin: 0 0 24px 0;
        padding: 0 20px;
        color: $text-secondary;

        li {
          margin-bottom: 6px;
          font-size: 13px;
          line-height: 1.5;
        }
      }
    }
  }
}
</style>
