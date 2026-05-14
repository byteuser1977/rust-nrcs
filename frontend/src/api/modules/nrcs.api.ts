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
  getAccount(account: string, includeLessors?: boolean) {
    return nrcsGet<NrcsAccount>('getAccount', { account, includeLessors })
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

  getEffectiveBalance(account: string) {
    return nrcsGet<{ effectiveBalanceNRCS: number }>('getEffectiveBalance', { account })
  },

  setAccountInfo(data: { secretPhrase: string; name: string; description: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('setAccountInfo', data)
  },

  setAccountProperty(data: { secretPhrase: string; recipient: string; property: string; value: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('setAccountProperty', data)
  },

  getAccountProperties(account: string) {
    return nrcsGet<{ properties: any[] }>('getAccountProperties', { recipient: account })
  },

  searchAccounts(query: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ accounts: any[] }>('searchAccounts', { query, firstIndex, lastIndex })
  },

  getBlockchainTransactions(account: string, firstIndex?: number, lastIndex?: number, type?: number, subtype?: number) {
    return nrcsGet<{ transactions: NrcsTransaction[] }>('getBlockchainTransactions', {
      account, firstIndex, lastIndex, type, subtype
    })
  },

  getUnconfirmedTransactions(account?: string) {
    return nrcsGet<{ unconfirmedTransactions: NrcsUnconfirmedTransaction[] }>('getUnconfirmedTransactions', { account })
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

  broadcastTransaction(transactionBytes: string) {
    return nrcsPost<any>('broadcastTransaction', { transactionBytes })
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

  getExchanges(currency?: string, account?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ exchanges: any[] }>('getExchanges', { currency, account, firstIndex, lastIndex })
  },

  getCurrencyTransfers(currency?: string, account?: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ transfers: any[] }>('getCurrencyTransfers', { currency, account, firstIndex, lastIndex })
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

  getDGSPurchase(purchase: string) {
    return nrcsGet<NrcsDGSPurchase>('getDGSPurchase', { purchase })
  },

  getDGSPurchases(buyer?: string, seller?: string, firstIndex?: number, lastIndex?: number, completed?: boolean) {
    return nrcsGet<{ purchases: NrcsDGSPurchase[] }>('getDGSPurchases', { buyer, seller, firstIndex, lastIndex, completed })
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

  decodeToken(token: string) {
    return nrcsGet<any>('decodeToken', { token })
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

  getScheduledTransactions(account: string, firstIndex?: number, lastIndex?: number) {
    return nrcsGet<{ transactions: NrcsTransaction[] }>('getScheduledTransactions', { account, firstIndex, lastIndex })
  },

  getFundingMonitor(secretPhrase?: string) {
    return nrcsGet<{ monitors: any[] }>('getFundingMonitor', { secretPhrase })
  },

  startFundingMonitor(data: { secretPhrase: string; property: string; amount: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('startFundingMonitor', data)
  },

  stopFundingMonitor(data: { secretPhrase: string; property: string; feeNQT: string; deadline: number }) {
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

  approveTransaction(data: { secretPhrase: string; transaction: string; feeNQT: string; deadline: number }) {
    return nrcsPost<any>('approveTransaction', data)
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
