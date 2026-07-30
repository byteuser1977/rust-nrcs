/**
 * NRCS 前端 API 模块统一出口。
 *
 * ✅ 推荐入口：`nrcsApi`（基于 `@/api/nrcs-client`，NRCS requestType 风格），
 *    覆盖账户、交易、区块、节点、别名、资产、货币、投票、混币、标记数据、消息等全部 NRCS 业务端点。
 *
 * ⛔ 下列 `*Api` 对象均为 Ethereum RESTful 风格占位实现（@deprecated），
 *    与 NRCS 模型冲突，仅为兼容尚未重写的以太坊占位视图保留，新代码禁止使用：
 *    - accountApi / transactionApi / nodeApi / contractApi
 *
 * @see @/api/nrcs-client.ts NRCS HTTP 客户端
 * @see /Volumes/DATA/data/develop/git/nrcs/nrcs-main/html/www/ui/js/nrs.server.js 参考实现
 */

// ✅ 推荐：NRCS 业务 API（唯一应在业务代码中使用的 API 入口）
export { nrcsApi } from './nrcs.api'

// ⛔ @deprecated Ethereum RESTful 占位 API（新代码禁止使用）
export { accountApi } from './account.api'
export { transactionApi } from './transaction.api'
export { contractApi } from './contract.api'
export { nodeApi } from './node.api'

// ✅ NRCS 业务类型定义
export type {
  NrcsAccount, NrcsBalance, NrcsTransaction, NrcsUnconfirmedTransaction,
  NrcsBlock, NrcsPeer, NrcsBlockchainStatus, NrcsAlias, NrcsAsset,
  NrcsCurrency, NrcsPoll, NrcsDGSProduct, NrcsDGSPurchase, NrcsMessage,
  NrcsGenerator
} from './nrcs.api'
