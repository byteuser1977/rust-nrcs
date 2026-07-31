/**
 * NRCS Blockchain Types
 *
 * Centralized type definitions for all NRCS blockchain-specific entities.
 * These types correspond to the NRCS API response structures.
 */

// ============================================================================
// API Response Wrappers
// (ApiResponse and PageResult are defined in ./api.ts;
//  use the aliases below for NRCS-specific contexts.)
// ============================================================================

// Re-export with NRCS-specific aliases
export type {
  ApiResponse as NrcsApiResponse,
  PageResult as NrcsPageResult
} from './api'

// ============================================================================
// Account
// ============================================================================

/**
 * NRCS blockchain account.
 */
export interface NrcsAccount {
  /** Numeric account ID */
  account: string
  /** RS-encoded address (Reed-Solomon format) */
  accountRS: string
  /** Account public key (hex) */
  publicKey: string
  /** Balance in NQT (1 NRC = 10^8 NQT) */
  balanceNQT: string
  /** Unconfirmed balance in NQT */
  unconfirmedBalanceNQT: string
  /** Effective (forging) balance in NQT */
  effectiveBalanceNQT?: string
  /** Effective balance in NRC display units */
  effectiveBalanceNRCS?: number
  /** Number of blocks forged by this account */
  forgedBalanceNQT: string
  /** Guaranteed balance in NQT */
  guaranteedBalanceNQT?: string
  /** Account name (if set) */
  name?: string
  /** Account description (if set) */
  description?: string
  /** Current leasing height-from, or 0 if not leasing */
  currentLessee?: string
  /** Next lessee account ID (RS format) */
  nextLessee?: string
  /** Leasing height-from */
  currentLeasingHeightFrom?: number
  /** Leasing height-to */
  currentLeasingHeightTo?: number
  /** Next leasing height-from */
  nextLeasingHeightFrom?: number
  /** Next leasing height-to */
  nextLeasingHeightTo?: number
}

// ============================================================================
// Transaction
// ============================================================================

/**
 * Base NRCS transaction structure.
 *
 * Uses an `attachment` discriminator whose shape varies by type/subtype.
 */
export interface NrcsTransaction {
  /** Transaction full hash (hex) */
  transaction: string
  /** Transaction ID */
  transactionId?: number
  /** Transaction type (0-12 or negative for parent chain) */
  type: number
  /** Transaction subtype */
  subtype: number
  /** Block height where this transaction was included */
  height: number
  /** Block timestamp (epoch seconds from NRCS genesis) */
  timestamp: number
  /** Sender account ID */
  sender: string
  /** Sender RS-encoded address */
  senderRS: string
  /** Recipient account ID */
  recipient?: string
  /** Recipient RS-encoded address */
  recipientRS?: string
  /** Amount in NQT */
  amountNQT: string
  /** Fee in NQT */
  feeNQT: string
  /** Transaction sender public key (hex) */
  senderPublicKey?: string
  /** Block ID */
  block: string
  /** Block height */
  blockTimestamp?: number
  /** Transaction deadline (minutes) */
  deadline: number
  /** Transaction signature (hex) */
  signature?: string
  /** Full hash of referenced transaction (hex) */
  referencedTransactionFullHash?: string
  /** Transaction ecBlockHeight */
  ecBlockHeight?: number
  /** Transaction ecBlockId */
  ecBlockId?: string
  /** Number of confirmations */
  confirmations: number
  /** Transaction index within block */
  transactionIndex?: number
  /** Whether phased (conditional execution) */
  phased?: boolean
  /** Phasing completion status (true = completed, false = pending, undefined = not phased) */
  expectedCancellation?: boolean
  /** Attachment data (type-specific payloads) */
  attachment?: Record<string, unknown>
  /** Whether this is a child chain transaction */
  isChildChain?: boolean
}

// ============================================================================
// Block
// ============================================================================

/**
 * NRCS blockchain block.
 */
export interface NrcsBlock {
  /** Block ID (full hash) */
  block: string
  /** Block height */
  height: number
  /** Block timestamp (epoch seconds from NRCS genesis) */
  timestamp: number
  /** Generator account ID */
  generator: string
  /** Generator RS-encoded address */
  generatorRS: string
  /** Generator public key (hex) */
  generatorPublicKey?: string
  /** Base target value for forging difficulty */
  baseTarget: string
  /** Cumulative difficulty */
  cumulativeDifficulty: string
  /** Previous block hash */
  previousBlockHash: string
  /** Next block hash (if available) */
  nextBlockHash?: string
  /** Block payload hash */
  payloadHash: string
  /** Generation signature */
  generationSignature: string
  /** Block signature */
  blockSignature: string
  /** Total amount in NQT for all transactions in block */
  totalAmountNQT: string
  /** Total fee in NQT for all transactions in block */
  totalFeeNQT: string
  /** Payload length in bytes */
  payloadLength: number
  /** Number of transactions in block */
  numberOfTransactions: number
  /** Version of the block */
  version?: number
  /** Previous block height */
  previousBlock?: string
}

// ============================================================================
// Peer
// ============================================================================

/**
 * NRCS network peer node.
 */
export interface NrcsPeer {
  /** Peer network address (IP:port or hostname) */
  address: string
  /** Announced address */
  announcedAddress?: string
  /** Peer software version */
  version: string
  /** Platform description */
  platform: string
  /** Application name */
  application?: string
  /** API port */
  apiPort?: number
  /** API SSL port */
  apiSSLPort?: number
  /** Current blockchain height */
  height: number
  /** State of the peer (CONNECTED, NON_CONNECTED, DISCONNECTED) */
  state: number
  /** Whether the peer provides hallmark-protected services */
  isNewlyConnected?: boolean
  /** Downloaded volume from this peer (bytes) */
  downloadedVolume?: number
  /** Uploaded volume to this peer (bytes) */
  uploadedVolume?: number
  /** Last update timestamp */
  lastUpdated?: number
  /** Time since last update (epoch seconds) */
  lastConnectTime?: number
  /** Weight assigned to this peer */
  weight?: number
  /** Blacklisted status */
  blacklisted?: boolean
  /** Type of service: HALLMARK, NORMAL */
  serviceType?: string
  /** Hallmark info (if peer is hallmark-protected) */
  hallmark?: string
  /** Shared address indicator */
  shareAddress?: boolean
  /** Block chain state (UP_TO_DATE, DOWNLOADING, etc.) */
  blockState?: string
}

// ============================================================================
// Blockchain Status
// ============================================================================

/**
 * NRCS current blockchain status.
 */
export interface NrcsBlockchainStatus {
  /** Application name */
  application: string
  /** Application version */
  version: string
  /** Current blockchain height */
  numberOfBlocks: number
  /** Timestamp of the last block (epoch seconds) */
  time: number
  /** Last block ID */
  lastBlock: string
  /** Account RS-encoded address of the last block generator */
  lastBlockGenerator?: string
  /** Cumulative difficulty */
  cumulativeDifficulty: string
  /** Average block generation time (seconds) */
  averageBlockGenerationTime?: number
  /** Total effective balance of all forging accounts */
  totalEffectiveBalance?: string
  /** Total number of accounts */
  numberOfAccounts?: number
  /** Number of child (non-parent) transactions */
  numberOfTransactions?: number
  /** Number of connected peers */
  numberOfPeers?: number
  /** Number of currently unlocked accounts */
  numberOfUnlockedAccounts?: number
  /** Number of available processors */
  numberOfProcessors?: number
  /** Maximum available memory (bytes) */
  maxMemory?: number
  /** Memory currently in use (bytes) */
  freeMemory?: number
  /** Total memory (bytes) */
  totalMemory?: number
  /** Whether the node is currently scanning the blockchain (对标 NRS.state.isScanning) */
  isScanning?: boolean
  /** Whether the node is running as a light client (对标 NRS.state.isLightClient) */
  isLightClient?: boolean
  /** Whether the node is acting as an API proxy (对标 NRS.state.apiProxy) */
  apiProxy?: boolean
  /** Whether the node is in testnet mode (对标 NRS.state.isTestnet) */
  isTestnet?: boolean
  /** Name of the last blockchain feeder peer (对标 NRS.state.lastBlockchainFeeder) */
  lastBlockchainFeeder?: string
  /** Height of the last blockchain feeder (对标 NRS.state.lastBlockchainFeederHeight) */
  lastBlockchainFeederHeight?: number
  /** Number of blocks to trim from ledger (对标 NRS.state.ledgerTrimKeep) */
  ledgerTrimKeep?: number
  /** Max number of transactions (对标 NRS.state.maxTransactions) */
  maxTransactions?: number
}

// ============================================================================
// Alias
// ============================================================================

/**
 * NRCS alias (human-readable name mapped to URI/account).
 */
export interface NrcsAlias {
  /** Alias name (lowercase) */
  aliasName: string
  /** Associated URI or data */
  aliasURI: string
  /** Numeric account ID that owns the alias */
  account: string
  /** RS-encoded owner address */
  accountRS?: string
  /** Alias ID (numeric) */
  aliasId?: string
  /** NQT price if offered for sale, 0 otherwise */
  priceNQT: string
  /** Buyer account ID if alias has a pending sale buyer */
  buyer?: string
  /** Buyer RS address */
  buyerRS?: string
  /** Block height when alias was registered */
  timestamp?: number
  /** Transaction ID that registered this alias */
  transactionId?: string
}

// ============================================================================
// Asset
// ============================================================================

/**
 * NRCS Asset (user-issued token).
 */
export interface NrcsAsset {
  /** Asset ID (numeric) */
  asset: string
  /** Issuer account ID */
  account: string
  /** Issuer RS-encoded address */
  accountRS: string
  /** Asset name (3-10 uppercase letters) */
  name: string
  /** Asset description */
  description: string
  /** Total quantity in QNT (base units) */
  quantityQNT: string
  /** Number of asset decimal places */
  decimals: number
  /** Number of accounts holding this asset */
  numberOfAccounts?: number
  /** Number of trades */
  numberOfTrades?: number
  /** Number of open ask orders */
  numberOfAsks?: number
  /** Number of open bid orders */
  numberOfBids?: number
  /** Height when asset was issued */
  height?: number
}

/**
 * NRCS account's balance for a specific asset.
 */
export interface NrcsAssetBalance {
  /** Asset ID */
  asset: string
  /** Account RS address */
  accountRS: string
  /** Account ID (numeric) */
  account: string
  /** Confirmed quantity in QNT */
  quantityQNT: string
  /** Unconfirmed quantity in QNT */
  unconfirmedQuantityQNT: string
  /** Asset name */
  name?: string
  /** Asset decimals */
  decimals?: number
}

/**
 * NRCS asset order (ask/bid on the asset exchange).
 */
export interface NrcsAssetOrder {
  /** Order ID */
  order: string
  /** Asset ID */
  asset: string
  /** Account RS address */
  accountRS: string
  /** Account ID (numeric) */
  account: string
  /** Quantity in QNT */
  quantityQNT: string
  /** Price in NQT per QNT */
  priceNQT: string
  /** Order type: "ask" or "bid" */
  type: 'ask' | 'bid'
  /** Order height */
  height: number
  /** Transaction ID */
  transactionId?: string
  /** Transaction index */
  transactionIndex?: number
  /** Order status: OPEN, FILLED, CANCELLED */
  status?: 'OPEN' | 'FILLED' | 'CANCELLED'
}

// ============================================================================
// Currency (Monetary System)
// ============================================================================

/**
 * NRCS Monetary System currency (MS coin).
 */
export interface NrcsCurrency {
  /** Currency ID (numeric) */
  currency: string
  /** Issuer account ID */
  account: string
  /** Issuer RS-encoded address */
  accountRS: string
  /** Currency code (3-5 uppercase letters) */
  code: string
  /** Currency name (3-10 characters) */
  name: string
  /** Currency description */
  description?: string
  /** Currency type: EXCHANGEABLE, CONTROLLABLE, RESERVABLE, CLAIMABLE, MINTABLE, NON_SHUFFLEABLE */
  type?: number
  /** Maximum supply in QNT */
  maxSupplyQNT: string
  /** Reserve supply (units) in QNT */
  reserveSupplyQNT: string
  /** Current supply in QNT */
  currentSupplyQNT: string
  /** Number of decimal places for this currency */
  decimals: number
  /** Algorithm for maintaining supply */
  algorithm?: number
  /** Number of accounts holding this currency */
  numberOfOwners?: number
  /** Number of trades */
  numberOfTrades?: number
  /** Height when currency was issued */
  issuanceHeight?: number
  /** Reserve NQT per unit */
  reserveNQTPerUnit?: string
  /** Minimum reserve NQT */
  minReservePerUnitNQT?: string
  /** Minting difficulty */
  minDifficulty?: number
  /** Maximum difficulty */
  maxDifficulty?: number
  /** Current minting difficulty */
  currentDifficulty?: number
  /** Whether deleted */
  deleted?: boolean
}

// ============================================================================
// Poll / Voting
// ============================================================================

/**
 * NRCS Poll (voting).
 */
export interface NrcsPoll {
  /** Poll ID (numeric) */
  poll: string
  /** Creator account ID */
  account: string
  /** Creator RS-encoded address */
  accountRS: string
  /** Poll name/title */
  name: string
  /** Poll description */
  description?: string
  /** Height at which the poll finishes */
  finishHeight: number
  /** Whether the poll has finished */
  finished: boolean
  /** Voting model (0 = one account one vote, etc.) */
  votingModel?: number
  /** Minimum number of options selectable */
  minNumberOfOptions?: number
  /** Maximum number of options selectable */
  maxNumberOfOptions?: number
  /** Minimum balance required to vote */
  minBalance?: string
  /** Minimum balance model */
  minBalanceModel?: number
  /** Holding ID (asset/currency) for weighted voting */
  holding?: string
  /** Array of option strings */
  options: string[]
  /** Raw options as received from server */
  rawOptions?: string[][]
  /** Transaction ID that created this poll */
  transactionId?: string
}

/**
 * NRCS poll result (vote tally for a single option).
 */
export interface NrcsPollResult {
  /** Option index or string representation */
  option: string
  /** Total weight of votes for this option */
  weight: string
  /** Number of accounts that voted for this option */
  numberOfAccounts?: number
  /** Total number of NRC votes (when weighted) */
  total?: string
  /** Option name */
  name?: string
}

// ============================================================================
// DGS (Digital Goods Store / Marketplace)
// ============================================================================

/**
 * NRCS Marketplace product listing (DGS good).
 */
export interface NrcsDGSProduct {
  /** Product ID (numeric) */
  goods: string
  /** Product name */
  name: string
  /** Product description */
  description: string
  /** Product tags */
  tags: string
  /** Seller account ID */
  seller: string
  /** Seller RS-encoded address */
  sellerRS: string
  /** Price in NQT per unit */
  priceNQT: string
  /** Available quantity */
  quantity: number
  /** Product listed state (true = delisted) */
  delisted?: boolean
  /** Block height when first listed */
  timestamp?: number
  /** Whether the product has any image data */
  hasImage?: boolean
  /** Encrypted goods data (if applicable) */
  goodsData?: Record<string, unknown>
  /** In-stock indicator derived from quantity */
  inStock?: string
}

/**
 * NRCS Marketplace purchase.
 */
export interface NrcsDGSPurchase {
  /** Purchase ID (numeric) */
  purchase: string
  /** Product ID being purchased */
  goods: string
  /** Product name (resolved from goods) */
  name?: string
  /** Buyer account ID */
  buyer: string
  /** Buyer RS-encoded address */
  buyerRS?: string
  /** Seller account ID */
  seller: string
  /** Seller RS-encoded address */
  sellerRS?: string
  /** Purchase quantity */
  quantity: number
  /** Price in NQT per unit */
  priceNQT: string
  /** Purchase delivery deadline timestamp (epoch seconds) */
  deliveryDeadlineTimestamp: number
  /** Encrypted goods data */
  goodsData?: Record<string, unknown>
  /** Encrypted goods nonce */
  goodsNonce?: string
  /** Whether goods have been delivered */
  pending?: boolean
  /** Whether goods are encrypted */
  goodsIsText?: boolean
  /** Refund amount in NQT (if any) */
  refundNQT?: string
  /** Discount amount in NQT (if any) */
  discountNQT?: string
  /** Feedback note */
  note?: string
  /** Whether buyer has submitted feedback */
  buyerFeedback?: string
  /** Whether seller has submitted feedback */
  sellerFeedback?: string
  /** Whether refund was issued */
  hasRefund?: boolean
}

// ============================================================================
// Messages
// ============================================================================

/**
 * NRCS arbitrary message / encrypted message.
 */
export interface NrcsMessage {
  /** Enclosing transaction ID */
  transaction: string
  /** Sender account ID */
  sender: string
  /** Sender RS-encoded address */
  senderRS: string
  /** Recipient account ID (may be empty for public messages) */
  recipient?: string
  /** Recipient RS-encoded address */
  recipientRS?: string
  /** Whether message is encrypted */
  isText?: boolean
  /** Whether message is an encrypted-to-self note */
  isEncryptedToSelf?: boolean
  /** The message content (decrypted if applicable) */
  message?: string
  /** Raw message attachment data */
  attachment?: Record<string, unknown>
  /** Block timestamp (epoch seconds) */
  timestamp?: number
  /** Block height */
  height?: number
}

// ============================================================================
// Shuffling
// ============================================================================

/**
 * NRCS Coin Shuffling instance.
 */
export interface NrcsShuffling {
  /** Shuffling ID (numeric) */
  shuffling: string
  /** Current stage (0-4) */
  stage: number
  /** Issuer account ID */
  issuer?: string
  /** Issuer RS address */
  issuerRS?: string
  /** Holding type (0 = NRC, asset, currency) */
  holdingType?: number
  /** Holding ID (if not NRC) */
  holding?: string
  /** Amount being shuffled (in NQT or QNT) */
  amount: string
  /** Number of required participants */
  participantCount: number
  /** Number of registered participants */
  registrantCount: number
  /** Shuffling state: ACTIVE, BLAME, CANCELLED, DONE */
  shufflingState?: number
  /** Full hash of shuffling creation transaction */
  shufflingFullHash?: string
  /** Assigned recipient account */
  assignee?: string
  /** Recipient public key */
  recipientPublicKey?: string
  /** Blame data (if stage is BLAME) */
  blame?: Record<string, unknown>
  /** Linked full hash */
  linkedFullHash?: string
}

// ============================================================================
// Tagged Data
// ============================================================================

/**
 * NRCS tagged data (on-chain data storage).
 */
export interface NrcsTaggedData {
  /** Enclosing transaction ID */
  transaction: string
  /** Data name */
  name: string
  /** Data description */
  description?: string
  /** Data tags */
  tags?: string
  /** Data type (raw or structured) */
  type?: string
  /** Channel (for filtering) */
  channel?: string
  /** Whether this data is text */
  isText?: boolean
  /** Filename (if applicable) */
  filename?: string
  /** Parsed/formatted data */
  data?: string
  /** Raw hex data */
  rawData?: string
  /** Hash of the tagged data */
  hash?: string
  /** Account that uploaded the data */
  account?: string
  /** Account RS address */
  accountRS?: string
  /** Block timestamp (epoch seconds) */
  timestamp?: number
  /** Block height */
  height?: number
  /** Transaction timestamp */
  transactionTimestamp?: number
}

// ============================================================================
// Monetary System - Exchange
// ============================================================================

/**
 * NRCS currency exchange offer.
 */
export interface NrcsExchangeOffer {
  /** Offer ID */
  offer: string
  /** Currency ID */
  currency: string
  /** Offer publisher account ID */
  account: string
  /** Offer publisher RS address */
  accountRS: string
  /** Buy rate in NQT per unit */
  buyRate: string
  /** Sell rate in NQT per unit */
  sellRate: string
  /** Initial buy supply (units) */
  initialBuySupply: string
  /** Initial sell supply (units) */
  initialSellSupply: string
  /** Total buy limit (units) */
  buyLimit: string
  /** Total sell limit (units) */
  sellLimit: string
  /** Current buy supply (units) */
  supply: string
  /** Offer expiration height */
  expirationHeight?: number
  /** Height the offer was placed */
  height: number
  /** Available units for buy */
  buySupply?: string
  /** Available units for sell */
  sellSupply?: string
}

/**
 * NRCS currency exchange transaction.
 */
export interface NrcsExchange {
  /** Exchange ID */
  exchange: string
  /** Currency ID */
  currency: string
  /** Offer ID */
  offer?: string
  /** Buyer account ID */
  buyer: string
  /** Buyer RS address */
  buyerRS?: string
  /** Seller account ID */
  seller: string
  /** Seller RS address */
  sellerRS?: string
  /** Units exchanged */
  units: string
  /** Exchange rate in NQT per unit */
  rate: string
  /** Block height */
  height: number
  /** Transaction ID */
  transactionId?: string
  /** Timestamp (epoch seconds) */
  timestamp?: number
}

// ============================================================================
// Forging / Generator
// ============================================================================

/**
 * NRCS block generator (forging) info.
 */
export interface NrcsGenerator {
  /** Account ID (numeric) */
  account: string
  /** Account RS address */
  accountRS: string
  /** Account effective balance in NQT */
  effectiveBalanceNXT: string
  /** Estimated deadline (seconds) until next block */
  deadline: number
  /** Estimated hit time (epoch seconds) */
  hitTime: number
  /** Secret sharing piece (for clustering) */
  secretSharingPiece?: number[]
}

// ============================================================================
// Funding Monitor
// ============================================================================

/**
 * NRCS funding monitor (watches for incoming transactions matching criteria).
 */
export interface NrcsFundingMonitor {
  /** Monitor holder account ID */
  account: string
  /** Monitor holder RS address */
  accountRS: string
  /** Monitored property name */
  property: string
  /** Threshold amount */
  amount: string
  /** Threshold comparison: >, <, >=, <=, ==, != */
  threshold: string
  /** Interval (seconds between checks) */
  interval: number
  /** Whether to include holding type (for asset/currency) */
  holdingType?: number
  /** Holding ID (for asset/currency) */
  holding?: string
  /** Monitor state: ON, OFF */
  monitorState?: 'ON' | 'OFF'
}

// ============================================================================
// Plugin
// ============================================================================

/**
 * NRCS server plugin info.
 */
export interface NrcsPlugin {
  /** Plugin name */
  name: string
  /** Plugin version */
  version: string
  /** Current status: active, deactivated, error */
  status: 'active' | 'deactivated' | 'error'
  /** Plugin description */
  description?: string
  /** Plugin website URL */
  website?: string
  /** Whether started automatically */
  autostart?: boolean
  /** Error message if status is "error" */
  errorDescription?: string
}

// ============================================================================
// Ledger Entry
// ============================================================================

/**
 * NRCS ledger entry (account balance change log).
 */
export interface NrcsLedgerEntry {
  /** Account ID */
  account: string
  /** Account RS address */
  accountRS: string
  /** Ledger entry ID */
  ledgerId?: string
  /** Block height */
  height: number
  /** Entry timestamp (epoch seconds) */
  timestamp: number
  /** Description of the event */
  event: string
  /** Type of the event */
  eventType: string
  /** Is this a transaction event */
  isTransactionEvent: boolean
  /** Change amount in NQT (can be negative) */
  change: string
  /** New balance after this entry in NQT */
  balance: string
  /** Holding type for non-NRC entries */
  holdingType?: number
  /** Holding ID for non-NRC entries */
  holding?: string
  /** Transaction ID (if event is a transaction) */
  transactionId?: string
}

// ============================================================================
// Account Properties
// ============================================================================

/**
 * NRCS account property (key-value metadata set on an account).
 */
export interface NrcsAccountProperty {
  /** Property setter account ID */
  setter: string
  /** Property setter RS address */
  setterRS: string
  /** Recipient account ID */
  recipient: string
  /** Recipient RS address */
  recipientRS: string
  /** Property key name */
  property: string
  /** Property value */
  value: string
  /** Transaction that set this property */
  transactionId?: string
  /** Block height */
  height?: number
}

// ============================================================================
// Scheduled Transaction
// ============================================================================

/**
 * NRCS scheduled transaction (phased transaction awaiting approval/trigger).
 */
export interface NrcsScheduledTransaction {
  /** Transaction ID */
  transaction: string
  /** Transaction sender account */
  sender: string
  /** Sender RS address */
  senderRS: string
  /** Transaction type */
  type: number
  /** Transaction subtype */
  subtype: number
  /** Whether phased execution is enabled */
  phased: boolean
  /** Whether execution will be cancelled */
  expectedCancellation?: boolean
  /** Voted transaction ID */
  votesTransaction?: string
  /** Phasing voting model */
  votingModel?: number
  /** Phasing quorum */
  quorum?: string
  /** Phasing minimum balance (if applicable) */
  minBalance?: string
  /** Phasing minimum balance model */
  minBalanceModel?: number
  /** Holding ID for voting weight */
  holding?: string
  /** Whitelisted accounts */
  whitelist?: string[]
  /** Linked full hash of the phased transaction */
  linkedFullHash?: string
  /** Number of confirmations */
  confirmations?: number
  /** Amount in NQT */
  amountNQT?: string
  /** Fee in NQT */
  feeNQT?: string
  /** Block timestamp (epoch seconds) */
  timestamp?: number
  /** Deadline (minutes) */
  deadline?: number
}

// (ApiResponse and PageResult aliases are declared at the top of this file.)
