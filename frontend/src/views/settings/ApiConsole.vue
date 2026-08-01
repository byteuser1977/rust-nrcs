<template>
  <div class="page-container api-console-page">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Cpu /></el-icon>
        {{ t('settings.apiConsole') }}
      </h2>
      <div class="header-actions">
        <el-button size="small" @click="loadRequestTypes" :loading="loadingTypes">
          <el-icon><Refresh /></el-icon>
          {{ t('apiConsole.reloadTypes') }}
        </el-button>
      </div>
    </div>

    <el-row :gutter="16">
      <!-- 左侧：请求构造 -->
      <el-col :span="11">
        <el-card shadow="hover" class="request-card">
          <template #header>
            <div class="card-header">
              <span>{{ t('apiConsole.requestBuilder') }}</span>
              <el-tag size="small" type="info">{{ requestTypeList.length }} {{ t('apiConsole.typesAvailable') }}</el-tag>
            </div>
          </template>

          <el-form label-position="top" class="request-form">
            <el-form-item :label="t('apiConsole.requestType')">
              <el-select
                v-model="selectedRequestType"
                filterable
                clearable
                :placeholder="t('apiConsole.selectRequestType')"
                style="width: 100%"
                @change="onRequestTypeChange"
              >
                <el-option
                  v-for="rt in requestTypeList"
                  :key="rt"
                  :label="rt"
                  :value="rt"
                />
              </el-select>
            </el-form-item>

            <el-form-item :label="t('apiConsole.parameters')">
              <div class="params-editor">
                <div
                  v-for="(param, idx) in params"
                  :key="idx"
                  class="param-row"
                >
                  <el-input
                    v-model="param.key"
                    :placeholder="t('apiConsole.paramName')"
                    size="small"
                    class="param-key"
                  />
                  <el-input
                    v-model="param.value"
                    :placeholder="t('apiConsole.paramValue')"
                    size="small"
                    class="param-value"
                  />
                  <el-button
                    size="small"
                    type="danger"
                    text
                    @click="removeParam(idx)"
                  >
                    <el-icon><Delete /></el-icon>
                  </el-button>
                </div>
                <el-button size="small" type="primary" text @click="addParam">
                  <el-icon><Plus /></el-icon>
                  {{ t('apiConsole.addParam') }}
                </el-button>
              </div>
            </el-form-item>

            <el-form-item>
              <el-button
                type="primary"
                :loading="sending"
                :disabled="!selectedRequestType"
                @click="sendRequest"
              >
                <el-icon><Promotion /></el-icon>
                {{ t('apiConsole.sendRequest') }}
              </el-button>
              <el-button @click="resetForm" :disabled="sending">
                {{ t('common.reset') }}
              </el-button>
            </el-form-item>

            <!-- 当前请求的元信息 -->
            <div v-if="selectedRequestType && currentConfig" class="request-meta">
              <el-descriptions :column="1" size="small" border>
                <el-descriptions-item :label="t('apiConsole.requireBlockchain')">
                  <el-tag :type="currentConfig.requireBlockchain ? 'warning' : 'info'" size="small">
                    {{ currentConfig.requireBlockchain ? t('common.yes') : t('common.no') }}
                  </el-tag>
                </el-descriptions-item>
                <el-descriptions-item :label="t('apiConsole.requireFullClient')">
                  <el-tag :type="currentConfig.requireFullClient ? 'warning' : 'info'" size="small">
                    {{ currentConfig.requireFullClient ? t('common.yes') : t('common.no') }}
                  </el-tag>
                </el-descriptions-item>
                <el-descriptions-item :label="t('apiConsole.requirePassword')">
                  <el-tag :type="currentConfig.requirePassword ? 'danger' : 'info'" size="small">
                    {{ currentConfig.requirePassword ? t('common.yes') : t('common.no') }}
                  </el-tag>
                </el-descriptions-item>
                <el-descriptions-item :label="t('apiConsole.httpMethod')">
                  <el-tag :type="isPostRequired ? 'warning' : 'success'" size="small">
                    {{ isPostRequired ? 'POST' : 'GET' }}
                  </el-tag>
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </el-form>
        </el-card>
      </el-col>

      <!-- 右侧：响应展示 -->
      <el-col :span="13">
        <el-card shadow="hover" class="response-card">
          <template #header>
            <div class="card-header">
              <span>{{ t('apiConsole.response') }}</span>
              <div class="response-actions" v-if="response">
                <el-tag :type="isError ? 'danger' : 'success'" size="small">
                  {{ isError ? t('common.error') : t('common.success') }}
                </el-tag>
                <el-tag size="small" type="info">{{ responseTime }}ms</el-tag>
                <el-button size="small" text @click="copyResponse">
                  <el-icon><CopyDocument /></el-icon>
                </el-button>
              </div>
            </div>
          </template>

          <div v-if="!response" class="empty-response">
            <el-icon class="empty-icon"><Document /></el-icon>
            <p>{{ t('apiConsole.noResponse') }}</p>
          </div>

          <pre v-else class="response-pre" :class="{ 'response-pre--error': isError }">{{ formattedResponse }}</pre>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import {
  Cpu, Refresh, Delete, Plus, Promotion,
  CopyDocument, Document,
} from '@element-plus/icons-vue'
import { useConstantsStore } from '@/stores/modules/constants.store'
import { nrcsGet, nrcsPost, isRequirePost } from '@/api/nrcs-client'
import type { RequestTypeConfig } from '@/constants/server-constants'

const { t } = useI18n()
const constantsStore = useConstantsStore()

/** 参数行 */
interface ParamRow {
  key: string
  value: string
}

/** 选中的 requestType */
const selectedRequestType = ref('')

/** 参数列表 */
const params = ref<ParamRow[]>([{ key: '', value: '' }])

/** 响应数据 */
const response = ref<any>(null)

/** 是否为错误响应 */
const isError = ref(false)

/** 响应耗时（毫秒） */
const responseTime = ref(0)

/** 是否正在发送请求 */
const sending = ref(false)

/** 是否正在加载 requestTypes */
const loadingTypes = ref(false)

/** requestTypes 列表（按字母排序） */
const requestTypeList = computed(() => {
  const types = Object.keys(constantsStore.requestTypes)
  if (types.length === 0) return []
  return types.sort()
})

/** 当前 requestType 的服务端配置 */
const currentConfig = computed<RequestTypeConfig | null>(() => {
  if (!selectedRequestType.value) return null
  return constantsStore.requestTypes[selectedRequestType.value] || null
})

/** 是否需要 POST */
const isPostRequired = computed(() => {
  if (!selectedRequestType.value) return false
  return isRequirePost(selectedRequestType.value, buildParamsObject())
})

/** 格式化的响应 JSON */
const formattedResponse = computed(() => {
  if (response.value === null || response.value === undefined) return ''
  if (typeof response.value === 'string') return response.value
  try {
    return JSON.stringify(response.value, null, 2)
  } catch {
    return String(response.value)
  }
})

/**
 * 加载服务端 requestTypes 列表。
 */
async function loadRequestTypes(): Promise<void> {
  loadingTypes.value = true
  try {
    await constantsStore.loadServerConstants()
    if (requestTypeList.value.length === 0) {
      ElMessage.warning(t('apiConsole.noTypesLoaded'))
    } else {
      ElMessage.success(t('apiConsole.typesLoaded'))
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loadingTypes.value = false
  }
}

/**
 * requestType 变化回调。
 */
function onRequestTypeChange(): void {
  response.value = null
  isError.value = false
}

/**
 * 添加参数行。
 */
function addParam(): void {
  params.value.push({ key: '', value: '' })
}

/**
 * 移除参数行。
 */
function removeParam(idx: number): void {
  params.value.splice(idx, 1)
  if (params.value.length === 0) {
    params.value.push({ key: '', value: '' })
  }
}

/**
 * 构建参数对象（过滤空键）。
 */
function buildParamsObject(): Record<string, any> {
  const obj: Record<string, any> = {}
  for (const p of params.value) {
    if (p.key.trim()) {
      obj[p.key.trim()] = p.value
    }
  }
  return obj
}

/**
 * 发送 API 请求。
 */
async function sendRequest(): Promise<void> {
  if (!selectedRequestType.value) {
    ElMessage.warning(t('apiConsole.selectRequestType'))
    return
  }

  sending.value = true
  response.value = null
  isError.value = false
  const startTime = performance.now()

  try {
    const reqParams = buildParamsObject()
    const usePost = isRequirePost(selectedRequestType.value, reqParams)
    const result = usePost
      ? await nrcsPost<any>(selectedRequestType.value, reqParams)
      : await nrcsGet<any>(selectedRequestType.value, reqParams)

    response.value = result
    responseTime.value = Math.round(performance.now() - startTime)
  } catch (e: any) {
    response.value = e.response?.data || {
      errorCode: e.code || -1,
      errorDescription: e.message || String(e),
    }
    isError.value = true
    responseTime.value = Math.round(performance.now() - startTime)
  } finally {
    sending.value = false
  }
}

/**
 * 重置表单。
 */
function resetForm(): void {
  selectedRequestType.value = ''
  params.value = [{ key: '', value: '' }]
  response.value = null
  isError.value = false
  responseTime.value = 0
}

/**
 * 复制响应到剪贴板。
 */
async function copyResponse(): Promise<void> {
  try {
    await navigator.clipboard.writeText(formattedResponse.value)
    ElMessage.success(t('common.copied'))
  } catch {
    ElMessage.error(t('common.copyFailed'))
  }
}

onMounted(() => {
  // 若常量未加载则触发加载
  if (requestTypeList.value.length === 0) {
    loadRequestTypes()
  }
})
</script>

<style scoped lang="scss">
.api-console-page {
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .request-form {
    .params-editor {
      width: 100%;

      .param-row {
        display: flex;
        gap: 6px;
        margin-bottom: 6px;

        .param-key {
          flex: 0 0 40%;
        }

        .param-value {
          flex: 1;
        }
      }
    }

    .request-meta {
      margin-top: 12px;
      padding-top: 12px;
      border-top: 1px dashed $border-default;
    }
  }

  .response-card {
    .response-actions {
      display: flex;
      align-items: center;
      gap: 6px;
    }

    .empty-response {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      min-height: 320px;
      color: $text-muted;

      .empty-icon {
        font-size: 48px;
        margin-bottom: 12px;
        color: $text-muted;
      }
    }

    .response-pre {
      margin: 0;
      padding: 12px;
      background: #0b1121;
      color: #e2e8f0;
      border-radius: $radius-sm;
      font-family: 'Roboto Mono', 'Courier New', monospace;
      font-size: 12.5px;
      line-height: 1.6;
      max-height: calc(100vh - 280px);
      min-height: 320px;
      overflow: auto;
      white-space: pre-wrap;
      word-break: break-all;

      &--error {
        color: #fca5a5;
      }
    }
  }
}
</style>
