<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useContractStore } from '@/stores/modules/contract.store'
import { ElCard, ElTable, ElTableColumn, ElTag, ElEmpty, ElButton } from 'element-plus'
import { useRouter } from 'vue-router'
import { formatAddress, formatTime } from '@/utils/format'

const { t } = useI18n()
const contractStore = useContractStore()
const router = useRouter()

onMounted(() => {
  loadContracts()
})

async function loadContracts() {
  await contractStore.fetchContracts()
}

function viewDetail(address: string) {
  router.push(`/contract/detail/${address}`)
}

function goToDeploy() {
  router.push('/contract/deploy')
}
</script>

<template>
  <div class="contract-list">
    <div class="header-row">
      <h1 class="page-title">{{ t('contract.list') }}</h1>
      <el-button type="primary" @click="goToDeploy">
        <el-icon><Plus /></el-icon>
        {{ t('contract.deploy') }}
      </el-button>
    </div>

    <el-card shadow="never" v-loading="contractStore.isLoading">
      <el-table :data="contractStore.contracts" style="width: 100%">
        <el-table-column prop="address" :label="t('contract.address')" width="220">
          <template #default="{ row }">
            <el-link type="primary" :underline="false" @click="viewDetail(row.address)">
              {{ formatAddress(row.address) }}
            </el-link>
          </template>
        </el-table-column>

        <el-table-column prop="name" :label="t('contract.name')" width="150" />

        <el-table-column prop="deployer" :label="t('contract.deployer')" width="180">
          <template #default="{ row }">
            {{ formatAddress(row.deployer) }}
          </template>
        </el-table-column>

        <el-table-column prop="deployed_at" :label="t('contract.deployedAt')" width="180">
          <template #default="{ row }">
            {{ formatTime(row.deployed_at) }}
          </template>
        </el-table-column>

        <el-table-column prop="is_verified" :label="t('contract.verified')" width="100">
          <template #default="{ row }">
            <el-tag :type="row.is_verified ? 'success' : 'warning'" size="small">
              {{ row.is_verified ? t('contract.verified') : t('contract.unverified') }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column :label="t('common.operation')" width="150" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" size="small" @click="viewDetail(row.address)">
              {{ t('common.view') || '查看' }}
            </el-button>
            <el-button link type="primary" size="small" disabled>
              {{ t('contract.call') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <el-empty v-if="!contractStore.isLoading && contractStore.contracts.length === 0" :description="t('common.noData')" />
    </el-card>
  </div>
</template>

<style lang="scss" scoped>
.contract-list {
  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;

    .page-title {
      margin: 0;
      font-size: 24px;
      font-weight: 600;
    }
  }
}
</style>
