<template>
  <div class="page-container debug-console-page">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Monitor /></el-icon>
        {{ t('settings.debugConsole') }}
      </h2>
      <div class="header-actions">
        <el-select
          v-model="level"
          size="small"
          style="width: 130px"
          @change="onLevelChange"
        >
          <el-option :label="t('debug.levelBasic')" :value="1" />
          <el-option :label="t('debug.levelVerbose')" :value="5" />
          <el-option :label="t('debug.levelAll')" :value="10" />
        </el-select>
        <el-input
          v-model="filter"
          size="small"
          clearable
          :placeholder="t('debug.filterPlaceholder')"
          style="width: 220px"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-button size="small" :disabled="entries.length === 0" @click="clearLogs">
          <el-icon><Delete /></el-icon>
          {{ t('debug.clear') }}
        </el-button>
        <el-button size="small" @click="refresh">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" class="console-card" v-loading="false">
      <template #header>
        <div class="console-card-header">
          <div class="header-left">
            <el-tag :type="entries.length > 0 ? 'success' : 'info'" size="small">
              {{ entries.length }} {{ t('debug.entries') }}
            </el-tag>
            <el-tag v-if="errorCount > 0" type="danger" size="small">
              {{ errorCount }} {{ t('debug.errors') }}
            </el-tag>
            <span class="auto-scroll-hint">
              <el-icon><Bottom /></el-icon>
              {{ t('debug.autoScroll') }}
            </span>
          </div>
          <div class="header-right">
            <el-switch
              v-model="autoScroll"
              :active-text="t('debug.autoScroll')"
              inline-prompt
            />
          </div>
        </div>
      </template>

      <div ref="logContainerRef" class="log-container">
        <div v-if="filteredEntries.length === 0" class="empty-state">
          <el-icon class="empty-icon"><DocumentRemove /></el-icon>
          <p>{{ t('debug.noLogs') }}</p>
          <p class="empty-hint">{{ t('debug.noLogsHint') }}</p>
        </div>

        <div
          v-for="entry in filteredEntries"
          :key="entry.id"
          class="log-entry"
          :class="{ 'log-entry--error': entry.isError }"
        >
          <div class="log-entry-header" @click="toggleExpand(entry.id)">
            <span class="log-time">{{ formatTime(entry.timestamp) }}</span>
            <el-tag
              :type="entry.method === 'GET' ? 'success' : 'warning'"
              size="small"
              effect="plain"
              class="log-method"
            >
              {{ entry.method }}
            </el-tag>
            <span class="log-url" :title="entry.url">{{ entry.url }}</span>
            <el-icon v-if="entry.isError" class="log-error-icon"><CircleCloseFilled /></el-icon>
            <el-icon class="log-expand-icon">
              <component :is="expandedIds.has(entry.id) ? ArrowUp : ArrowDown" />
            </el-icon>
          </div>

          <div v-if="expandedIds.has(entry.id)" class="log-entry-body">
            <div v-if="entry.data" class="log-section">
              <div class="log-section-title">{{ t('debug.request') }}</div>
              <pre class="log-pre log-pre--request">{{ formatJson(entry.data) }}</pre>
            </div>
            <div v-if="entry.response !== undefined" class="log-section">
              <div class="log-section-title" :class="{ 'text-danger': entry.isError }">
                {{ t('debug.response') }}
              </div>
              <pre
                class="log-pre"
                :class="{ 'log-pre--error': entry.isError }"
              >{{ formatJson(entry.response) }}</pre>
            </div>
          </div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Monitor, Search, Delete, Refresh, Bottom,
  DocumentRemove, CircleCloseFilled, ArrowDown, ArrowUp,
} from '@element-plus/icons-vue'
import { logger, type ConsoleEntry } from '@/utils/logger'

const { t } = useI18n()

/** 日志条目列表（响应式快照） */
const entries = ref<ConsoleEntry[]>([])

/** 展开的条目 ID 集合 */
const expandedIds = ref<Set<number>>(new Set())

/** 日志级别（1=基本/5=详细/10=全部） */
const level = ref(logger.getLevel())

/** 过滤关键词 */
const filter = ref('')

/** 是否自动滚动到底部 */
const autoScroll = ref(true)

/** 日志容器引用 */
const logContainerRef = ref<HTMLDivElement | null>(null)

/** logger 订阅取消函数 */
let unsubscribe: (() => void) | null = null

/** 错误条目数量 */
const errorCount = computed(() => entries.value.filter((e) => e.isError).length)

/** 过滤后的条目（按 URL/方法匹配关键词） */
const filteredEntries = computed(() => {
  const kw = filter.value.trim().toLowerCase()
  if (!kw) return entries.value
  return entries.value.filter(
    (e) =>
      e.url.toLowerCase().includes(kw) ||
      e.method.toLowerCase().includes(kw)
  )
})

/**
 * 从 logger 拉取最新条目并刷新视图。
 */
function refresh(): void {
  entries.value = [...logger.getEntries()]
  if (autoScroll.value) {
    nextTick(scrollToBottom)
  }
}

/**
 * 切换条目展开/折叠。
 */
function toggleExpand(id: number): void {
  const set = new Set(expandedIds.value)
  if (set.has(id)) {
    set.delete(id)
  } else {
    set.add(id)
  }
  expandedIds.value = set
}

/**
 * 清空所有日志。
 */
function clearLogs(): void {
  logger.clear()
  expandedIds.value = new Set()
  refresh()
}

/**
 * 日志级别变化回调。
 */
function onLevelChange(val: number): void {
  logger.setLevel(val)
}

/**
 * 滚动到底部。
 */
function scrollToBottom(): void {
  const container = logContainerRef.value
  if (container) {
    container.scrollTop = container.scrollHeight
  }
}

/**
 * 格式化时间戳为 HH:MM:SS.mmm。
 */
function formatTime(date: Date): string {
  const h = String(date.getHours()).padStart(2, '0')
  const m = String(date.getMinutes()).padStart(2, '0')
  const s = String(date.getSeconds()).padStart(2, '0')
  const ms = String(date.getMilliseconds()).padStart(3, '0')
  return `${h}:${m}:${s}.${ms}`
}

/**
 * 格式化 JSON 对象为缩进字符串。
 */
function formatJson(data: any): string {
  if (data === null || data === undefined) return ''
  if (typeof data === 'string') return data
  try {
    return JSON.stringify(data, null, 2)
  } catch {
    return String(data)
  }
}

/**
 * 监听过滤变化时重置滚动。
 */
watch(filter, () => {
  if (autoScroll.value) {
    nextTick(scrollToBottom)
  }
})

onMounted(() => {
  // 订阅 logger 变化
  unsubscribe = logger.subscribe(() => {
    refresh()
  })
  refresh()
})

onUnmounted(() => {
  if (unsubscribe) {
    unsubscribe()
    unsubscribe = null
  }
})
</script>

<style scoped lang="scss">
.debug-console-page {
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .console-card {
    :deep(.el-card__body) {
      padding: 0;
    }
  }

  .console-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-left {
      display: flex;
      align-items: center;
      gap: 8px;

      .auto-scroll-hint {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        color: $text-muted;
        font-size: 12px;
        margin-left: 8px;
      }
    }
  }

  .log-container {
    max-height: calc(100vh - 280px);
    min-height: 400px;
    overflow-y: auto;
    background: #0b1121;
    font-family: 'Roboto Mono', 'Courier New', monospace;
    font-size: 12.5px;
    line-height: 1.6;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    color: #64748b;

    .empty-icon {
      font-size: 48px;
      margin-bottom: 12px;
      color: #334155;
    }

    p {
      margin: 4px 0;
    }

    .empty-hint {
      font-size: 11px;
      color: #475569;
    }
  }

  .log-entry {
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    transition: background $duration-fast ease;

    &:hover {
      background: rgba(255, 255, 255, 0.03);
    }

    &--error {
      background: rgba(239, 68, 68, 0.08);

      &:hover {
        background: rgba(239, 68, 68, 0.12);
      }
    }
  }

  .log-entry-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    cursor: pointer;
    user-select: none;

    .log-time {
      color: #64748b;
      font-size: 11px;
      flex-shrink: 0;
    }

    .log-method {
      flex-shrink: 0;
      font-weight: 600;
    }

    .log-url {
      color: #29fd2f;
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      direction: rtl;
      text-align: left;
    }

    .log-error-icon {
      color: #ef4444;
      flex-shrink: 0;
    }

    .log-expand-icon {
      color: #64748b;
      flex-shrink: 0;
      font-size: 12px;
    }
  }

  .log-entry-body {
    padding: 0 12px 12px 12px;
    background: rgba(0, 0, 0, 0.3);
  }

  .log-section {
    margin-top: 8px;

    .log-section-title {
      color: #94a3b8;
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      margin-bottom: 4px;

      &.text-danger {
        color: #ef4444;
      }
    }
  }

  .log-pre {
    margin: 0;
    padding: 8px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 4px;
    color: #e2e8f0;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 320px;
    overflow-y: auto;

    &--request {
      color: #cbd5e1;
    }

    &--error {
      color: #fca5a5;
      background: rgba(239, 68, 68, 0.08);
    }
  }
}
</style>
