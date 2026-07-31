<template>
  <a
    v-if="link"
    href="#"
    class="transaction-link"
    :class="`transaction-link--${link.type}`"
    @click.prevent="handleClick"
  >
    {{ link.text || link.value }}
  </a>
</template>

<script setup lang="ts">
/**
 * TransactionLink 组件 —— 渲染交易详情中的可点击链接。
 *
 * 对标参考中 `<a href='#' data-transaction='...'>` / `<a href='#' data-user='...'>` 等
 * jQuery 拼接的链接元素。Vue3 中改为结构化 LinkRef，由本组件统一渲染。
 *
 * 点击行为：
 *   - transaction → 触发 'navigate' 事件，父组件打开交易详情 modal
 *   - account → 触发 'navigate' 事件，父组件打开账户详情 modal
 *   - block → 触发 'navigate' 事件，父组件打开区块详情 modal
 *   - asset / currency / poll / alias → 路由跳转
 *   - tagged-data / shuffling / goods → 触发 'navigate' 事件
 *
 * 父组件监听 'navigate' 事件并实现具体导航逻辑（对标 NRS.modalStack）。
 */
import type { LinkRef } from '@/utils/transaction-links'

const props = defineProps<{ link: LinkRef }>()
const emit = defineEmits<{
  navigate: [link: LinkRef]
}>()

/**
 * 处理点击事件：触发 navigate 事件，由父组件决定具体导航行为。
 */
function handleClick(): void {
  emit('navigate', props.link)
}
</script>

<style scoped lang="scss">
.transaction-link {
  color: var(--el-color-primary);
  text-decoration: none;
  cursor: pointer;
  word-break: break-all;
  &:hover {
    text-decoration: underline;
  }
}
</style>
