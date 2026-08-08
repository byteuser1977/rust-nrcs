import { nrcsGet, nrcsPost } from '../nrcs-client'

export interface NrcsAccount {
  account: string
  accountRS: string
  name?: string
  description?: string
  publicKey?: string
  balanceNQT: string
  unconfirmedBalanceNQT: string
  forgedBalanceNQT: string
  guaranteedBalanceNQT: string
  effectiveBalanceNRCS: number
  requestProcessingTime?: number
  // === Leasing 相关（对标 nrs.js:1398-1426 updateAccountLeasingStatus） ===
  /** 当前出租开始高度（对标 NRS.accountInfo.currentLeasingHeightFrom） */
  currentLeasingHeightFrom?: number
  /** 当前出租结束高度（对标 NRS.accountInfo.currentLeasingHeightTo） */
  currentLeasingHeightTo?: number
  /** 当前承租方账户 ID（对标 NRS.accountInfo.currentLessee） */
  currentLessee?: string
  /** 当前承租方 RS 地址（对标 NRS.accountInfo.currentLesseeRS） */
  currentLesseeRS?: string
  /** 下一轮出租开始高度（对标 NRS.accountInfo.nextLeasingHeightFrom） */
  nextLeasingHeightFrom?: number
  /** 下一轮出租结束高度（对标 NRS.accountInfo.nextLeasingHeightTo） */
  nextLeasingHeightTo?: number
  /** 下一轮承租方账户 ID（对标 NRS.accountInfo.nextLessee） */
  nextLessee?: string
  /** 出租方列表（对标 NRS.accountInfo.lessors） */
  lessors?: string[]
  /** 出租方 RS 列表（对标 NRS.accountInfo.lessorsRS） */
  lessorsRS?: string[]
  // === Account Control 相关（对标 nrs.js:1497 updateAccountControlStatus） ===
  /** 账户控制列表（如 ['PHASING_ONLY']，对标 NRS.accountInfo.accountControls） */
  accountControls?: string[]
  /** Phasing Only 控制详情（由 getPhasingOnlyControl 填充） */
  phasingOnly?: NrcsPhasingOnlyControl
  // === 资产/货币相关（对标 nrs.js:1167-1200 getAccountInfo） ===
  /** 资产余额列表（includeAssets=true 时返回） */
  assetBalances?: NrcsAssetBalance[]
  /** 账户货币列表（includeCurrencies=true 时返回） */
  accountCurrencies?: NrcsCurrencyBalance[]
  /** 未确认资产余额列表 */
  unconfirmedAssetBalances?: NrcsAssetBalance[]
}

/** 资产余额（对标 getAccount 响应中的 assetBalances 元素） */
export interface NrcsAssetBalance {
  /** 资产 ID */
  asset: string
  /** 余额数量 QNT（base units） */
  balanceQNT: string
  /** 未确认余额数量 QNT */
  unconfirmedBalanceQNT?: string
}

/** 货币余额（对标 getAccount 响应中的 accountCurrencies 元素） */
export interface NrcsCurrencyBalance {
  /** 货币 ID */
  currency: string
  /** 余额数量 QNT */
  balanceQNT: string
  /** 未确认余额数量 QNT */
  unconfirmedBalanceQNT?: string
}

/**
 * Phasing Only 控制详情（对标 getPhasingOnlyControl 响应）。
 *
 * 用于 updateAccountControlStatus（nrs.js:1491-1531），描述账户的
 * 强制审批策略（quorum/whitelist/minBalance/holding 等）。
 */
export interface NrcsPhasingOnlyControl {
  /** 账户 RS 地址 */
  account?: string
  /** 账户 ID */
  accountID?: string
  /** 投票模型（0=ACCOUNT,1=NQT,5=HASH 等） */
  votingModel: number
  /** 法定人数（NXT 数值，对标 NRS.phasingControlObjectToPhasingParams） */
  quorum?: string
  /** 白名单账户 ID 列表 */
  phasingWhitelisted?: string[]
  /** 最小余额模型 */
  minBalanceModel?: number
  /** 最小余额 NQT */
  minBalance?: string
  /** 持有类型（0=NXT,1=ASSET,2=CURRENCY） */
  holding?: number
  /** 持有资产/货币 ID */
  holdingId?: string
  /** 最小持续时间（区块数） */
  minDuration?: number
  /** 最大持续时间（区块数） */
  maxDuration?: number
  /** 最大手续费 NQT */
  maxFees?: string
  /** 交易 ID */
  transaction?: string
  /** 区块高度 */
  height?: number
  requestProcessingTime?: number
}

export interface NrcsBalance {
  account: string
  accountRS: string
  balanceNQT: string
  unconfirmedBalanceNQT: string
  effectiveBalanceNRCS: number
  guaranteedBalanceNQT: string
  requestProcessingTime?: number
}

export interface NrcsTransaction {
  transaction?: string
  sender?: string
  senderRS?: string
  recipient?: string
  recipientRS?: string
  amountNQT?: string
  feeNQT?: string
  type?: number
  subtype?: number
  timestamp?: number
  height?: number
  block?: string
  signature?: string
  signatureHash?: string
  fullHash?: string
  confirmations?: number
  attachment?: Record<string, any>
  senderPublicKey?: string
  recipientPublicKey?: string
  blockTimestamp?: number
  requestProcessingTime?: number
}

export interface NrcsUnconfirmedTransaction {
  transaction?: string
  sender?: string
  senderRS?: string
  recipient?: string
  recipientRS?: string
  amountNQT?: string
  feeNQT?: string
  type?: number
  subtype?: number
  timestamp?: number
  attachment?: Record<string, any>
  requestProcessingTime?: number
}

export interface NrcsBlock {
  block: string
  height: number
  generator: string
  generatorRS: string
  timestamp: number
  numberOfTransactions: number
  totalAmountNQT: string
  totalFeeNQT: string
  payloadLength: number
  version: number
  baseTarget: string
  cumulativeDifficulty: string
  previousBlock?: string
  nextBlock?: string
  payloadHash: string
  generationSignature: string
  previousBlockHash: string
  blockSignature: string
  transactions: NrcsTransaction[]
  requestProcessingTime?: number
}

export interface NrcsPeer {
  address: string
  port: number
  state: number
  announcedAddress?: string
  shareAddress?: boolean
  software?: string
  version?: string
  application?: string
  platform?: string
  lastUpdated?: number
  requestProcessingTime?: number
}

export interface NrcsBlockchainStatus {
  application: string
  version: string
  time: number
  lastBlock: string
  lastBlockHeight: number
  cumulativeDifficulty: string
  numberOfBlocks: number
  lastBlockchainFeeder?: string
  lastBlockchainFeederHeight: number
  isScanning: boolean
  isDownloading: boolean
  maxRollback: number
  isTestnet: boolean
  blockchainState: string
  requestProcessingTime?: number
  /** 是否为轻客户端（对标 NRS.state.isLightClient，nrs.js:1651 用） */
  isLightClient?: boolean
  /** 是否作为 API 代理（对标 NRS.state.apiProxy，nrs.js:438/512 用） */
  apiProxy?: boolean
  /** 账本裁剪保留数（对标 NRS.state.ledgerTrimKeep，nrs.js:441 用） */
  ledgerTrimKeep?: number
  /** 最大交易数（对标 NRS.state.maxTransactions） */
  maxTransactions?: number
  /** 当前账户数（对标 NRS.state.numberOfAccounts） */
  numberOfAccounts?: number
  /** 当前交易数（对标 NRS.state.numberOfTransactions） */
  numberOfTransactions?: number
  /** 当前连接的 peer 数（对标 NRS.state.numberOfPeers） */
  numberOfPeers?: number
  /** 当前解锁账户数（对标 NRS.state.numberOfUnlockedAccounts） */
  numberOfUnlockedAccounts?: number
  /** 总有效余额 NQT（对标 NRS.state.totalEffectiveBalance） */
  totalEffectiveBalance?: string
  /** 平均出块时间（对标 NRS.state.averageBlockGenerationTime） */
  averageBlockGenerationTime?: number
}

export interface NrcsAlias {
  alias: string
  aliasName: string
  account: string
  accountRS: string
  timestamp: number
  aliasURI?: string
  requestProcessingTime?: number
}

export interface NrcsAsset {
  asset: string
  name: string
  description?: string
  quantityQNT: string
  decimals: number
  issuer: string
  issuerRS: string
  requestProcessingTime?: number
}

export interface NrcsCurrency {
  currency: string
  code: string
  name?: string
  description?: string
  decimals: number
  issuer: string
  issuerRS: string
  type?: number
  algorithm?: number
  /** 发行高度（对标参考 issuanceHeight） */
  issuanceHeight?: number
  /** 最小每单位储备 NQT（对标参考 minReservePerUnitNQT） */
  minReservePerUnitNQT?: string
  /** 当前每单位储备 NQT（对标参考 currentReservePerUnitNQT） */
  currentReservePerUnitNQT?: string
  /** 储备供应量 QNT（对标参考 reserveSupply） */
  reserveSupply?: string
  /** 初始供应量 QNT（对标参考 initialSupply） */
  initialSupply?: string
  /** 最大供应量 QNT（对标参考 maxSupply） */
  maxSupply?: string
  /** 当前供应量 QNT（对标参考 currentSupply） */
  currentSupply?: string
  requestProcessingTime?: number
}

export interface NrcsPoll {
  poll: string
  name: string
  description?: string
  account: string
  accountRS: string
  finishHeight: number
  finished: boolean
  requestProcessingTime?: number
}

export interface NrcsDGSProduct {
  goods: string
  name: string
  description?: string
  quantity: number
  priceNQT: string
  seller: string
  sellerRS: string
  tags?: string
  delisted: boolean
  requestProcessingTime?: number
}

export interface NrcsDGSPurchase {
  purchase: string
  goods: string
  name: string
  buyer: string
  buyerRS: string
  seller: string
  sellerRS: string
  quantity: number
  priceNQT: string
  deadline: number
  note?: string
  pending: boolean
  requestProcessingTime?: number
}

export interface NrcsMessage {
  transaction: string
  sender: string
  senderRS: string
  recipient: string
  recipientRS: string
  timestamp: number
  attachment?: Record<string, any>
  requestProcessingTime?: number
  /** 发送方公钥（hex，getAccountMessages 返回，用于加密消息解密） */
  senderPublicKey?: string
  /** 接收方公钥（hex，部分场景返回） */
  recipientPublicKey?: string
}

export interface NrcsGenerator {
  account: string
  accountRS: string
  deadline: number
  hitTime: number
  requestProcessingTime?: number
}

export interface NrcsToken {
  token: string
  requestProcessingTime?: number
}

export const nrcsApi = {
  /**
   * 获取账户信息（对标 nrs.js:1102 getAccount）。
   *
   * @param account RS 地址或数字账户 ID
   * @param options 可选参数：includeLessors/includeAssets/includeCurrencies/includeEffectiveBalance
   */
  getAccount(
    account: string,
    options?: {
      includeLessors?: boolean
      includeAssets?: boolean
      includeCurrencies?: boolean
      includeEffectiveBalance?: boolean
    },
  ) {
    // 兼容旧签名：getAccount(account, includeLessors?)
    const params: Record<string, any> = { account }
    if (typeof options === 'boolean') {
      if (options) params.includeLessors = true
    } else if (options) {
      if (options.includeLessors) params.includeLessors = true
      if (options.includeAssets) params.includeAssets = true
      if (options.includeCurrencies) params.includeCurrencies = true
      if (options.includeEffectiveBalance) params.includeEffectiveBalance = true
    }
    return nrcsGet<NrcsAccount>('getAccount', params)
  },

  getBalance(account: string) {
    return nrcsGet<NrcsBalance>('getBalance', { account })
  },

  getAccountId(secretPhrase: string) {
    return nrcsPost<{ account: string; accountRS: string; publicKey: string }>('getAccountId', { secretPhrase })
  },

  getAccountPublicKey(account: string) {
    return nrcsGet<{ publicKey: string }>('getAccountPublicKey', { account })
  },

  /**
   * 获取账户的 Phasing Only 控制详情（对标 nrs.js:1498 getPhasingOnlyControl）。
   *
   * 用于 updateAccountControlStatus 检测账户是否设置了强制审批策略。
   *
   * @param account RS 地址或数字账户 ID
   */
  getPhasingOnlyControl(account: string) {
    return nrcsGet<NrcsPhasingOnlyControl>('getPhasingOnlyControl', { account })
  },

  getEffectiveBalance(account: string) {
    return nrcsGet<{ effectiveBalanceNRCS: number }>('getEffectiveBalance', { account })
  },

  setAccountInfo(data: { secretPhrase: string; name: string; description: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('setAccountInfo', data)
  },

  setAccountProperty(data: { secretPhrase: string; recipient: string; property: string; value: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('setAccountProperty', data)
  },

  /**
   * 删除账户属性（对标 deleteAccountProperty API）。
   *
   * 用于资金监控模块移除被监控账户（nrs.monitors.js:231 remove_monitored_account_modal）。
   */
  deleteAccountProperty(data: { secretPhrase: string; recipient?: string; property: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('deleteAccountProperty', data)
  },

  /**
   * 查询账户属性（对标 getAccountProperties API）。
   *
   * @param options.recipient 查询指定账户的属性
   * @param options.setter 属性设置者（资金监控模块按 setter 查询被监控账户，nrs.monitors.js:162）
   * @param options.property 属性名（资金监控模块按 property 过滤，nrs.monitors.js:163）
   * @param options.firstIndex 分页起始索引
   * @param options.lastIndex 分页结束索引
   */
  getAccountProperties(options: { recipient?: string; setter?: string; property?: string; firstIndex?: number; lastIndex?: number } | string) {
    const params = typeof options === 'string' ? { recipient: options } : options
    return nrcsGet<{ properties: any[] }>('getAccountProperties', params)
  },

  searchAccounts(query: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ accounts: any[] }>('searchAccounts', { query, firstIndex, lastIndex })
  },

  getBlockchainTransactions(
    account: string,
    firstIndex?: number,
    lastIndex?: number,
    type?: number,
    subtype?: number,
    /** 额外参数（timestamp/includePhasingResult 等，对标 nrs.notifications.js:149-153） */
    extraParams?: { timestamp?: number; includePhasingResult?: boolean },
  ) {
    return nrcsGet<{ transactions: NrcsTransaction[] }>('getBlockchainTransactions', {
      account, firstIndex, lastIndex, type, subtype,
      ...extraParams,
    })
  },

  getUnconfirmedTransactions(account?: string) {
    return nrcsGet<{ unconfirmedTransactions: NrcsUnconfirmedTransaction[] }>('getUnconfirmedTransactions', { account })
  },

  /**
   * 获取账户待审批（Phasing）交易数（对标 nrs.notifications.js:229 getAccountPhasedTransactionCount）。
   *
   * @param account RS 地址或数字账户 ID
   */
  getAccountPhasedTransactionCount(account: string) {
    return nrcsGet<{ numberOfPhasedTransactions: number }>('getAccountPhasedTransactionCount', { account })
  },

  getTransaction(transaction: string) {
    return nrcsGet<NrcsTransaction>('getTransaction', { transaction })
  },

  sendMoney(data: { secretPhrase: string; recipient: string; amountNQT: string; feeNQT: string; deadline: number; referencedTransactionFullHash?: string; message?: string; messageIsText?: boolean }) {
    return nrcsPost<any>('sendMoney', data)
  },

  sendMessage(data: { secretPhrase: string; recipient: string; feeNQT: string; deadline: number; message: string; messageIsText?: boolean; messageToEncrypt?: string }) {
    return nrcsPost<any>('sendMessage', data)
  },

  /**
   * 广播交易（对标 nrs.server.js broadcastTransaction）。
   *
   * 支持两种模式：
   *   - transactionBytes：已签名的交易字节十六进制字符串
   *   - transactionJSON：已签名的交易 JSON（含 signature 字段）
   *
   * @param transactionBytes 已签名交易字节（与 transactionJSON 二选一）
   * @param transactionJSON 已签名交易 JSON 字符串（与 transactionBytes 二选一）
   */
  broadcastTransaction(transactionBytes?: string, transactionJSON?: string) {
    return nrcsPost<any>('broadcastTransaction', { transactionBytes, transactionJSON })
  },

  parseTransaction(transactionBytes?: string, transactionJSON?: string) {
    return nrcsGet<any>('parseTransaction', { transactionBytes, transactionJSON })
  },

  calculateFee(transactionBytes?: string, transactionJSON?: string) {
    return nrcsGet<{ feeNQT: string }>('calculateFee', { transactionBytes, transactionJSON })
  },

  getBlock(block?: string, height?: number, timestamp?: number, includeTransactions?: boolean) {
    return nrcsGet<NrcsBlock>('getBlock', { block, height, timestamp, includeTransactions })
  },

  getBlocks(firstIndex?: number, lastIndex?: number, includeTransactions?: boolean) {
    return nrcsGet<{ blocks: NrcsBlock[] }>('getBlocks', { firstIndex, lastIndex, includeTransactions })
  },

  /**
   * 获取指定账户锻造的区块列表（对标 nrs.blocks.js:224 getAccountBlocks+）。
   * 用于"我的锻造区块"视图。
   */
  getAccountBlocks(account: string, firstIndex?: number, lastIndex?: number, includeTransactions?: boolean) {
    return nrcsGet<{ blocks: NrcsBlock[] }>('getAccountBlocks', {
      account, firstIndex, lastIndex, includeTransactions,
    })
  },

  /**
   * 获取指定账户锻造的区块总数（对标 nrs.blocks.js:289 getAccountBlockCount+）。
   */
  getAccountBlockCount(account: string) {
    return nrcsGet<{ numberOfBlocks: number }>('getAccountBlockCount', { account })
  },

  getBlockchainStatus() {
    return nrcsGet<NrcsBlockchainStatus>('getBlockchainStatus')
  },

  getPeers(state?: string, includePeerInfo?: boolean, active?: boolean) {
    return nrcsGet<{ peers: NrcsPeer[] }>('getPeers', { state, includePeerInfo, active })
  },

  getPeer(peer: string) {
    return nrcsGet<NrcsPeer>('getPeer', { peer })
  },

  addPeer(peer: string) {
    return nrcsPost<any>('addPeer', { peer })
  },

  getState(includeCounts?: boolean) {
    return nrcsGet<any>('getState', { includeCounts })
  },

  getTime() {
    return nrcsGet<{ time: number }>('getTime')
  },

  startForging(secretPhrase: string) {
    return nrcsPost<any>('startForging', { secretPhrase })
  },

  stopForging(secretPhrase?: string) {
    return nrcsPost<any>('stopForging', { secretPhrase })
  },

  getForging() {
    return nrcsGet<{ generators: NrcsGenerator[] }>('getForging')
  },

  getNextBlockGenerators(limit?: number) {
    return nrcsGet<{ generators: NrcsGenerator[] }>('getNextBlockGenerators', { limit })
  },

  /**
   * 设置 API 代理节点（对标 nrs.header.js NRS.forms.setAPIProxyPeer）。
   *
   * @param data.peer 远程节点地址
   * @param data.adminPassword 管理员密码
   */
  setAPIProxyPeer(data: { peer: string; adminPassword?: string }) {
    return nrcsPost<any>('setAPIProxyPeer', data)
  },

  /**
   * 黑名单 API 代理节点（对标 nrs.header.js NRS.forms.blacklistAPIProxyPeer）。
   *
   * @param data.adminPassword 管理员密码
   */
  blacklistAPIProxyPeer(data: { adminPassword?: string }) {
    return nrcsPost<any>('blacklistAPIProxyPeer', data)
  },

  getAlias(alias?: string, aliasName?: string) {
    return nrcsGet<NrcsAlias>('getAlias', { alias, aliasName })
  },

  getAliases(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ aliases: NrcsAlias[] }>('getAliases', { account, firstIndex, lastIndex })
  },

  getAliasCount(account: string) {
    return nrcsGet<{ numberOfAliases: number }>('getAliasCount', { account })
  },

  setAlias(data: { secretPhrase: string; aliasName: string; aliasURI: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('setAlias', data)
  },

  sellAlias(data: { secretPhrase: string; aliasName: string; priceNQT: string; buyer?: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('sellAlias', data)
  },

  buyAlias(data: { secretPhrase: string; aliasName: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('buyAlias', data)
  },

  deleteAlias(data: { secretPhrase: string; aliasName: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('deleteAlias', data)
  },

  getAsset(asset: string) {
    return nrcsGet<NrcsAsset>('getAsset', { asset })
  },

  getAssets(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ assets: NrcsAsset[] }>('getAssets', { firstIndex, lastIndex })
  },

  getAllAssets(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ assets: NrcsAsset[] }>('getAllAssets', { firstIndex, lastIndex })
  },

  /**
   * 按发行方账户查询其发行的所有资产（对标 nrs.assetexchange.js:169 getAssetsByIssuer）。
   *
   * 响应结构为 `assets` 数组的数组（每个账户一组），通常取 `response.assets[0]`。
   *
   * @param account 发行方账户 RS 或数字 ID
   * @param firstIndex 起始索引
   * @param lastIndex 结束索引
   */
  getAssetsByIssuer(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ assets: NrcsAsset[][] }>('getAssetsByIssuer', { account, firstIndex, lastIndex })
  },

  getAccountAssets(account: string) {
    return nrcsGet<{ assetBalances: any[] }>('getAccountAssets', { account })
  },

  getLastTrades(assetIds?: string) {
    return nrcsGet<{ trades: any[] }>('getLastTrades', { assetIds })
  },

  issueAsset(data: { secretPhrase: string; name: string; description: string; quantityQNT: string; decimals: number; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('issueAsset', data)
  },

  placeAskOrder(data: { secretPhrase: string; asset: string; quantityQNT: string; priceNQTPerShare: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('placeAskOrder', data)
  },

  placeBidOrder(data: { secretPhrase: string; asset: string; quantityQNT: string; priceNQTPerShare: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('placeBidOrder', data)
  },

  cancelAskOrder(data: { secretPhrase: string; order: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('cancelAskOrder', data)
  },

  cancelBidOrder(data: { secretPhrase: string; order: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('cancelBidOrder', data)
  },

  getAskOrders(asset: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ askOrders: any[] }>('getAskOrders', { asset, firstIndex, lastIndex })
  },

  getBidOrders(asset: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ bidOrders: any[] }>('getBidOrders', { asset, firstIndex, lastIndex })
  },

  getTrades(asset?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ trades: any[] }>('getTrades', { asset, firstIndex, lastIndex })
  },

  /**
   * 获取订单成交记录（对标 nrs.modals.transaction.js:1414 getOrderTrades）。
   *
   * 用于交易详情 modal 中展示 ask/bid 订单的成交明细。
   *
   * @param askOrder ask 订单 ID（与 bidOrder 二选一）
   * @param bidOrder bid 订单 ID
   */
  getOrderTrades(askOrder?: string, bidOrder?: string) {
    return nrcsGet<{ trades: any[] }>('getOrderTrades', { askOrder, bidOrder })
  },

  /**
   * 获取资产的预期买单（对标 nrs.assetexchange.js getExpectedBidOrders）。
   *
   * @param asset 资产 ID
   */
  getExpectedBidOrders(asset: string) {
    return nrcsGet<{ bidOrders: any[] }>('getExpectedBidOrders', { asset })
  },

  /**
   * 获取资产的预期卖单（对标 nrs.assetexchange.js getExpectedAskOrders）。
   *
   * @param asset 资产 ID
   */
  getExpectedAskOrders(asset: string) {
    return nrcsGet<{ askOrders: any[] }>('getExpectedAskOrders', { asset })
  },

  /**
   * 获取资产股息历史（对标 nrs.assetexchange.js:777 getAssetDividends）。
   *
   * @param asset 资产 ID
   * @param firstIndex 起始索引
   * @param lastIndex 结束索引
   */
  getAssetDividends(asset: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ dividends: any[] }>('getAssetDividends', { asset, firstIndex, lastIndex })
  },

  getAccountCurrentAskOrders(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ askOrders: any[] }>('getAccountCurrentAskOrders', { account, firstIndex, lastIndex })
  },

  getAccountCurrentBidOrders(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ bidOrders: any[] }>('getAccountCurrentBidOrders', { account, firstIndex, lastIndex })
  },

  getAccountCurrencies(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ currencyBalances: any[] }>('getAccountCurrencies', { account, firstIndex, lastIndex })
  },

  getAllExchanges(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ exchanges: any[] }>('getAllExchanges', { firstIndex, lastIndex })
  },

  getCurrency(currency: string) {
    return nrcsGet<NrcsCurrency>('getCurrency', { currency })
  },

  getCurrencies(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ currencies: NrcsCurrency[] }>('getCurrencies', { firstIndex, lastIndex })
  },

  getAllCurrencies(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ currencies: NrcsCurrency[] }>('getAllCurrencies', { firstIndex, lastIndex })
  },

  issueCurrency(data: { secretPhrase: string; code: string; name: string; description: string; decimals: number; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('issueCurrency', data)
  },

  mintCurrency(data: { secretPhrase: string; currency: string; nonce: string; unitsQNT: string; counter: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('currencyMint', data)
  },

  deleteCurrency(data: { secretPhrase: string; currency: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('deleteCurrency', data)
  },

  currencyReserveIncrease(data: { secretPhrase: string; currency: string; amountPerUnitNQT: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('currencyReserveIncrease', data)
  },

  currencyReserveClaim(data: { secretPhrase: string; currency: string; unitsQNT: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('currencyReserveClaim', data)
  },

  orderCurrency(data: { secretPhrase: string; currency: string; unitsQNT: string; rateNQT: string; offerType: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('exchangeCurrency', data)
  },

  transferCurrency(data: { secretPhrase: string; currency: string; unitsQNT: string; recipient: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('transferCurrency', data)
  },

  getSellOffers(currency: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ offers: any[] }>('getSellOffers', { currency, firstIndex, lastIndex })
  },

  getBuyOffers(currency: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ offers: any[] }>('getBuyOffers', { currency, firstIndex, lastIndex })
  },

  /**
   * 获取预期卖单报价（对标 nrs.monetarysystem.js:374 getExpectedSellOffers）。
   * 返回由未确认交易产生的预期卖单，与真实卖单合并展示。
   */
  getExpectedSellOffers(currency: string) {
    return nrcsGet<{ offers: any[] }>('getExpectedSellOffers', { currency })
  },

  /**
   * 获取预期买单报价（对标 nrs.monetarysystem.js:374 getExpectedBuyOffers）。
   * 返回由未确认交易产生的预期买单，与真实买单合并展示。
   */
  getExpectedBuyOffers(currency: string) {
    return nrcsGet<{ offers: any[] }>('getExpectedBuyOffers', { currency })
  },

  /**
   * 取消兑换报价（对标 nrs.monetarysystem.js cancelOffer）。
   */
  cancelOffer(data: { secretPhrase: string; offer: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('cancelOffer', data)
  },

  /**
   * 获取单个兑换报价（对标 nrs.modals.transaction.js:1502 getOffer）。
   *
   * @param offer 报价 ID
   */
  getOffer(offer: string) {
    return nrcsGet<{ buyOffer: any; sellOffer: any }>('getOffer', { offer })
  },

  /**
   * 按兑换请求获取成交记录（对标 nrs.modals.transaction.js:1463 getExchangesByExchangeRequest）。
   *
   * @param transaction 交易 ID
   */
  getExchangesByExchangeRequest(transaction: string) {
    return nrcsGet<{ exchanges: any[] }>('getExchangesByExchangeRequest', { transaction })
  },

  /**
   * 按报价获取成交记录（对标 nrs.modals.transaction.js:1522 getExchangesByOffer）。
   *
   * @param offer 报价 ID
   */
  getExchangesByOffer(offer: string) {
    return nrcsGet<{ exchanges: any[] }>('getExchangesByOffer', { offer })
  },

  /**
   * 获取货币创始人（对标 nrs.monetarysystem.js:423 getCurrencyFounders）。
   *
   * @param currency 货币 ID
   */
  getCurrencyFounders(currency: string) {
    return nrcsGet<{ founders: any[] }>('getCurrencyFounders', { currency })
  },

  /**
   * 获取铸造目标（对标 GetMintingTarget.java + CurrencyMinting.getNumericTarget）。
   *
   * 客户端使用返回的 targetBytes 和 counter 进行工作量证明计算：
   * 循环递增 counter，计算 hash(nonce || currencyId || units || counter || accountId)，
   * 直到 hash <= target（从高位字节比较）。
   *
   * @param currency 货币 ID
   * @param account 账户 RS 地址
   * @param units 铸造单位数（QNT）
   */
  getMintingTarget(currency: string, account: string, units: string) {
    return nrcsGet<{
      currency: string
      difficulty: string
      targetBytes: string
      counter: string
    }>('getMintingTarget', { currency, account, units })
  },

  /**
   * 获取货币持有人（对标 nrs.monetarysystem.js:1327 getCurrencyAccounts）。
   *
   * @param currency 货币 ID
   * @param firstIndex 起始索引
   * @param lastIndex 结束索引
   */
  getCurrencyAccounts(currency: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ accountCurrencies: any[] }>('getCurrencyAccounts', { currency, firstIndex, lastIndex })
  },

  /**
   * 获取账户的兑换请求（对标 nrs.monetarysystem.js:615 getAccountExchangeRequests）。
   *
   * @param account 账户 ID
   * @param firstIndex 起始索引
   * @param lastIndex 结束索引
   */
  getAccountExchangeRequests(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ exchangeRequests: any[] }>('getAccountExchangeRequests', { account, firstIndex, lastIndex })
  },

  getExchanges(currency?: string, account?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ exchanges: any[] }>('getExchanges', { currency, account, firstIndex, lastIndex })
  },

  getCurrencyTransfers(currency?: string, account?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ transfers: any[] }>('getCurrencyTransfers', { currency, account, firstIndex, lastIndex })
  },

  publishExchangeOffer(data: { secretPhrase: string; currency: string; buyRateNQT: string; sellRateNQT: string; totalBuyLimit: string; totalSellLimit: string; expirationHeight: number; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('publishExchangeOffer', data)
  },

  transferAsset(data: { secretPhrase: string; asset: string; quantityQNT: string; recipient: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('transferAsset', data)
  },

  getAssetAccounts(asset: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ accountAssets: any[] }>('getAssetAccounts', { asset, firstIndex, lastIndex })
  },

  getAssetTransfers(asset?: string, account?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ transfers: any[] }>('getAssetTransfers', { asset, account, firstIndex, lastIndex })
  },

  dividendPayment(data: { secretPhrase: string; asset: string; height: number; amountNQTPerShare: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dividendPayment', data)
  },

  increaseAssetShares(data: { secretPhrase: string; asset: string; quantityQNT: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('increaseAssetShares', data)
  },

  deleteAssetShares(data: { secretPhrase: string; asset: string; quantityQNT: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('deleteAssetShares', data)
  },

  searchDGSGoods(query?: string, tag?: string, seller?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ goods: NrcsDGSProduct[] }>('searchDGSGoods', { query, tag, seller, firstIndex, lastIndex })
  },

  dgsPriceChange(data: { secretPhrase: string; goods: string; priceNQT: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dgsPriceChange', data)
  },

  dgsQuantityChange(data: { secretPhrase: string; goods: string; deltaQuantity: number; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dgsQuantityChange', data)
  },

  dgsDelisting(data: { secretPhrase: string; goods: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dgsDelisting', data)
  },

  dgsDelivery(data: { secretPhrase: string; purchase: string; goodsData?: string; goodsIsText?: boolean; discountNQT?: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dgsDelivery', data)
  },

  dgsFeedback(data: { secretPhrase: string; purchase: string; message?: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dgsFeedback', data)
  },

  dgsRefund(data: { secretPhrase: string; purchase: string; refundNQT?: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dgsRefund', data)
  },

  getLastExchanges(currencyIds?: string) {
    return nrcsGet<{ exchanges: any[] }>('getLastExchanges', { currencyIds })
  },

  getPoll(poll: string) {
    return nrcsGet<NrcsPoll>('getPoll', { poll })
  },

  getPolls(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ polls: NrcsPoll[] }>('getPolls', { firstIndex, lastIndex })
  },

  createPoll(data: { secretPhrase: string; name: string; description: string; feeNQT: string; deadline: number; finishHeight: number; votingModel: number; minNumberOfOptions: number; maxNumberOfOptions: number; minRangeValue: number; maxRangeValue: number; options: string[] }) {
    return nrcsPost<any>('createPoll', data)
  },

  castVote(data: { secretPhrase: string; poll: string; vote: number[]; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('castVote', data)
  },

  getDGSGoods(seller?: string, firstIndex?: number, lastIndex?: number, inStockOnly?: boolean) {
    return nrcsGet<{ goods: NrcsDGSProduct[] }>('getDGSGoods', { seller, firstIndex, lastIndex, inStockOnly })
  },

  getDGSGoodsCount() {
    return nrcsGet<{ numberOfGoods: number }>('getDGSGoodsCount')
  },

  getDGSPurchase(purchase: string) {
    return nrcsGet<NrcsDGSPurchase>('getDGSPurchase', { purchase })
  },

  getDGSPurchases(buyer?: string, seller?: string, firstIndex?: number, lastIndex?: number, completed?: boolean) {
    return nrcsGet<{ purchases: NrcsDGSPurchase[] }>('getDGSPurchases', { buyer, seller, firstIndex, lastIndex, completed })
  },

  getDGSGoodsPurchases(goods: string, firstIndex?: number, lastIndex?: number, withPublicFeedbacks?: boolean) {
    return nrcsGet<{ purchases: NrcsDGSPurchase[] }>('getDGSGoodsPurchases', { goods, firstIndex, lastIndex, withPublicFeedbacks })
  },

  getDGSPendingPurchases(seller: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ purchases: NrcsDGSPurchase[] }>('getDGSPendingPurchases', { seller, firstIndex, lastIndex })
  },

  getDGSPurchaseCount(buyer?: string, seller?: string, completed?: boolean) {
    return nrcsGet<{ numberOfPurchases: number }>('getDGSPurchaseCount', { buyer, seller, completed })
  },

  dgsListing(data: { secretPhrase: string; name: string; description: string; tags: string; quantity: number; priceNQT: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dgsListing', data)
  },

  dgsPurchase(data: { secretPhrase: string; goods: string; quantity: number; priceNQT: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('dgsPurchase', data)
  },

  addFollowedPoll(data: { poll: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('addFollowedPoll', data)
  },

  searchTaggedData(query: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ data: any[] }>('searchTaggedData', { query, firstIndex, lastIndex })
  },

  getTaggedData(transaction: string) {
    return nrcsGet<any>('getTaggedData', { transaction })
  },

  uploadTaggedData(data: { secretPhrase: string; name: string; description: string; tags: string; data: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('uploadTaggedData', data)
  },

  extendTaggedData(data: { secretPhrase: string; transaction: string; data: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('extendTaggedData', data)
  },

  getAllTaggedData(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ data: any[] }>('getAllTaggedData', { firstIndex, lastIndex })
  },

  getAccountTaggedData(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ data: any[] }>('getAccountTaggedData', { account, firstIndex, lastIndex })
  },

  getDataTags(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ tags: string[] }>('getDataTags', { firstIndex, lastIndex })
  },

  getDataTagCount() {
    return nrcsGet<{ numberOfDataTags: number }>('getDataTagCount')
  },

  downloadTaggedData(transaction: string) {
    return nrcsGet<any>('downloadTaggedData', { transaction })
  },

  getAccountMessages(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ messages: NrcsMessage[] }>('getAccountMessages', { account, firstIndex, lastIndex })
  },

  readMessage(transaction: string, secretPhrase?: string) {
    return nrcsGet<any>('readMessage', { transaction, secretPhrase })
  },

  decryptMessages(data: { secretPhrase: string; transaction: string }) {
    return nrcsPost<any>('decryptMessages', data)
  },

  getPrunableMessage(transaction: string) {
    return nrcsGet<any>('getPrunableMessage', { transaction })
  },

  getPrunableMessages(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ prunableMessages: any[] }>('getPrunableMessages', { account, firstIndex, lastIndex })
  },

  getAllPrunableMessages(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ prunableMessages: any[] }>('getAllPrunableMessages', { firstIndex, lastIndex })
  },

  verifyPrunableMessage(transaction: string) {
    return nrcsGet<any>('verifyPrunableMessage', { transaction })
  },

  generateToken(data: { secretPhrase: string; website: string }) {
    return nrcsPost<NrcsToken>('generateToken', data)
  },

  /**
   * 解码并验证 token（对标 nrs.modals.token.js:39-46 decodeToken）。
   *
   * @param website 关联的网站/来源
   * @param token 待验证的 token 字符串
   */
  decodeToken(website: string, token: string) {
    return nrcsGet<any>('decodeToken', { website, token })
  },

  hash(data: { hashAlgorithm: number; secret: string; secretIsText: boolean }) {
    return nrcsGet<{ hash: string }>('hash', data)
  },

  rsConvert(account: string) {
    return nrcsGet<{ accountRS: string; account: string }>('rsConvert', { account })
  },

  longConvert(id: string) {
    return nrcsGet<{ stringId: string; longId: string }>('longConvert', { id })
  },

  getAccountLedger(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ entries: any[] }>('getAccountLedger', { account, firstIndex, lastIndex })
  },

  /**
   * 获取账户总账单条目详情（对标 nrs.modals.ledger.js:43 getAccountLedgerEntry）。
   *
   * 用于总账详情 modal 中展示单条 ledger entry。
   *
   * @param ledgerId 总账条目 ID
   */
  getAccountLedgerEntry(ledgerId: string) {
    return nrcsGet<any>('getAccountLedgerEntry', { ledgerId })
  },

  getScheduledTransactions(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ transactions: NrcsTransaction[] }>('getScheduledTransactions', { account, firstIndex, lastIndex })
  },

  /**
   * 获取资金监控列表（对标 nrs.monitors.js:86 getFundingMonitor）。
   *
   * @param options.account 查询指定账户的监控（管理员可查任意账户）
   * @param options.adminPassword 管理员密码（本地节点操作时需要）
   * @param options.secretPhrase 签名密钥（可选）
   * @param options.firstIndex 分页起始索引
   * @param options.lastIndex 分页结束索引
   */
  getFundingMonitor(options?: {
    account?: string
    adminPassword?: string
    secretPhrase?: string
    firstIndex?: number
    lastIndex?: number
  }) {
    return nrcsGet<{ monitors: any[] }>('getFundingMonitor', options)
  },

  /**
   * 启动资金监控（对标 nrs.monitors.js:115 startFundingMonitor）。
   *
   * @param data.secretPhrase 签名密钥
   * @param data.adminPassword 管理员密码（本地节点操作时需要）
   * @param data.property 监控属性名
   * @param data.amount 单次注资金额 NQT
   * @param data.threshold 触发阈值 NQT
   * @param data.interval 监控间隔（区块数）
   * @param data.feeNQT 手续费 NQT
   * @param data.deadline 截止时间（分钟）
   */
  startFundingMonitor(data: {
    secretPhrase: string
    adminPassword?: string
    property: string
    amount: string
    threshold?: string
    interval?: number
    feeNQT: string
    deadline: number
  }) {
    return nrcsPost<any>('startFundingMonitor', data)
  },

  /**
   * 停止资金监控（对标 nrs.monitors.js:135 stopFundingMonitor）。
   *
   * @param data.secretPhrase 签名密钥
   * @param data.adminPassword 管理员密码（本地节点操作时需要）
   * @param data.property 监控属性名
   * @param data.account 停止指定账户的监控（管理员可指定）
   * @param data.feeNQT 手续费 NQT
   * @param data.deadline 截止时间（分钟）
   */
  stopFundingMonitor(data: {
    secretPhrase: string
    adminPassword?: string
    property: string
    account?: string
    feeNQT: string
    deadline: number
  }) {
    return nrcsPost<any>('stopFundingMonitor', data)
  },

  getPlugins() {
    return nrcsGet<{ plugins: any[] }>('getPlugins')
  },

  getConstants() {
    return nrcsGet<any>('getConstants')
  },

  getAccountPhasedTransactions(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ transactions: NrcsTransaction[] }>('getAccountPhasedTransactions', { account, firstIndex, lastIndex })
  },

  /**
   * 获取 phased 交易的投票轮询状态（对标 nrs.transactions.js:278 getPhasingPoll）。
   * 返回 result（0=pending/1=approved/2=rejected）、yesVotes、noVotes 等。
   */
  getPhasingPoll(transaction: string, countVotes?: boolean) {
    return nrcsGet<any>('getPhasingPoll', { transaction, countVotes })
  },

  /**
   * 获取当前账户对指定 phased 交易的投票记录（对标 nrs.transactions.js:283 getPhasingPollVote）。
   */
  getPhasingPollVote(transaction: string, account: string) {
    return nrcsGet<any>('getPhasingPollVote', { transaction, account })
  },

  approveTransaction(data: { secretPhrase: string; transaction: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('approveTransaction', data)
  },

  setPhasingOnlyControl(data: {
    secretPhrase: string; controlVotingModel: number; controlQuorum: number;
    controlMinBalance: string; controlMinBalanceModel: number;
    controlWhitelisted?: string[]; controlMaxFees: string;
    controlMinDuration: number; controlMaxDuration: number; feeNQT: string; deadline: number;
  }) {
    return nrcsPost<any>('setPhasingOnlyControl', data)
  },

  signTransaction(unsignedTransactionBytes?: string, unsignedTransactionJSON?: string, secretPhrase?: string) {
    return nrcsPost<any>('signTransaction', { unsignedTransactionBytes, unsignedTransactionJSON, secretPhrase })
  },

  calculateFullHash(unsignedTransactionBytes?: string, unsignedTransactionJSON?: string, signatureHash?: string) {
    return nrcsGet<any>('calculateFullHash', { unsignedTransactionBytes, unsignedTransactionJSON, signatureHash })
  },

  blacklistPeer(peer: string) {
    return nrcsPost<any>('blacklistPeer', { peer })
  },

  deleteScheduledTransaction(transaction: string) {
    return nrcsPost<any>('deleteScheduledTransaction', { transaction })
  },

  markHost(secretPhrase: string, host: string, weight: number, date: string) {
    return nrcsPost<any>('markHost', { secretPhrase, host, weight, date })
  },

  decodeHallmark(hallmark: string) {
    return nrcsGet<any>('decodeHallmark', { hallmark })
  },

  getBlockId(block: string) {
    return nrcsGet<{ block: string }>('getBlockId', { block })
  },

  getECBlock(timestamp: number) {
    return nrcsGet<any>('getECBlock', { timestamp })
  },

  leaseBalance(secretPhrase: string, recipient: string, period: number, feeNQT: string, deadline: number) {
    return nrcsPost<any>('leaseBalance', { secretPhrase, recipient, period, feeNQT, deadline })
  },

  // Shuffling (混币)
  shufflingCreate(data: { secretPhrase: string; amount: string; participantCount: number; registrationPeriod: number; holdingType: number; holding: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('shufflingCreate', data)
  },

  shufflingRegister(data: { secretPhrase: string; shuffling: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('shufflingRegister', data)
  },

  shufflingProcess(data: { secretPhrase: string; shuffling: string; recipientSecretPhrase?: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('shufflingProcess', data)
  },

  shufflingVerify(data: { secretPhrase: string; shuffling: string; shufflingStateHash: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('shufflingVerify', data)
  },

  shufflingCancel(data: { secretPhrase: string; shuffling: string; cancellingAccount: string; shufflingStateHash: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('shufflingCancel', data)
  },

  getAllShufflings(firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ shufflings: any[] }>('getAllShufflings', { firstIndex, lastIndex })
  },

  getAccountShufflings(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ shufflings: any[] }>('getAccountShufflings', { account, firstIndex, lastIndex })
  },

  getShuffling(shuffling: string) {
    return nrcsGet<any>('getShuffling', { shuffling })
  },

  getShufflingParticipants(shuffling: string) {
    return nrcsGet<{ participants: any[] }>('getShufflingParticipants', { shuffling })
  },

  getPollResult(poll: string, votingModel?: number, holding?: string, minBalance?: string, minBalanceModel?: number) {
    return nrcsGet<{ poll: string; results: { option: string; weight: string; result: string }[] }>('getPollResult', { poll, votingModel, holding, minBalance, minBalanceModel })
  },

  getPollVotes(poll: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ votes: { voter: string; voterRS: string; votes: string[]; transaction: string; timestamp: number }[] }>('getPollVotes', { poll, firstIndex, lastIndex })
  },

  getPollVote(poll: string, account: string) {
    return nrcsGet<{ voter: string; voterRS: string; votes: string[] }>('getPollVote', { poll, account })
  },

  getDGSTags(inStockOnly?: boolean, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ tags: string[] }>('getDGSTags', { inStockOnly, firstIndex, lastIndex })
  },

  getDGSTagCount(inStockOnly?: boolean) {
    return nrcsGet<{ numberOfTags: number }>('getDGSTagCount', { inStockOnly })
  },

  getDGSExpiredPurchases(seller?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ purchases: NrcsDGSPurchase[] }>('getDGSExpiredPurchases', { seller, firstIndex, lastIndex })
  },

  getShufflers(account?: string, adminPassword?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ shufflers: { account: string; accountRS: string; shuffling: string }[] }>('getShufflers', { account, adminPassword, firstIndex, lastIndex })
  },

  startShuffler(data: { secretPhrase: string; shufflingFullHash: string; recipientPublicKey?: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('startShuffler', data)
  },

  stopShuffler(data: { secretPhrase: string; shufflingFullHash: string; processing?: boolean; adminPassword?: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('stopShuffler', data)
  }
}
