/**
 * NRCS 前端 Store 统一出口。
 *
 * ✅ 推荐 Store：
 *    - useAccountStore：NRCS 风格账户（secretPhrase 派生 / RS 地址 / NQT 余额），基于 nrcsApi
 *    - useAppStore / useUiStore：通用应用与 UI 状态
 *    - useConstantsStore：动态常量加载（getConstants → VOTING_MODELS/HASH_ALGORITHMS/REQUEST_TYPES 等），
 *      提供 isApiEnabled/isRequestTypeEnabled/isRequireBlockchain 等运行时判定
 *
 * ⛔ @deprecated Ethereum 风格 Store（基于以太坊 RESTful API，与 NRCS 模型冲突，新代码禁止使用）：
 *    - useTransactionStore：将在阶段 3.12 基于 nrcsApi 重写
 *    - useNodeStore：将在阶段 1.1 基于 nrcsApi 重写（当前无视图引用）
 *    - useContractStore：NRCS 智能合约第二步实现时重写
 *
 * @see @/stores/modules/account.store.ts NRCS 风格账户 Store（推荐）
 */

// ✅ 推荐 Store
export { useAccountStore } from './account.store'
export { useAppStore } from './app.store'
export { useUiStore } from './ui.store'
export { useConstantsStore } from './constants.store'

// ⛔ @deprecated Ethereum 风格 Store（新代码禁止使用）
export { useTransactionStore } from './transaction.store'
export { useContractStore } from './contract.store'
export { useNodeStore } from './node.store'
