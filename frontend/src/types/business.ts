import type { UserRole, TxStatus, ChainId } from './enums'

/**
 * 用户信息
 */
export interface User {
  id: number
  name: string
  email: string
  wallet_address: string
  role: UserRole
  avatar?: string
  created_at: string  // ISO 8601
  updated_at: string
}

/**
 * 登录凭证
 */
export interface LoginCredentials {
  wallet_address: string
  signature: string  // 钱包签名
  message: string    // 签名消息
}

/**
 * 注册凭证
 */
export interface RegisterCredentials extends LoginCredentials {
  name: string
  email: string
}

/**
 * 交易信息
 */
export interface Transaction {
  hash: string
  block_number: number
  from: string
  to: string | null  // 合约创建时为 null
  value: string      // wei 单位，字符串避免精度丢失
  gas_used: number
  gas_limit: number
  gas_price: string  // wei 单位
  status: TxStatus
  input: string      // 合约调用数据（hex）
  nonce: number
  transaction_index: number
  created_at: string // ISO 8601
  confirmed_at?: string
}

/**
 * 发送交易参数
 */
export interface SendTxParams {
  from: string
  to: string
  value: string      // wei 单位
  gasPrice?: string
  gasLimit?: number
  data?: string      // 合约调用数据（可选）
  nonce?: number
}

/**
 * 预估 Gas 返回
 */
export interface GasEstimate {
  gas_limit: number
  gas_price: string
  estimated_cost: string  // wei 单位
}

/**
 * 合约信息
 */
export interface Contract {
  address: string
  name: string
  abi: ContractABI[]
  bytecode: string
  source_code?: string
  deployed_at: string  // ISO 8601
  deployer: string
  is_verified: boolean
}

/**
 * 合约 ABI 条目
 */
export interface ContractABI {
  type: 'function' | 'event' | 'constructor' | 'fallback' | 'receive'
  name: string
  inputs: Array<{
    name: string
    type: string
    indexed?: boolean  // 事件专用
  }>
  outputs?: Array<{
    name: string
    type: string
  }>
  stateMutability?: 'view' | 'pure' | 'payable' | 'nonpayable'
  anonymous?: boolean  // 事件专用
}

/**
 * 合约部署参数
 */
export interface DeployContractParams {
  name: string
  bytecode: string
  abi: ContractABI[]
  constructorArgs?: any[]  // 构造函数参数
  from: string
  gasLimit?: number
  gasPrice?: string
  value?: string  // 部署时发送的 ETH（wei）
}

/**
 * 合约调用参数
 */
export interface CallContractParams {
  address: string
  abi: ContractABI[]
  functionName: string
  args?: any[]
  from: string
  gasLimit?: number
  gasPrice?: string
  value?: string  //  payable 函数使用
}

/**
 * 合约事件日志
 */
export interface ContractEvent {
  block_number: number
  transaction_hash: string
  log_index: number
  address: string
  event_name: string
  args: Record<string, any>
  raw_data: string
  block_timestamp: number
}

/**
 * 节点信息
 */
export interface NodeInfo {
  node_id: string
  version: string
  block_number: number
  peer_count: number
  network_id: number
  chain_id: ChainId
  uptime: number  // 秒
  sync_status: 'synced' | 'syncing' | 'caught_up'
  latest_block_hash: string
  total_difficulty: string
  listening: boolean
  mining: boolean
  active: boolean
}

/**
 * 网络统计
 */
export interface NetworkStats {
  total_transactions: number
  total_blocks: number
  total_accounts: number
  tps: number  // 每秒交易数（最近1分钟）
  block_time: number  // 平均出块时间（秒）
  difficulty: string
  gas_price_avg: string
  pending_txs: number
}

/**
 * 区块信息
 */
export interface BlockInfo {
  number: number
  hash: string
  parent_hash: string
  nonce: string
  timestamp: number  // Unix timestamp
  transactions: string[]  // 交易哈希列表
  transaction_count: number
  gas_used: number
  gas_limit: number
  miner: string
  extra_data: string
  logs_bloom: string
  total_difficulty?: string
}

/**
 * 钱包余额
 */
export interface Balance {
  address: string
  balance: string       // wei 单位
  balance_eth: number   // ETH 单位
  token_balances?: Array<{
    contract: string
    symbol: string
    balance: string
    decimals: number
  }>
}

/**
 * Gas 价格
 */
export interface GasPrice {
  slow: string
  standard: string
  fast: string
  instant: string
  base_fee: string
}

/**
 * 钱包建议的 Gas 价格
 */
export interface GasOracle {
  estimated_base_fee: string
  estimated_priority_fee: string
  max_priority_fee_per_gas: string
  max_fee_per_gas: string
}

/**
 * 接收的 Token 转账（ERC20）
 */
export interface TokenTransfer {
  from: string
  to: string
  contract: string
  symbol: string
  amount: string
  decimals: number
  transaction_hash: string
  block_number: number
  timestamp: number
}

/**
 * 合约验证信息
 */
export interface ContractVerification {
  address: string
  is_verified: boolean
  compiler_version?: string
  optimization_used?: boolean
  source_code?: string
  abi?: ContractABI[]
}
