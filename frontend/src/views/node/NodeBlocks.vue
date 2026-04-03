<template>
  <div class="node-blocks-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon><Grid /></el-icon>
            <span>区块浏览</span>
          </div>
          <div class="header-right">
            <el-button type="primary" @click="fetchBlocks(1)">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </template>

      <!-- 筛选工具栏 -->
      <div class="filter-bar">
        <el-form :model="filters" inline>
          <el-form-item label="区块高度">
            <el-input
              v-model="filters.blockNumber"
              placeholder="输入区块高度"
              style="width: 160px"
              @keyup.enter="fetchBlocks(1)"
            />
          </el-form-item>
          <el-form-item label="矿工地址">
            <el-input
              v-model="filters.miner"
              placeholder="矿工地址"
              style="width: 200px"
              @keyup.enter="fetchBlocks(1)"
            />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="fetchBlocks(1)">
              <el-icon><Search /></el-icon>
              查询
            </el-button>
            <el-button @click="resetFilters">
              <el-icon><Refresh /></el-icon>
              重置
            </el-button>
          </el-form-item>
        </el-form>
      </div>

      <!-- 区块列表 -->
      <el-table :data="blocks" style="width: 100%" v-loading="loading">
        <el-table-column prop="number" label="高度" width="100" sortable>
          <template #default="{ row }">
            <el-link
              type="primary"
              :underline="false"
              @click="viewBlockDetail(row.number)"
            >
              {{ row.number.toLocaleString() }}
            </el-link>
          </template>
        </el-table-column>
        <el-table-column prop="hash" label="区块哈希" min-width="200">
          <template #default="{ row }">
            <el-tooltip :content="row.hash" placement="top">
              <span class="hash-text">{{ formatHash(row.hash) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column prop="miner" label="矿工" min-width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.miner" placement="top">
              <span class="address-text">{{ formatAddress(row.miner) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column prop="transaction_count" label="交易数" width="100" />
        <el-table-column prop="gas_used" label="Gas消耗" width="120">
          <template #default="{ row }">
            {{ row.gas_used.toLocaleString() }}
          </template>
        </el-table-column>
        <el-table-column prop="timestamp" label="时间" width="180">
          <template #default="{ row }">
            {{ formatTime(row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              type="primary"
              link
              @click="viewBlockDetail(row.number)"
            >
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper" v-if="total > 0">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :total="total"
          layout="total, sizes, prev, pager, next, jumper"
          @current-change="fetchBlocks"
          @size-change="handleSizeChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Grid, Refresh, Search } from '@element-plus/icons-vue'
import { nodeApi } from '@/api/modules'
import type { BlockInfo } from '@/types/business'
import { formatAddress, formatTime } from '@/utils/format'

const router = useRouter()

const blocks = ref<BlockInfo[]>([])
const loading = ref(false)
const currentPage = ref(1)
const pageSize = ref(20)
const total = ref(0)

const filters = reactive({
  blockNumber: '',
  miner: ''
})

const fetchBlocks = async (page: number) => {
  try {
    loading.value = true
    currentPage.value = page

    const params: any = {
      page,
      size: pageSize.value
    }

    if (filters.blockNumber) {
      params.fromNumber = parseInt(filters.blockNumber)
      params.toNumber = parseInt(filters.blockNumber)
    }
    if (filters.miner) {
      params.miner = filters.miner
    }

    const response = await nodeApi.getBlocks(params)
    blocks.value = response.data.items || response.data.list || []
    total.value = response.data.total || 0
  } catch (error: any) {
    console.error('Failed to fetch blocks:', error)
    ElMessage.error('获取区块列表失败')
  } finally {
    loading.value = false
  }
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  fetchBlocks(1)
}

const resetFilters = () => {
  filters.blockNumber = ''
  filters.miner = ''
  fetchBlocks(1)
}

const viewBlockDetail = (blockNumber: number) => {
  // TODO: 实现区块详情页面
  ElMessage.info(`查看区块 ${blockNumber} 详情（待实现）`)
}

const formatHash = (hash: string): string => {
  return `${hash.slice(0, 12)}...${hash.slice(-8)}`
}

const formatAddress = (addr: string): string => {
  return formatAddress(addr, 8)
}

const formatTime = (timestamp: number): string => {
  return formatTime(new Date(timestamp * 1000).toISOString())
}

onMounted(() => {
  fetchBlocks(1)
})
</script>

<style scoped lang="scss">
.node-blocks-page {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-left {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 16px;
      font-weight: 600;
    }
  }

  .filter-bar {
    margin-bottom: 16px;
    padding: 16px;
    background: #fff;
    border-radius: 4px;

    .el-form-item {
      margin-bottom: 12px;
    }
  }

  .hash-text {
    font-family: 'Roboto Mono', monospace;
    color: #409eff;
  }

  .address-text {
    font-family: 'Roboto Mono', monospace;
    color: #606266;
  }

  .pagination-wrapper {
    margin-top: 24px;
    display: flex;
    justify-content: flex-end;
  }
}
</style>
