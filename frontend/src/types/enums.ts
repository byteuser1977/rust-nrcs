/**
 * 用户角色枚举
 */
export enum UserRole {
  ADMIN = 'admin',
  USER = 'user',
  GUEST = 'guest'
}

/**
 * 交易状态枚举
 */
export enum TxStatus {
  PENDING = 'pending',
  CONFIRMING = 'confirming',
  SUCCESS = 'success',
  FAILED = 'failed'
}

/**
 * 链 ID 枚举
 */
export enum ChainId {
  MAINNET = 1,
  ROPSTEN = 3,
  RINKEBY = 4,
  GOERLI = 5,
  KOVAN = 42,
  BSC_MAINNET = 56,
  BSC_TESTNET = 97,
  POLYGON_MAINNET = 137,
  POLYGON_TESTNET = 80001,
  ARBITRUM_MAINNET = 42161,
  ARBITRUM_TESTNET = 421613,
  OPTIMISM_MAINNET = 10,
  OPTIMISM_TESTNET = 69,
  LOCALNET = 1337
}

/**
 * 错误码枚举（业务错误码，不同于 HTTP 状态码）
 */
export enum BusinessErrorCode {
  // Auth (1000-1999)
  WALLET_NOT_CONNECTED = 1001,
  INVALID_SIGNATURE = 1002,
  TOKEN_EXPIRED = 1003,
  TOKEN_INVALID = 1004,

  // Transaction (2000-2999)
  INSUFFICIENT_BALANCE = 2001,
  GAS_PRICE_TOO_LOW = 2002,
  GAS_LIMIT_EXCEEDED = 2003,
  TX_FAILED = 2004,
  TX_REJECTED = 2005,
  NONCE_TOO_LOW = 2006,

  // Contract (3000-3999)
  CONTRACT_NOT_FOUND = 3001,
  CALL_FAILED = 3002,
  INVALID_ABI = 3003,
  CONTRACT_ALREADY_EXISTS = 3004,
  CONTRACT_VERIFICATION_FAILED = 3005,

  // Node (4000-4999)
  NODE_UNAVAILABLE = 4001,
  SYNC_FAILED = 4002,
  NETWORK_ERROR = 4003,

  // Common (5000-5999)
  INVALID_PARAMS = 5001,
  RESOURCE_NOT_FOUND = 5002,
  RESOURCE_ALREADY_EXISTS = 5003,
  RATE_LIMIT_EXCEEDED = 5004,
  INTERNAL_ERROR = 5999
}

/**
 * 网络状态枚举
 */
export enum NetworkStatus {
  CONNECTED = 'connected',
  CONNECTING = 'connecting',
  DISCONNECTED = 'disconnected',
  RECONNECTING = 'reconnecting'
}

/**
 * 余额单位枚举
 */
export enum BalanceUnit {
  WEI = 'wei',
  GWEI = 'gwei',
  ETH = 'eth',
  TOKEN = 'token'
}

/**
 * 日志级别枚举
 */
export enum LogLevel {
  DEBUG = 'debug',
  INFO = 'info',
  WARN = 'warn',
  ERROR = 'error'
}

/**
 * 通知类型枚举
 */
export enum NotificationType {
  SUCCESS = 'success',
  WARNING = 'warning',
  INFO = 'info',
  ERROR = 'error'
}

/**
 * 主题模式枚举
 */
export enum ThemeMode {
  LIGHT = 'light',
  DARK = 'dark'
}

/**
 * 权限操作枚举
 */
export enum PermissionAction {
  VIEW = 'view',
  CREATE = 'create',
  EDIT = 'edit',
  DELETE = 'delete',
  ADMIN = 'admin'
}
