<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Star /></el-icon> {{ t('voting.followedPolls') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showAddDialog = true">
          <el-icon><Plus /></el-icon> {{ t('voting.addFollowedPoll', 'Add Followed Poll') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-row :gutter="16">
      <!-- Sidebar: followed polls list -->
      <el-col :span="6">
        <el-card shadow="hover" class="sidebar-card">
          <template #header>
            <div class="sidebar-header">
              <span>{{ t('voting.followedPolls') }}</span>
              <el-tag size="small" type="info">{{ followedPolls.length }}</el-tag>
            </div>
          </template>
          <div v-if="followedPolls.length === 0" class="empty-sidebar">
            {{ t('voting.noFollowedPolls', 'No followed polls yet. Add one using the button above.') }}
          </div>
          <div v-else class="followed-list">
            <div
              v-for="(fp, idx) in followedPolls"
              :key="fp.id"
              :class="['followed-item', { active: selectedPollId === fp.id }]"
              @click="selectPoll(fp)"
            >
              <div class="followed-item-content">
                <span class="poll-name">{{ fp.name || truncateHash(fp.id) }}</span>
                <span class="poll-id-hint">{{ truncateHash(fp.id, 10) }}</span>
              </div>
              <el-button
                size="small"
                text
                type="danger"
                @click.stop="unfollow(fp)"
              >
                <el-icon><Delete /></el-icon>
              </el-button>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- Main area: poll details -->
      <el-col :span="18">
        <el-card shadow="hover" v-if="selectedPoll" v-loading="detailLoading">
          <template #header>
            <div class="detail-header">
              <span class="detail-title">{{ selectedPoll.name || selectedPoll.id }}</span>
              <div class="detail-header-actions">
                <el-button
                  :type="isFollowing(selectedPoll.id) ? 'warning' : 'primary'"
                  size="small"
                  @click="toggleFollow(selectedPoll.id)"
                >
                  <el-icon><Star /></el-icon>
                  {{ isFollowing(selectedPoll.id)
                    ? t('voting.unfollow', 'Unfollow')
                    : t('voting.follow', 'Follow') }}
                </el-button>
                <el-button size="small" text type="primary" @click="openVote(selectedPoll)">
                  {{ t('voting.vote') }}
                </el-button>
              </div>
            </div>
          </template>

          <!-- Poll info -->
          <div class="poll-info-grid">
            <div class="info-item">
              <span class="info-label">{{ t('common.id') }}</span>
              <span class="info-value">{{ truncateHash(selectedPoll.id) }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('voting.description') }}</span>
              <span class="info-value">{{ selectedPoll._description || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('voting.creator', 'Creator') }}</span>
              <span class="info-value">{{ selectedPoll._accountRS || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('voting.finishHeight') }}</span>
              <span class="info-value">{{ selectedPoll._finishHeight || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('voting.status', 'Status') }}</span>
              <span class="info-value">
                <el-tag :type="selectedPoll._finished ? 'info' : 'success'" size="small">
                  {{ selectedPoll._finished
                    ? t('voting.finished', 'Finished')
                    : t('voting.active', 'Active') }}
                </el-tag>
              </span>
            </div>
          </div>

          <!-- Results table -->
          <h4 class="section-title">{{ t('voting.pollResults', 'Poll Results') }}</h4>
          <el-table :data="pollResults" stripe size="small" style="width: 100%; margin-bottom: 16px">
            <el-table-column prop="option" :label="t('voting.option', 'Option')" min-width="160" />
            <el-table-column prop="weight" :label="t('voting.weight', 'Weight')" width="120" align="right" />
            <el-table-column prop="result" :label="t('voting.result', 'Result')" min-width="200" />
          </el-table>
          <v-chart :option="resultsChartOption" style="height: 300px; margin-bottom: 16px" v-if="pollResults.length > 0" />

          <!-- Votes table -->
          <h4 class="section-title">
            {{ t('voting.castVotes', 'Cast Votes') }} ({{ pollVotes.length }})
          </h4>
          <el-table :data="pollVotes" stripe size="small" style="width: 100%">
            <el-table-column prop="voterRS" :label="t('voting.voter', 'Voter')" width="200" show-overflow-tooltip />
            <el-table-column :label="t('voting.voteValue', 'Vote')" width="200">
              <template #default="{ row }">{{ row.votes?.join(', ') || '-' }}</template>
            </el-table-column>
            <el-table-column :label="t('common.transaction')" min-width="180" show-overflow-tooltip>
              <template #default="{ row }">{{ truncateHash(row.transaction) }}</template>
            </el-table-column>
            <el-table-column :label="t('common.date')" width="160">
              <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
            </el-table-column>
          </el-table>
        </el-card>

        <!-- Empty state -->
        <el-card shadow="hover" v-else v-loading="detailLoading">
          <el-empty :description="t('voting.selectPollHint', 'Select a followed poll from the sidebar to view details')" />
        </el-card>
      </el-col>
    </el-row>

    <!-- Add Followed Poll Dialog -->
    <el-dialog
      v-model="showAddDialog"
      :title="t('voting.addFollowedPoll', 'Add Followed Poll')"
      width="480px"
      :close-on-click-modal="false"
      destroy-on-close
      class="nrcs-modal"
      @close="addForm.pollId = ''"
    >
      <el-form label-position="top">
        <el-form-item :label="t('voting.pollId', 'Poll ID')">
          <el-input v-model="addForm.pollId" :placeholder="t('voting.pollIdPlaceholder', 'Enter poll transaction ID')" clearable />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="addLoading" @click="addPoll">{{ t('common.confirm') }}</el-button>
      </template>
    </el-dialog>

    <CastVoteModal v-model:visible="showVote" :poll="castVotePoll" @success="loadSelectedPollDetails" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { BarChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp, truncateHash } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import CastVoteModal from '@/components/modals/CastVoteModal.vue'

use([CanvasRenderer, BarChart, GridComponent, TooltipComponent, LegendComponent])

const { t } = useI18n()
const accountStore = useAccountStore()

const STORAGE_KEY = 'nrcs_followed_polls'

const followedPolls = ref<Array<{ id: string; name: string }>>([])
const selectedPollId = ref<string>('')
const selectedPoll = ref<any>(null)
const detailLoading = ref(false)
const pollResults = ref<any[]>([])
const pollVotes = ref<any[]>([])
const showAddDialog = ref(false)
const addLoading = ref(false)
const addForm = reactive({ pollId: '' })
const showVote = ref(false)
const castVotePoll = ref<any>(null)

onMounted(() => loadFollowedPolls())

function loadFollowedPolls() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY) || '[]'
    followedPolls.value = JSON.parse(raw)
  } catch {
    followedPolls.value = []
  }
}

function saveFollowedPolls() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(followedPolls.value))
}

function isFollowing(pollId: string): boolean {
  return followedPolls.value.some((fp: any) => fp.id === pollId)
}

async function addPoll() {
  if (!addForm.pollId.trim()) {
    ElMessage.warning(t('voting.pollIdRequired', 'Poll ID is required'))
    return
  }
  addLoading.value = true
  try {
    const pollRes = await nrcsApi.getPoll(addForm.pollId.trim())
    const poll = pollRes as any
    const pollName = poll.name || addForm.pollId.trim()
    if (isFollowing(addForm.pollId.trim())) {
      ElMessage.info(t('voting.alreadyFollowing', 'Already following this poll'))
    } else {
      followedPolls.value.push({ id: addForm.pollId.trim(), name: pollName })
      saveFollowedPolls()
      ElMessage.success(t('voting.followSuccess', 'Poll added to followed list'))
    }
    showAddDialog.value = false
    addForm.pollId = ''
    // Select the newly added poll
    selectPoll({ id: addForm.pollId, name: pollName })
  } catch (e: any) {
    ElMessage.error(e?.message || t('voting.pollNotFound', 'Poll not found'))
  } finally {
    addLoading.value = false
  }
}

function unfollow(fp: { id: string }) {
  followedPolls.value = followedPolls.value.filter((p: any) => p.id !== fp.id)
  saveFollowedPolls()
  if (selectedPollId.value === fp.id) {
    selectedPollId.value = ''
    selectedPoll.value = null
    pollResults.value = []
    pollVotes.value = []
  }
  ElMessage.success(t('voting.unfollowSuccess', 'Poll removed from followed list'))
}

function toggleFollow(pollId: string) {
  if (isFollowing(pollId)) {
    unfollow({ id: pollId })
  } else {
    followedPolls.value.push({ id: pollId, name: selectedPoll.value?.name || pollId })
    saveFollowedPolls()
    ElMessage.success(t('voting.followSuccess', 'Poll added to followed list'))
  }
}

async function selectPoll(fp: { id: string; name: string }) {
  selectedPollId.value = fp.id
  selectedPoll.value = { id: fp.id, name: fp.name }
  await loadSelectedPollDetails()
}

async function loadSelectedPollDetails() {
  if (!selectedPollId.value) return
  detailLoading.value = true
  try {
    const [pollRes, resultRes, votesRes, statusRes] = await Promise.all([
      nrcsApi.getPoll(selectedPollId.value).catch(() => null),
      nrcsApi.getPollResult(selectedPollId.value).catch(() => null),
      nrcsApi.getPollVotes(selectedPollId.value, 0, 99).catch(() => null),
      nrcsApi.getBlockchainStatus().catch(() => null),
    ])
    const poll: any = pollRes || {}
    const results = (resultRes as any)?.results || []
    const votes = (votesRes as any)?.votes || []
    const currentHeight = (statusRes as any)?.numberOfBlocks || (statusRes as any)?.lastBlockHeight || 0

    selectedPoll.value = {
      ...selectedPoll.value,
      _description: poll.description || '',
      _accountRS: poll.accountRS || poll.account || '',
      _finishHeight: poll.finishHeight || 0,
      _finished: poll.finishHeight > 0 && poll.finishHeight <= currentHeight,
      _options: poll.options || [],
      _votingModel: poll.votingModel || 0,
    }
    pollResults.value = results
    pollVotes.value = votes

    // Update name in followed list
    if (poll.name) {
      const fp = followedPolls.value.find((p: any) => p.id === selectedPollId.value)
      if (fp) {
        fp.name = poll.name
        saveFollowedPolls()
      }
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    detailLoading.value = false
  }
}

const resultsChartOption = computed(() => {
  if (!pollResults.value || pollResults.value.length === 0) return {}
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
    xAxis: {
      type: 'category',
      data: pollResults.value.map((r: any) => r.option || ''),
      axisLabel: { color: '#909399' },
    },
    yAxis: {
      type: 'value',
      axisLabel: { color: '#909399' },
    },
    series: [{
      name: t('voting.weight', 'Weight'),
      type: 'bar',
      data: pollResults.value.map((r: any) => Number(r.weight) || 0),
      itemStyle: {
        color: {
          type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [{ offset: 0, color: '#409eff' }, { offset: 1, color: '#66b1ff' }],
        },
      },
    }],
  }
})

function refreshData() {
  if (selectedPollId.value) {
    loadSelectedPollDetails()
  }
}

function openVote(poll: any) {
  castVotePoll.value = {
    poll: poll.id,
    name: poll.name || poll.id,
    description: poll._description || '',
    options: poll._options || [],
    minRangeValue: poll._minRangeValue || 0,
    maxRangeValue: poll._maxRangeValue || 1,
  }
  showVote.value = true
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
}

.sidebar-card {
  min-height: 400px;
  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
}

.empty-sidebar {
  padding: 40px $space-md;
  text-align: center;
  color: $text-muted;
  font-size: $font-size-sm;
}

.followed-list {
  .followed-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: $space-sm $space-xs;
    border-radius: $radius-sm;
    cursor: pointer;
    transition: background $duration-fast;
    margin-bottom: 2px;

    &:hover {
      background: $bg-hover;
    }
    &.active {
      background: rgba(64, 158, 255, 0.1);
      border-left: 2px solid $primary;
    }

    .followed-item-content {
      flex: 1;
      min-width: 0;
      .poll-name {
        display: block;
        font-size: $font-size-sm;
        color: $text-primary;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }
      .poll-id-hint {
        display: block;
        font-size: $font-size-xs;
        color: $text-muted;
      }
    }
  }
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  .detail-title {
    font-size: $font-size-lg;
    font-weight: 600;
  }
  .detail-header-actions {
    display: flex;
    gap: $space-sm;
  }
}

.poll-info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: $space-md;
  margin-bottom: $space-lg;
  .info-item {
    .info-label {
      display: block;
      font-size: $font-size-xs;
      color: $text-muted;
      margin-bottom: 2px;
    }
    .info-value {
      display: block;
      font-size: $font-size-sm;
      color: $text-primary;
      word-break: break-all;
    }
  }
}

.section-title {
  font-size: $font-size-lg;
  font-weight: 600;
  margin-bottom: $space-md;
  color: $text-primary;
}
</style>
