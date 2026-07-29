<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><DataLine /></el-icon> {{ t('voting.activePolls') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showCreate = true">
          <el-icon><Plus /></el-icon> {{ t('voting.createPoll') }}
        </el-button>
        <el-button size="small" @click="refreshNow">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <!-- Stats bar -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-value">{{ items.length }}</div>
          <div class="stat-label">{{ t('voting.activePolls') }}</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-value">{{ currentHeight.toLocaleString() }}</div>
          <div class="stat-label">{{ t('voting.currentHeight', 'Current Height') }}</div>
        </el-card>
      </el-col>
    </el-row>

    <el-card shadow="hover" v-loading="loading">
      <el-table
        ref="tableRef"
        :data="items"
        :row-key="(row: any) => row.poll"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
      >
        <el-table-column type="expand">
          <template #default="{ row }">
            <div v-if="row._results && row._votes" class="results-panel">
              <h4 class="results-title">{{ t('voting.pollResults', 'Poll Results') }}</h4>
              <el-table :data="row._results" stripe size="small" style="width: 100%; margin-bottom: 16px">
                <el-table-column prop="option" :label="t('voting.option', 'Option')" min-width="160" />
                <el-table-column prop="weight" :label="t('voting.weight', 'Weight')" width="120" align="right" />
                <el-table-column prop="result" :label="t('voting.result', 'Result')" min-width="200" />
              </el-table>
              <h4 class="results-title">{{ t('voting.voteDistribution', 'Vote Distribution') }}</h4>
              <v-chart :option="buildChartOption(row)" style="height: 300px" />
              <h4 class="results-title" style="margin-top: 16px">
                {{ t('voting.castVotes', 'Cast Votes') }} ({{ row._votes.length }})
              </h4>
              <el-table :data="row._votes" stripe size="small" style="width: 100%">
                <el-table-column prop="voterRS" :label="t('voting.voter', 'Voter')" width="200" show-overflow-tooltip />
                <el-table-column :label="t('voting.voteValue', 'Vote')" width="200">
                  <template #default="{ row: vr }">{{ vr.votes?.join(', ') || '-' }}</template>
                </el-table-column>
                <el-table-column :label="t('common.transaction')" min-width="180" show-overflow-tooltip>
                  <template #default="{ row: vr }">{{ truncateHash(vr.transaction) }}</template>
                </el-table-column>
                <el-table-column :label="t('common.date')" width="160">
                  <template #default="{ row: vr }">{{ formatTimestamp(vr.timestamp) }}</template>
                </el-table-column>
              </el-table>
            </div>
            <div v-else-if="row._loadingResults" class="results-loading">
              <el-icon class="is-loading"><Loading /></el-icon> {{ t('common.loading') }}
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="name" :label="t('voting.name')" min-width="180" />
        <el-table-column prop="description" :label="t('voting.description')" min-width="220" show-overflow-tooltip />
        <el-table-column :label="t('voting.creator', 'Creator')" width="200" show-overflow-tooltip>
          <template #default="{ row }">{{ row.accountRS || row.account }}</template>
        </el-table-column>
        <el-table-column :label="t('voting.startDate', 'Start Date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('voting.blocksRemaining', 'Blocks Left')" width="130" align="center">
          <template #default="{ row }">
            <el-tag :type="row.finishHeight - currentHeight <= 0 ? 'danger' : (row.finishHeight - currentHeight < 144 ? 'warning' : 'success')">
              {{ Math.max(0, row.finishHeight - currentHeight) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="180" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openVote(row)">{{ t('voting.vote') }}</el-button>
            <el-button size="small" text type="success" @click="toggleResults(row)">{{ t('voting.results') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination
          v-model:current-page="page"
          :page-size="20"
          :total="total"
          layout="prev,pager,next"
          @current-change="refreshData"
        />
      </div>
    </el-card>

    <CreatePollModal v-model:visible="showCreate" @success="refreshData" />
    <CastVoteModal v-model:visible="showVote" :poll="selectedPoll" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Loading } from '@element-plus/icons-vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { BarChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp, truncateHash } from '@/utils/format'
import { usePolling } from '@/composables/usePolling'
import CreatePollModal from '@/components/modals/CreatePollModal.vue'
import CastVoteModal from '@/components/modals/CastVoteModal.vue'

use([CanvasRenderer, BarChart, GridComponent, TooltipComponent, LegendComponent])

const { t } = useI18n()
const tableRef = ref<any>(null)
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showCreate = ref(false)
const showVote = ref(false)
const selectedPoll = ref<any>(null)
const currentHeight = ref(0)

onMounted(async () => {
  try {
    const status = await nrcsApi.getBlockchainStatus()
    currentHeight.value = (status as any).numberOfBlocks || (status as any).lastBlockHeight || 0
  } catch { /* ignore */ }
  refreshData()
})

const { refreshNow } = usePolling(async () => {
  try {
    const status = await nrcsApi.getBlockchainStatus()
    currentHeight.value = (status as any).numberOfBlocks || (status as any).lastBlockHeight || 0
  } catch { /* ignore */ }
  refreshData()
}, 30000, false)

async function refreshData() {
  loading.value = true
  try {
    const result = await nrcsApi.getPolls((page.value - 1) * 20, page.value * 20 - 1)
    const polls = (result as any).polls || []
    items.value = polls.filter((p: any) => p.finishHeight > currentHeight.value)
    total.value = items.value.length
    // Merge back expanded results state from existing items
    const prevMap = new Map<string, any>()
    for (const prev of items.value) {
      if (prev._results) prevMap.set(prev.poll, { _results: prev._results, _votes: prev._votes })
    }
    items.value = items.value.map((p: any) => {
      const prev = prevMap.get(p.poll)
      return prev ? { ...p, _results: prev._results, _votes: prev._votes } : p
    })
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function openVote(row: any) {
  selectedPoll.value = row
  showVote.value = true
}

async function toggleResults(row: any) {
  if (row._results && row._votes) {
    tableRef.value?.toggleRowExpansion(row)
    return
  }
  row._loadingResults = true
  tableRef.value?.toggleRowExpansion(row)
  try {
    const [resultRes, votesRes] = await Promise.all([
      nrcsApi.getPollResult(row.poll).catch(() => null),
      nrcsApi.getPollVotes(row.poll, 0, 99).catch(() => null),
    ])
    row._results = (resultRes as any)?.results || []
    row._votes = (votesRes as any)?.votes || []
  } catch {
    row._results = []
    row._votes = []
  } finally {
    row._loadingResults = false
  }
}

function buildChartOption(row: any) {
  if (!row._results || !Array.isArray(row._results)) {
    return {}
  }
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
    xAxis: {
      type: 'category',
      data: row._results.map((r: any) => r.option || ''),
      axisLabel: { color: '#909399' },
    },
    yAxis: {
      type: 'value',
      axisLabel: { color: '#909399' },
    },
    series: [
      {
        name: t('voting.weight', 'Weight'),
        type: 'bar',
        data: row._results.map((r: any) => Number(r.weight) || 0),
        itemStyle: {
          color: {
            type: 'linear',
            x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              { offset: 0, color: '#409eff' },
              { offset: 1, color: '#66b1ff' },
            ],
          },
        },
      },
    ],
  }
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-md;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
    }
  }
  .stats-row {
    margin-bottom: $space-md;
  }
  .stat-card {
    text-align: center;
    .stat-value {
      font-size: 24px;
      font-weight: 700;
      color: $primary;
    }
    .stat-label {
      font-size: $font-size-sm;
      color: $text-muted;
      margin-top: $space-xs;
    }
  }
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: $space-md;
  }
}

.results-panel {
  padding: $space-lg;
  .results-title {
    font-size: $font-size-lg;
    font-weight: 600;
    margin-bottom: $space-md;
    color: $text-primary;
  }
}

.results-loading {
  padding: 40px;
  text-align: center;
  color: $text-muted;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
}
</style>
