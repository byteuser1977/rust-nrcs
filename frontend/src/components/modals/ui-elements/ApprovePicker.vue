<template>
  <!--
    ApprovePicker —— Phasing 审批控制 UI 元素。
    对标 nrs.util.js:1438-1441 的 phasingParams 构造 + NRCS 表单中的 phasing 字段组。
  -->
  <div class="approve-picker">
    <el-checkbox v-model="enabled" :disabled="disabled" @change="onToggle">
      {{ t('uiElements.requirePhasingApproval') }}
    </el-checkbox>

    <div v-if="enabled" class="phasing-params">
      <!-- 投票模型 -->
      <el-form-item :label="t('uiElements.votingModel')" size="small">
        <el-select v-model="params.votingModel" :disabled="disabled" @change="emitChange">
          <el-option :label="t('uiElements.votingNone')" :value="-1" />
          <el-option :label="t('uiElements.votingAccount')" :value="0" />
          <el-option :label="t('uiElements.votingTransaction')" :value="1" />
          <el-option :label="t('uiElements.votingBalance')" :value="2" />
          <el-option :label="t('uiElements.votingAsset')" :value="3" />
          <el-option :label="t('uiElements.votingCurrency')" :value="4" />
        </el-select>
      </el-form-item>

      <!-- 法定人数 -->
      <el-form-item :label="t('uiElements.quorum')" size="small">
        <el-input-number
          v-model="params.quorum"
          :min="0"
          :disabled="disabled || params.votingModel === -1"
          controls-position="right"
          @change="emitChange"
        />
      </el-form-item>

      <!-- 最小余额（仅 balance/asset/currency 模型） -->
      <el-form-item
        v-if="params.votingModel >= 2"
        :label="t('uiElements.minBalance')"
        size="small"
      >
        <el-input-number
          v-model="params.minBalance"
          :min="0"
          :disabled="disabled"
          controls-position="right"
          @change="emitChange"
        />
      </el-form-item>

      <!-- 白名单账户 -->
      <el-form-item :label="t('uiElements.whitelistedAccounts')" size="small">
        <el-input
          v-model="whitelistText"
          type="textarea"
          :rows="2"
          :placeholder="t('uiElements.whitelistPlaceholder')"
          :disabled="disabled"
          @input="onWhitelistInput"
        />
      </el-form-item>

      <!-- 截止高度（复用 BlockHeightPicker） -->
      <el-form-item :label="t('uiElements.phasingFinishHeight')" size="small">
        <BlockHeightPicker
          v-model="params.finishHeight"
          :disabled="disabled"
          @change="emitChange"
        />
      </el-form-item>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * ApprovePicker 组件 —— Phasing（阶段审批）控制参数输入。
 *
 * 对标 nrs.util.js:1438-1441 的 phasingParams 构造 + NRCS 表单 phasing 字段组：
 *   - phasingVotingModel：投票模型（-1=无/0=账户/1=交易/2=余额/3=资产/4=货币）
 *   - phasingQuorum：法定人数
 *   - phasingMinBalance：最小余额（balance/asset/currency 模型）
 *   - phasingWhitelisted：白名单账户（逗号分隔 RS）
 *   - phasingFinishHeight：截止高度（复用 BlockHeightPicker）
 *
 * 用于 setPhasingOnly、sendMoney 等需要阶段审批的交易表单。
 */
import { reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BlockHeightPicker from './BlockHeightPicker.vue'

/** Phasing 参数结构（对标 nrs.util.js:1438-1441 phasingParams） */
export interface PhasingParams {
  votingModel: number
  quorum: number
  minBalance: number
  whitelisted: string[]
  finishHeight: string
}

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的 phasing 参数 */
    modelValue?: Partial<PhasingParams>
    /** 是否启用 */
    enabled?: boolean
    /** 是否禁用 */
    disabled?: boolean
  }>(),
  {
    modelValue: () => ({}),
    enabled: false,
    disabled: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: PhasingParams]
  'update:enabled': [value: boolean]
  change: [value: PhasingParams]
}>()

const { t } = useI18n()

/** 是否启用 phasing 审批 */
const enabled = ref<boolean>(props.enabled)

/** phasing 参数（对标 phasingParams） */
const params = reactive<PhasingParams>({
  votingModel: props.modelValue?.votingModel ?? -1,
  quorum: props.modelValue?.quorum ?? 1,
  minBalance: props.modelValue?.minBalance ?? 0,
  whitelisted: props.modelValue?.whitelisted ?? [],
  finishHeight: props.modelValue?.finishHeight ?? '',
})

/** 白名单文本（逗号分隔，用于 textarea 输入） */
const whitelistText = ref<string>((params.whitelisted || []).join(', '))

// 监听外部 modelValue 变化
watch(
  () => props.modelValue,
  (val) => {
    if (val) {
      if (val.votingModel !== undefined) params.votingModel = val.votingModel
      if (val.quorum !== undefined) params.quorum = val.quorum
      if (val.minBalance !== undefined) params.minBalance = val.minBalance
      if (val.whitelisted !== undefined) {
        params.whitelisted = val.whitelisted
        whitelistText.value = val.whitelisted.join(', ')
      }
      if (val.finishHeight !== undefined) params.finishHeight = val.finishHeight
    }
  },
  { deep: true },
)

watch(
  () => props.enabled,
  (val) => {
    if (val !== enabled.value) enabled.value = val
  },
)

/**
 * 启用切换。
 */
function onToggle(val: boolean): void {
  enabled.value = val
  emit('update:enabled', val)
  if (val) emitChange()
}

/**
 * 白名单输入（逗号分隔 → 数组）。
 */
function onWhitelistInput(val: string): void {
  params.whitelisted = val
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
  emitChange()
}

/**
 * 统一 emit 参数变化。
 */
function emitChange(): void {
  const snapshot: PhasingParams = {
    votingModel: params.votingModel,
    quorum: params.quorum,
    minBalance: params.minBalance,
    whitelisted: [...params.whitelisted],
    finishHeight: params.finishHeight,
  }
  emit('update:modelValue', snapshot)
  emit('change', snapshot)
}
</script>

<style scoped lang="scss">
.approve-picker {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.phasing-params {
  padding: 12px;
  border: 1px solid var(--el-border-color-light);
  border-radius: 4px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
</style>
