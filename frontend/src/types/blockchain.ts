/**
 * 区块链模块类型定义
 */

// ============ 交易类型 ============

// 交易状态枚举
export enum TransactionStatus {
  PENDING = 'pending',
  SUCCESS = 'success',
  FAILED = 'failed',
  REVERTED = 'reverted'
}

// 交易方向枚举
export enum TransactionDirection {
  IN = 'in',
  OUT = 'out',
  SELF = 'self'
}

// 交易对象
export interface Transaction {
  hash: string
  from: string
  to: string
  value: string  // 以 wei 为单位
  gasPrice: number  // wei
  gasLimit: number
  gasUsed?: number  // 实际使用的 gas
  nonce: number
  data?: string  // 附加数据或合约调用数据
  status: TransactionStatus
  blockNumber: number
  blockHash?: string
  transactionIndex: number
  timestamp: number
  confirmations: number
  chainId: number
  direction?: TransactionDirection  // 相对于当前用户的方向

  // 附加字段（如果服务端提供）
  fee?: string  // 交易费用
  tokenTransfers?: TokenTransfer[]  // 代币转账记录
  logs?: TransactionLog[]  // 事件日志
}

// 交易发送请求
export interface SendTransactionRequest {
  from: string
  to: string
  value: string
  gasPrice?: number
  gasLimit: number
  data?: string
  nonce?: number

  // 钱包签名相关
  privateKey?: string  // 私钥（前端不推荐，仅用于测试）
  signedTx?: string  // 已签名的交易原始数据
}

// 批量交易请求
export interface BatchSendTransactionRequest {
  transactions: SendTransactionRequest[]
}

// 交易筛选条件
export interface TransactionFilter {
  from?: string
  to?: string
  status?: TransactionStatus
  minBlock?: number
  maxBlock?: number
  fromTime?: number
  toTime?: number
}

// 交易费用估算响应
export interface FeeEstimate {
  gasPrice: number
  gasLimit: number
  maxFee: string
  recommendedFee: string
  priorityFee: number
  baseFee: number
}

// 交易收据
export interface TransactionReceipt {
  txHash: string
  status: TransactionStatus
  blockNumber: number
  blockHash: string
  gasUsed: number
  cumulativeGasUsed: number
  contractAddress?: string
  logs: TransactionLog[]
  logsBloom: string
  effectiveGasPrice?: number
}

// 交易日志（事件）
export interface TransactionLog {
  logIndex: number
  transactionIndex: number
  transactionHash: string
  blockHash: string
  blockNumber: number
  address: string
  data: string
  topics: string[]
  removed: boolean
}

// 代币转账记录
export interface TokenTransfer {
  tokenAddress: string
  from: string
  to: string
  value: string
  tokenSymbol?: string
  tokenDecimals?: number
  tokenName?: string
}

// ============ 区块类型 ============

// 区块信息
export interface BlockInfo {
  number: number
  hash: string
  parentHash: string
  nonce: string
  sha3Uncles: string
  logsBloom: string
  transactionsRoot: string
  stateRoot: string
  receiptsRoot: string
  miner: string
  difficulty: bigint
  totalDifficulty: bigint
  extraData: string
  size: number
  gasLimit: number
  gasUsed: number
  timestamp: number
  transactionCount: number
  transactions?: Transaction[]

  // EIP-1559 字段
  baseFeePerGas?: number
  withdrawalsRoot?: string
  withdrawals?: any[]
}

// ============ 合约类型 ============

// 合约对象
export interface Contract {
  address: string
  name: string
  abi: ContractABI
  bytecode: string
  deployer: string
  createdAt: string
  verified: boolean
  sourceCode?: string
  compilerVersion?: string
  optimizationUsed?: boolean
  runs?: number
  constructorArguments?: string
}

// 合约ABI（简化版）
export interface ContractABI {
  type: 'constructor' | 'function' | 'event' | 'fallback' | 'receive'
  name?: string
  inputs: ContractABIParameter[]
  outputs?: ContractABIParameter[]
  stateMutability?: 'pure' | 'view' | 'nonpayable' | 'payable'
  anonymous?: boolean
}

// 合约ABI参数
export interface ContractABIParameter {
  name: string
  type: string
  indexed?: boolean
  components?: ContractABIParameter[]
}

// 合约部署请求
export interface ContractDeployRequest {
  bytecode: string
  constructorArgs?: any[]
 abi?: ContractABI  // 可选，部署后需要验证
  estimateGas?: boolean
  gasLimit?: number
  gasPrice?: number
  maxFeePerGas?: number
  maxPriorityFeePerGas?: number
  value?: string  // 发送的ETH金额
}

// 合约调用请求（只读）
export interface ContractCallRequest {
  contractAddress: string
  abi: ContractABI
  functionName: string
  args?: any[]
  from?: string
  block?: 'latest' | 'earliest' | 'pending' | number
}

// 合约交易请求（写操作）
export interface ContractTransactionRequest extends ContractCallRequest {
  gasLimit?: number
  gasPrice?: number
  maxFeePerGas?: number
  maxPriorityFeePerGas?: number
  value?: string
  nonce?: number
  signedTx?: string  // 已签名的交易
}

// 合约事件
export interface ContractEvent {
  eventName: string
  transactionHash: string
  blockNumber: number
  transactionIndex: number
  logIndex: number
  address: string
  returnedValues: Record<string, any>
  rawData: string
  topics: string[]
  timestamp: number
}

// ============ 节点类型 ============

// 节点状态
export enum NodeStatus {
  ACTIVE = 'active',
  INACTIVE = 'inactive',
  SYNCING = 'syncing',
  ERROR = 'error'
}

// 节点信息
export interface NodeInfo {
  chainId: number
  chainName: string
  networkId: number
  nodeVersion: string
  ethereumVersion: string
  protocolVersion: number
  confirmations: number
  isSyncing: boolean
  syncStatus?: SyncStatus
}

// 同步状态
export interface SyncStatus {
  startingBlock: number
  currentBlock: number
  highestBlock: number
  knownStates: number
  pulledStates: number
  startingHash: string
  currentHash: string
  highestHash: string
}

// 节点对等端（Peer）
export interface PeerInfo {
  enode: string
  id: string
  address: string
  port: number
  connected: boolean
  latency?: number  // 毫秒
  lastSeen: string
  capabilities: string[]
  protocols: PeerProtocol[]
}

// 对等端协议信息
export interface PeerProtocol {
  name: string
  version: number
  difficulty?: bigint
  head?: string
}

// 节点监控指标
export interface NodeMetrics {
  cpuUsage: number  // CPU使用率（%）
  memoryUsage: number  // 内存使用（字节）
  memoryTotal: number
  diskUsage: number  // 磁盘使用（字节）
  diskTotal: number
  networkIn: number  // 网络接收（字节）
  networkOut: number  // 网络发送（字节）
  peerCount: number
  blockCount: number
  tps: number  // 每秒交易数
  pendingTxCount: number
  uptime: number  // 运行时间（秒）
}

// ============ 代币类型 ============

// 代币信息
export interface TokenInfo {
  address: string
  symbol: string
  name: string
  decimals: number
  totalSupply: string
  circulatingSupply?: string
  logoURI?: string
  contractType: 'ERC20' | 'ERC721' | 'ERC1155' | 'OTHER'
}

// 代币余额
export interface TokenBalance {
  tokenAddress: string
  balance: string
  symbol: string
  decimals: number
}

// ============ 通用区块链类型 ============

// 哈希
export type Hash = string

// 地址（20字节十六进制字符串）
export type Address = string

// 私钥
export type PrivateKey = string

// 公钥
export type PublicKey = string

// 签名字符串
export type Signature = string

// 区块高度
export type BlockNumber = number

// 时间戳
export type Timestamp = number

// 金额（wei）
export type Wei = bigint | string

// Gas价格
export type GasPrice = number

// 链ID
export type ChainId = number

// 链信息
export interface ChainInfo {
  chainId: number
  chainName: string
  nativeCurrency: {
    name: string
    symbol: string
    decimals: number
  }
  rpcUrls: string[]
  blockExplorerUrls?: string[]
  iconUrl?: string
  isTestnet: boolean
}
