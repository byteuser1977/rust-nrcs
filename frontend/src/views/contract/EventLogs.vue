<template>
  <div class="event-logs">
    <div class="filter-bar">
      <el-form :model="filters" inline>
        <el-form-item label="事件名称">
          <el-input
            v-model="filters.eventName"
            placeholder="事件名称"
            clearable
            style="width: 180px"
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchEvents">
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

    <el-table :data="events" style="width: 100%" v-loading="loading">
      <el-table-column prop="block_number" label="区块" width="100" />
      <el-table-column prop="log_index" label="日志索引" width="100" />
      <el-table-column prop="event_name" label="事件" width="150">
        <template #default="{ row }">
          <el-tag type="info" size="small">{{ row.event_name }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="args" label="参数" min-width="300">
        <template #default="{ row }">
          <pre class="args-pre">{{ formatArgs(row.args) }}</pre>
        </template>
      </el-table-column>
      <el-table-column prop="block_timestamp" label="时间" width="180">
        <template #default="{ row }">
          {{ formatDateTime(row.block_timestamp) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="120" fixed="right">
        <template #default="{ row }">
          <el-button
            size="small"
            type="primary"
            link
            @click="viewTransaction(row.transaction_hash)"
          >
            查看交易
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-wrapper" v-if="total > 0">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[10, 20, 50]"
        :total="total"
        layout="total, sizes, prev, pager, next, jumper"
        @current-change="fetchEvents"
        @size-change="handleSizeChange"
      />
    </div>

    <el-empty
      v-if="events.length === 0 && !loading"
      description="暂无事件日志"
      :image-size="100"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Search, Refresh } from '@element-plus/icons-vue'
import { contractApi } from '@/api/modules'
import type { ContractEvent } from '@/types/business'
import { formatTime } from '@/utils/format'

const props = defineProps<{
  contractAddress: string
}>()

const router = useRouter()

const events = ref<ContractEvent[]>([])
const loading = ref(false)
const currentPage = ref(1)
const pageSize = ref(20)
const total = ref(0)

const filters = reactive({
  eventName: ''
})

const fetchEvents = async () => {
  try {
    loading.value = true
    const response = await contractApi.getContractEvents({
      contractAddress: props.contractAddress,
      eventName: filters.eventName || undefined,
      page: currentPage.value,
      size: pageSize.value
    })
    events.value = response.data.list || response.data.items || []
    total.value = response.data.total || 0
  } catch (error: any) {
    console.error('Failed to fetch events:', error)
    ElMessage.error('获取事件日志失败')
  } finally {
    loading.value = false
  }
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  fetchEvents()
}

const resetFilters = () => {
  filters.eventName = ''
  currentPage.value = 1
  fetchEvents()
}

const viewTransaction = (txHash: string) => {
  router.push(`/transaction/detail/${txHash}`)
}

const formatArgs = (args: Record<string, any>): string => {
  return JSON.stringify(args, null, 2)
}

const formatDateTime = (timestamp: number): string => {
  return formatTime(new Date(timestamp * 1000).toISOString())
}

onMounted(() => {
  fetchEvents()
})
</script>

<style scoped lang="scss">
.event-logs {
  .filter-bar {
    margin-bottom: 16px;
    padding: 16px;
    background: #fff;
    border-radius: 4px;

    .el-form-item {
      margin-bottom: 12px;
    }
  }

  .args-pre {
    margin: 0;
    font-size: 12px;
    font-family: 'Roboto Mono', monospace;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .pagination-wrapper {
    margin-top: 16px;
    display: flex;
    justify-content: flex-end;
  }
}
</style>
