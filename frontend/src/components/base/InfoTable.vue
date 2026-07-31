<template>
  <div v-if="rows.length > 0" class="info-table">
    <el-descriptions :column="column" border size="small">
      <el-descriptions-item
        v-for="(row, idx) in rows"
        :key="idx"
        :label="row.label"
        :span="row.span ? column : 1"
      >
        <InfoValue :value="row.value" @navigate="onNavigate" />
      </el-descriptions-item>
    </el-descriptions>
  </div>
</template>

<script setup lang="ts">
/**
 * InfoTable 组件 —— 渲染交易详情信息表。
 *
 * 对标参考中 NRS.createInfoTable 生成的 `<table><tbody><tr><td>label</td><td>value</td></tr></tbody></table>`。
 * Vue3 中改用 el-descriptions 组件，更符合 Element Plus 风格。
 *
 * 接收 InfoRow[] 数组，逐行渲染 label + value（支持多种值类型）。
 */
import type { InfoRow } from '@/utils/transaction-info-table'
import type { LinkRef } from '@/utils/transaction-links'
import InfoValue from './InfoValue.vue'

withDefaults(
  defineProps<{
    rows: InfoRow[]
    column?: number
  }>(),
  {
    column: 2,
  },
)

const emit = defineEmits<{
  navigate: [link: LinkRef]
}>()

/**
 * 转发 InfoValue 的 navigate 事件。
 */
function onNavigate(link: LinkRef): void {
  emit('navigate', link)
}
</script>

<style scoped lang="scss">
.info-table {
  margin-bottom: 12px;
}
</style>
