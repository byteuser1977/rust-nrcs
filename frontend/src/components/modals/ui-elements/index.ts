/******************************************************************************
 * UI Elements 框架 —— 可复用表单子组件出口。
 *
 * 对标 nrs.modals.uielements.js（176 行）的可复用 UI 元素抽象。
 * 这些组件在多个交易表单 modal 中共享，避免重复实现。
 *
 * 组件清单：
 *   - BlockHeightPicker：区块高度选择器（当前高度+使用当前+/-按钮+时间估算）
 *   - HashAlgorithmSelect：哈希算法下拉（复用 constants.getAlgorithmOptions）
 *   - AssetInfoInput：资产 ID 输入 + 自动查询名称/精度
 *   - CurrencyInfoInput：货币代码输入 + 自动查询 ID/精度
 *   - DeadlinePicker：交易截止时间（分钟）输入
 *   - BroadcastToggle：不广播/不签名开关（离线签名模式）
 *   - NoteToSelfInput：给自己留言加密消息输入
 *   - ApprovePicker：Phasing 审批控制参数输入
 *   - RtHashInput：引用交易哈希输入（phasing by transaction）
 ******************************************************************************/
export { default as BlockHeightPicker } from './BlockHeightPicker.vue'
export { default as HashAlgorithmSelect } from './HashAlgorithmSelect.vue'
export { default as AssetInfoInput } from './AssetInfoInput.vue'
export { default as CurrencyInfoInput } from './CurrencyInfoInput.vue'
export { default as DeadlinePicker } from './DeadlinePicker.vue'
export { default as BroadcastToggle } from './BroadcastToggle.vue'
export { default as NoteToSelfInput } from './NoteToSelfInput.vue'
export { default as ApprovePicker } from './ApprovePicker.vue'
export { default as RtHashInput } from './RtHashInput.vue'

export type { PhasingParams } from './ApprovePicker.vue'
