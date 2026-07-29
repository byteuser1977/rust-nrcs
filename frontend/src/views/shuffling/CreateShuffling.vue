<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Plus /></el-icon>
        {{ t('shuffling.createShuffling') }}
      </h2>
    </div>

    <el-card shadow="hover" class="create-card">
      <div class="create-intro">
        <el-icon :size="48" color="$primary"><Connection /></el-icon>
        <h3>{{ t('shuffling.createShufflingDescription') }}</h3>
        <p class="text-muted">{{ t('shuffling.createShufflingHint') }}</p>
        <el-button type="primary" size="large" @click="showCreateModal = true">
          <el-icon><Plus /></el-icon>
          {{ t('shuffling.createShuffling') }}
        </el-button>
      </div>
    </el-card>

    <ShufflingCreateModal
      v-model:visible="showCreateModal"
      @success="onCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Plus, Connection } from '@element-plus/icons-vue'
import ShufflingCreateModal from '@/components/modals/ShufflingCreateModal.vue'

const { t } = useI18n()
const router = useRouter()

const showCreateModal = ref(false)

function onCreated() {
  ElMessage.success(t('common.operationSuccess'))
  router.push('/shuffling/my')
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  max-width: 600px;
  margin: 0 auto;

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-lg;
    .page-title {
      font-size: $font-size-lg;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
      color: $text-primary;
    }
  }
}

.create-card {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9)) !important;
}

.create-intro {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: $space-4xl $space-2xl;
  text-align: center;
  gap: $space-md;
  color: $primary;

  h3 {
    margin: 0;
    color: $text-primary;
    font-weight: 600;
    font-size: $font-size-md;
  }

  p {
    margin: 0 0 $space-lg 0;
  }
}

.text-muted {
  color: $text-muted;
}
</style>
