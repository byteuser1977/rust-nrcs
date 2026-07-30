/**
 * NRCS 服务端常量与静态常量定义。
 *
 * 端口自参考实现：
 * - `nrs.constants.js`：`NRS.constants` 顶部的静态常量（DB_VERSION / MAX_*_JAVA / PL_* / PV_* / PNC_* 等）
 * - `js/data/constants.js`：`getConstants` 响应的完整结构（运行时由 `constants.store.ts` 通过 `nrcsApi.getConstants()` 注入）
 *
 * 设计说明：
 * - 静态常量（本文件导出的 `STATIC_CONSTANTS`）在编译期固定，不依赖网络
 * - 运行时常量（VOTING_MODELS / HASH_ALGORITHMS / REQUEST_TYPES 等）初始为空对象，
 *   由 `useConstantsStore.loadServerConstants()` 在应用启动时从节点 `getConstants` 拉取并填充
 * - 所有 TypeScript 接口严格对标 `js/data/constants.js` 的 JSON 结构，字段名保持一致
 */

// ============================================================================
// 静态常量（端口自 nrs.constants.js 顶部 NRS.constants）
// ============================================================================

/** 客户端数据库版本（IndexedDB schema 升级用） */
export const DB_VERSION = 2

/** 插件清单版本 */
export const PLUGIN_VERSION = 1

/** Java short 上限（有符号） */
export const MAX_SHORT_JAVA = 32767

/** Java unsigned short 上限（leaseBalance 周期上限校验用） */
export const MAX_UNSIGNED_SHORT_JAVA = 65535

/** Java int 上限 */
export const MAX_INT_JAVA = 2147483647

/** 可修剪消息最小长度 */
export const MIN_PRUNABLE_MESSAGE_LENGTH = 28

/** API 被禁用时返回的错误码 */
export const DISABLED_API_ERROR_CODE = 16

/** 最大 phasing 持续时间（区块数） */
export const MAX_PHASING_DURATION = 20160

/** 最大区块 payload 长度 */
export const MAX_BLOCK_PAYLOAD_LENGTH = 44880

/** 最大任意消息长度 */
export const MAX_ARBITRARY_MESSAGE_LENGTH = 160

// --- 插件启动状态码 ---

/** 插件运行中 */
export const PL_RUNNING = 1
/** 插件已暂停 */
export const PL_PAUSED = 2
/** 插件已停用 */
export const PL_DEACTIVATED = 3
/** 插件已中止 */
export const PL_HALTED = 4

// --- 插件合法性状态码 ---

/** 合法 */
export const PV_VALID = 100
/** 不合法 */
export const PV_NOT_VALID = 300
/** 未知清单版本 */
export const PV_UNKNOWN_MANIFEST_VERSION = 301
/** 不兼容的清单版本 */
export const PV_INCOMPATIBLE_MANIFEST_VERSION = 302
/** 清单文件非法 */
export const PV_INVALID_MANIFEST_FILE = 303
/** 文件缺失或非法 */
export const PV_INVALID_MISSING_FILES = 304
/** JavaScript 文件非法 */
export const PV_INVALID_JAVASCRIPT_FILE = 305

// --- 插件 NRS 兼容性状态码 ---

/** 兼容 */
export const PNC_COMPATIBLE = 100
/** 次要版本差异（兼容） */
export const PNC_COMPATIBILITY_MINOR_RELEASE_DIFF = 101
/** 兼容性警告 */
export const PNC_COMPATIBILITY_WARNING = 200
/** 主要版本差异（兼容） */
export const PNC_COMPATIBILITY_MAJOR_RELEASE_DIFF = 202
/** 不兼容 */
export const PNC_NOT_COMPATIBLE = 300
/** 兼容性未知 */
export const PNC_COMPATIBILITY_UNKNOWN = 301
/** 客户端版本过旧 */
export const PNC_COMPATIBILITY_CLIENT_VERSION_TOO_OLD = 302

// --- Forging 状态标识 ---

/** 正在锻造 */
export const FORGING = 'forging'
/** 未锻造 */
export const NOT_FORGING = 'not_forging'
/** 未知状态 */
export const UNKNOWN = 'unknown'

// --- 其他静态标识 ---

/** Ignis 货币代码 */
export const IGNIS_CURRENCY_CODE = 'JLRDA'

/** 定时交易请求前缀（scheduleCurrencyBuy 等） */
export const SCHEDULE_PREFIX = 'schedule'

/** 主网最后已知区块（getECBlock 在节点不可达时的回退） */
export const LAST_KNOWN_BLOCK: { id: string; height: string } = {
  id: '3488276486778630462',
  height: '0'
}

/** 测试网最后已知区块 */
export const LAST_KNOWN_TESTNET_BLOCK: { id: string; height: string } = {
  id: '3488276486778630462',
  height: '0'
}

/** 聚合静态常量，便于按需导入 */
export const STATIC_CONSTANTS = {
  DB_VERSION,
  PLUGIN_VERSION,
  MAX_SHORT_JAVA,
  MAX_UNSIGNED_SHORT_JAVA,
  MAX_INT_JAVA,
  MIN_PRUNABLE_MESSAGE_LENGTH,
  DISABLED_API_ERROR_CODE,
  MAX_PHASING_DURATION,
  MAX_BLOCK_PAYLOAD_LENGTH,
  MAX_ARBITRARY_MESSAGE_LENGTH,
  PL_RUNNING,
  PL_PAUSED,
  PL_DEACTIVATED,
  PL_HALTED,
  PV_VALID,
  PV_NOT_VALID,
  PV_UNKNOWN_MANIFEST_VERSION,
  PV_INCOMPATIBLE_MANIFEST_VERSION,
  PV_INVALID_MANIFEST_FILE,
  PV_INVALID_MISSING_FILES,
  PV_INVALID_JAVASCRIPT_FILE,
  PNC_COMPATIBLE,
  PNC_COMPATIBILITY_MINOR_RELEASE_DIFF,
  PNC_COMPATIBILITY_WARNING,
  PNC_COMPATIBILITY_MAJOR_RELEASE_DIFF,
  PNC_NOT_COMPATIBLE,
  PNC_COMPATIBILITY_UNKNOWN,
  PNC_COMPATIBILITY_CLIENT_VERSION_TOO_OLD,
  FORGING,
  NOT_FORGING,
  UNKNOWN,
  IGNIS_CURRENCY_CODE,
  SCHEDULE_PREFIX,
  LAST_KNOWN_BLOCK,
  LAST_KNOWN_TESTNET_BLOCK
} as const

// ============================================================================
// getConstants 响应类型接口（对标 js/data/constants.js 结构）
// ============================================================================

/**
 * 交易子类型的服务端常量描述。
 * 对应 `js/data/constants.js` 中 `transactionSubTypes[name]` 与 `transactionTypes[type].subtypes[subtype]`。
 */
export interface TransactionSubTypeServerInfo {
  /** 是否可被 phased */
  isPhasable: boolean
  /** 子类型编号 */
  subtype: number
  /** 是否必须有收款人 */
  mustHaveRecipient: boolean
  /** 子类型名称（如 "OrdinaryPayment"） */
  name: string
  /** 是否允许有收款人 */
  canHaveRecipient: boolean
  /** 父类型编号 */
  type: number
  /** 是否对 phasing 安全 */
  isPhasingSafe: boolean
}

/**
 * 交易类型的服务端描述（含其子类型表）。
 * 对应 `transactionTypes[type]`。
 */
export interface TransactionTypeServerInfo {
  /** 子类型映射：subtype 编号 → 服务端描述 */
  subtypes: Record<string, TransactionSubTypeServerInfo>
}

/**
 * 单个 requestType 的服务端配置。
 * 对应 `requestTypes[requestType]`。
 */
export interface RequestTypeConfig {
  /** 是否允许必需的区块参数 */
  allowRequiredBlockParameters: boolean
  /** 是否需要全节点（非轻节点） */
  requireFullClient: boolean
  /** 是否需要管理员密码 */
  requirePassword: boolean
  /** 是否需要区块链已就绪 */
  requireBlockchain: boolean
  /** 文件上传参数名（仅文件上传类接口存在） */
  fileParameter?: string
  /** 是否必须 POST */
  requirePost: boolean
  /** 是否启用 */
  enabled: boolean
}

/**
 * 单个 API 标签的服务端配置。
 * 对应 `apiTags[tag]`。
 */
export interface ApiTagConfig {
  /** 标签显示名 */
  name: string
  /** 是否启用 */
  enabled: boolean
}

/**
 * `getConstants` 接口的完整响应结构。
 * 严格对标 `js/data/constants.js` 中 `NRS.constants.SERVER` 的字段。
 */
export interface ServerConstantsResponse {
  /** 交易子类型映射（按名称索引） */
  transactionSubTypes: Record<string, TransactionSubTypeServerInfo>
  /** 创世账户 ID（数字字符串） */
  genesisAccountId: string
  /** 交易类型映射（按 type 编号索引，含子类型表） */
  transactionTypes: Record<string, TransactionTypeServerInfo>
  /** 投票模型：{ NQT, CURRENCY, ACCOUNT, ASSET, TRANSACTION, NONE, HASH } → 编号 */
  votingModels: Record<string, number>
  /** 持有类型：{ CURRENCY, ASSET, NXT } → 编号 */
  holdingTypes: Record<string, number>
  /** 可修剪消息最大长度 */
  maxPrunableMessageLength: number
  /** 混币参与者状态：{ CANCELLED, REGISTERED, PROCESSED, VERIFIED } → 编号 */
  shufflingParticipantStates: Record<string, number>
  /** 被禁用的 API 标签列表 */
  disabledAPITags: string[] | string[][]
  /** 区块 payload 最大长度 */
  maxBlockPayloadLength: number
  /** phasing 最大持续时间 */
  maxPhasingDuration: number
  /** 代理节点不转发的请求列表 */
  proxyNotForwardedRequests: string[]
  /** 铸币哈希算法：{ SHA256, SHA3, SCRYPT, Keccak25 } → 编号 */
  mintingHashAlgorithms: Record<string, number>
  /** 任意消息最大长度 */
  maxArbitraryMessageLength: number
  /** API 标签映射：tag → { name, enabled } */
  apiTags: Record<string, ApiTagConfig>
  /** 标记数据最大数据长度 */
  maxTaggedDataDataLength: number
  /** 混币阶段：{ CANCELLED, DONE, PROCESSING, BLAME, REGISTRATION, VERIFICATION } → 编号 */
  shufflingStages: Record<string, number>
  /** 被禁用的 API 列表 */
  disabledAPIs: string[] | string[][]
  /** 创世区块 ID（数字字符串，可能为负） */
  genesisBlockId: string
  /** 货币类型：{ EXCHANGEABLE, CLAIMABLE, MINTABLE, CONTROLLABLE, RESERVABLE, NON_SHUFFLEABLE } → 编号 */
  currencyTypes: Record<string, number>
  /** 节点状态：{ DISCONNECTED, NON_CONNECTED, CONNECTED } → 编号 */
  peerStates: Record<string, number>
  /** NRCS 纪元起点（毫秒，2013-11-25 08:00:00 UTC） */
  epochBeginning: number
  /** 最低余额模型：{ NQT, CURRENCY, ASSET, NONE } → 编号 */
  minBalanceModels: Record<string, number>
  /** phasing 哈希算法：{ SHA256, RIPEMD160, RIPEMD160_SHA256 } → 编号 */
  phasingHashAlgorithms: Record<string, number>
  /** 哈希算法：{ SHA256, SHA3, SCRYPT, RIPEMD160, Keccak25, RIPEMD160_SHA256 } → 编号 */
  hashAlgorithms: Record<string, number>
  /** requestType → 配置映射 */
  requestTypes: Record<string, RequestTypeConfig>
}

// ============================================================================
// 运行行时容器默认值（加载前占位）
// ============================================================================

/** 空的运行时常量容器，加载完成前作为回退默认值 */
export const EMPTY_RUNTIME_MAP: Record<string, never> = {}

/**
 * 判断给定的运行时映射是否尚未加载（空对象）。
 * 对标参考 `nrs.constants.js` 中 `$.isEmptyObject(NRS.constants.REQUEST_TYPES)` 的判定。
 *
 * @param map - 运行时常量映射
 * @returns 为空对象时返回 true
 */
export function isRuntimeMapEmpty(map: Record<string, unknown>): boolean {
  return Object.keys(map).length === 0
}
