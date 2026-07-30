/**
 * NRCS Transaction Type Registry
 *
 * Ported from nrs.transactions.types.js
 * Maps all NRCS blockchain transaction types and subtypes to their
 * display metadata (title, icon, i18n key, receiver page).
 */

import type { ServerConstantsResponse, TransactionSubTypeServerInfo } from './server-constants'

// ============================================================================
// Interfaces
// ============================================================================

/** Metadata for a single transaction subtype */
export interface TransactionSubType {
  /** Display title */
  title: string
  /** i18n key for the title */
  i18nKeyTitle: string
  /** HTML icon markup */
  iconHTML?: string
  /** Target page name for linking (e.g., 'transactions', 'aliases') */
  receiverPage?: string
  /**
   * 服务端常量描述（由 `loadTransactionTypeConstants` 从 `getConstants` 响应注入）。
   * 包含 isPhasable / mustHaveRecipient / canHaveRecipient / isPhasingSafe 等运行时属性。
   * 加载前为 undefined。
   */
  serverConstants?: TransactionSubTypeServerInfo
}

/** Metadata for a transaction type (group of subtypes) */
export interface TransactionTypeDef {
  /** Display title */
  title: string
  /** i18n key for the title */
  i18nKeyTitle: string
  /** HTML icon markup */
  iconHTML?: string
  /** Chain domain: parent or child */
  chainType: 'parent' | 'child'
  /** Map of subtype id → subtype metadata */
  subTypes: Record<number, TransactionSubType>
}

// ============================================================================
// Complete Transaction Type Registry
// ============================================================================

/**
 * Complete map of all NRCS transaction types.
 *
 * Keys:
 *   -4 to -1: "parent chain" types (Coin Exchange, Account Control, Payment, Child Chain Block)
 *   0 to 12:  "child chain" types (Payment, Messaging, Asset Exchange, Marketplace, etc.)
 */
export const TRANSACTION_TYPES: Record<number, TransactionTypeDef> = {
  // ---- Parent Chain Types ----
  '-4': {
    title: 'Coin Exchange',
    i18nKeyTitle: 'coin_exchange',
    iconHTML: "<i class='fa fa-exchange-alt'></i>",
    chainType: 'parent',
    subTypes: {
      0: {
        title: 'Issue Order',
        i18nKeyTitle: 'issue_order',
        iconHTML: "<i class='fa fa-money-bill-alt'></i>",
        receiverPage: 'open_coin_orders'
      },
      1: {
        title: 'Cancel Order',
        i18nKeyTitle: 'cancel_order',
        iconHTML: "<i class='fa fa-times'></i>",
        receiverPage: 'open_coin_orders'
      }
    }
  },

  '-3': {
    title: 'Account Control',
    i18nKeyTitle: 'account_control',
    iconHTML: '<i class="ion-locked"></i>',
    chainType: 'parent',
    subTypes: {
      0: {
        title: 'Balance Leasing',
        i18nKeyTitle: 'balance_leasing',
        iconHTML: '<i class="fa fa-arrow-circle-o-right"></i>',
        receiverPage: 'transactions'
      }
    }
  },

  '-2': {
    title: 'Payment',
    i18nKeyTitle: 'payment',
    iconHTML: "<i class='ion-calculator'></i>",
    chainType: 'parent',
    subTypes: {
      0: {
        title: 'Ordinary Payment',
        i18nKeyTitle: 'ordinary_payment',
        iconHTML: "<i class='fa fa-money-bill-alt'></i>",
        receiverPage: 'transactions'
      }
    }
  },

  '-1': {
    title: 'Child Chain Block',
    i18nKeyTitle: 'child_chain_block',
    iconHTML: "<i class='fa fa-crop'></i>",
    chainType: 'parent',
    subTypes: {
      0: {
        title: 'Child Chain Block',
        i18nKeyTitle: 'child_chain_block',
        iconHTML: "<i class='fa fa-crop'></i>",
        receiverPage: 'transactions'
      }
    }
  },

  // ---- Child Chain Types ----

  // Type 0: Payment
  0: {
    title: 'Payment',
    i18nKeyTitle: 'payment',
    iconHTML: "<i class='ion-calculator'></i>",
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Ordinary Payment',
        i18nKeyTitle: 'ordinary_payment',
        iconHTML: "<i class='fa fa-money'></i>",
        receiverPage: 'transactions'
      }
    }
  },

  // Type 1: Messaging / Voting / Aliases
  1: {
    title: 'Messaging',
    i18nKeyTitle: 'messaging_voting_aliases',
    iconHTML: "<i class='fa fa-envelope-square'></i>",
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Arbitrary Message',
        i18nKeyTitle: 'arbitrary_message',
        iconHTML: "<i class='fa fa-envelope-o'></i>",
        receiverPage: 'messages'
      },
      1: {
        title: 'Alias Assignment',
        i18nKeyTitle: 'alias_assignment',
        iconHTML: "<i class='fa fa-bookmark'></i>"
      },
      2: {
        title: 'Poll Creation',
        i18nKeyTitle: 'poll_creation',
        iconHTML: "<i class='fa fa-check-square-o'></i>"
      },
      3: {
        title: 'Vote Casting',
        i18nKeyTitle: 'vote_casting',
        iconHTML: "<i class='fa fa-check'></i>"
      },
      4: {
        title: 'Hub Announcement',
        i18nKeyTitle: 'hub_announcement',
        iconHTML: "<i class='ion-radio-waves'></i>"
      },
      5: {
        title: 'Account Info',
        i18nKeyTitle: 'account_info',
        iconHTML: "<i class='fa fa-info'></i>"
      },
      6: {
        title: 'Alias Sale',
        i18nKeyTitle: 'alias_sale_transfer',
        iconHTML: "<i class='fa fa-tag'></i>",
        receiverPage: 'aliases'
      },
      7: {
        title: 'Alias Buy',
        i18nKeyTitle: 'alias_buy',
        iconHTML: "<i class='fa fa-money'></i>",
        receiverPage: 'aliases'
      },
      8: {
        title: 'Alias Deletion',
        i18nKeyTitle: 'alias_deletion',
        iconHTML: "<i class='fa fa-times'></i>"
      },
      9: {
        title: 'Transaction Approval',
        i18nKeyTitle: 'transaction_approval',
        iconHTML: "<i class='fa fa-gavel'></i>",
        receiverPage: 'transactions'
      },
      10: {
        title: 'Account Property',
        i18nKeyTitle: 'account_property',
        iconHTML: "<i class='fa fa-gavel'></i>",
        receiverPage: 'transactions'
      },
      11: {
        title: 'Account Property Delete',
        i18nKeyTitle: 'account_property_delete',
        iconHTML: "<i class='fa fa-question'></i>",
        receiverPage: 'transactions'
      },
      12: {
        title: 'Set Account Long Property',
        i18nKeyTitle: 'account_property',
        iconHTML: "<i class='fa fa-gavel'></i>",
        receiverPage: 'transactions'
      }
    }
  },

  // Type 2: Asset Exchange
  2: {
    title: 'Asset Exchange',
    i18nKeyTitle: 'asset_exchange',
    iconHTML: '<i class="fa fa-signal"></i>',
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Asset Issuance',
        i18nKeyTitle: 'asset_issuance',
        iconHTML: '<i class="fa fa-bullhorn"></i>'
      },
      1: {
        title: 'Asset Transfer',
        i18nKeyTitle: 'asset_transfer',
        iconHTML: '<i class="ion-arrow-swap"></i>',
        receiverPage: 'transfer_history'
      },
      2: {
        title: 'Ask Order Placement',
        i18nKeyTitle: 'ask_order_placement',
        iconHTML: '<i class="ion-arrow-graph-down-right"></i>',
        receiverPage: 'open_orders'
      },
      3: {
        title: 'Bid Order Placement',
        i18nKeyTitle: 'bid_order_placement',
        iconHTML: '<i class="ion-arrow-graph-up-right"></i>',
        receiverPage: 'open_orders'
      },
      4: {
        title: 'Ask Order Cancellation',
        i18nKeyTitle: 'ask_order_cancellation',
        iconHTML: '<i class="fa fa-times"></i>',
        receiverPage: 'open_orders'
      },
      5: {
        title: 'Bid Order Cancellation',
        i18nKeyTitle: 'bid_order_cancellation',
        iconHTML: '<i class="fa fa-times"></i>',
        receiverPage: 'open_orders'
      },
      6: {
        title: 'Dividend Payment',
        i18nKeyTitle: 'dividend_payment',
        iconHTML: '<i class="fa fa-gift"></i>',
        receiverPage: 'transactions'
      },
      7: {
        title: 'Asset Delete',
        i18nKeyTitle: 'delete_asset_shares',
        iconHTML: '<i class="fa fa-remove"></i>',
        receiverPage: 'transactions'
      },
      8: {
        title: 'Asset Increase',
        i18nKeyTitle: 'increase_asset_shares',
        iconHTML: '<i class="fa fa-plus"></i>',
        receiverPage: 'transactions'
      },
      9: {
        title: 'Asset Control',
        i18nKeyTitle: 'asset_control',
        iconHTML: '<i class="fa fa-lock"></i>',
        receiverPage: 'transactions'
      },
      10: {
        title: 'Set Asset Property',
        i18nKeyTitle: 'set_asset_property',
        iconHTML: '<i class="fa fa-pencil"></i>',
        receiverPage: 'transactions'
      },
      11: {
        title: 'Delete Asset Property',
        i18nKeyTitle: 'delete_asset_property',
        iconHTML: '<i class="fa fa-eraser"></i>',
        receiverPage: 'transactions'
      },
      12: {
        title: 'Set Asset Long Property',
        i18nKeyTitle: 'set_asset_property',
        iconHTML: '<i class="fa fa-pencil"></i>',
        receiverPage: 'transactions'
      }
    }
  },

  // Type 3: Marketplace (Digital Goods Store)
  3: {
    title: 'Marketplace',
    i18nKeyTitle: 'marketplace',
    iconHTML: '<i class="fa fa-shopping-cart"></i>',
    chainType: 'child',
    subTypes: {
      0: {
        title: 'DGS Listing',
        i18nKeyTitle: 'marketplace_listing',
        iconHTML: '<i class="fa fa-bullhorn"></i>'
      },
      1: {
        title: 'DGS Delisting',
        i18nKeyTitle: 'marketplace_removal',
        iconHTML: '<i class="fa fa-times"></i>'
      },
      2: {
        title: 'DGS Price Change',
        i18nKeyTitle: 'marketplace_price_change',
        iconHTML: '<i class="fa fa-line-chart"></i>'
      },
      3: {
        title: 'DGS Quantity Change',
        i18nKeyTitle: 'marketplace_quantity_change',
        iconHTML: '<i class="fa fa-sort"></i>'
      },
      4: {
        title: 'DGS Purchase',
        i18nKeyTitle: 'marketplace_purchase',
        iconHTML: '<i class="fa fa-money"></i>',
        receiverPage: 'pending_orders_dgs'
      },
      5: {
        title: 'DGS Delivery',
        i18nKeyTitle: 'marketplace_delivery',
        iconHTML: '<i class="fa fa-cube"></i>',
        receiverPage: 'purchased_dgs'
      },
      6: {
        title: 'DGS Feedback',
        i18nKeyTitle: 'marketplace_feedback',
        iconHTML: '<i class="ion-android-social"></i>',
        receiverPage: 'completed_orders_dgs'
      },
      7: {
        title: 'DGS Refund',
        i18nKeyTitle: 'marketplace_refund',
        iconHTML: '<i class="fa fa-reply"></i>',
        receiverPage: 'purchased_dgs'
      }
    }
  },

  // Type 4: Account Control (child)
  4: {
    title: 'Account Control',
    i18nKeyTitle: 'account_control',
    iconHTML: '<i class="ion-locked"></i>',
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Balance Leasing',
        i18nKeyTitle: 'balance_leasing',
        iconHTML: '<i class="fa fa-arrow-circle-o-right"></i>',
        receiverPage: 'transactions'
      },
      1: {
        title: 'Phasing Only',
        i18nKeyTitle: 'phasing_only',
        iconHTML: '<i class="fa fa-gavel"></i>',
        receiverPage: 'transactions'
      }
    }
  },

  // Type 5: Monetary System
  5: {
    title: 'Monetary System',
    i18nKeyTitle: 'monetary_system',
    iconHTML: '<i class="fa fa-bank"></i>',
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Currency Issuance',
        i18nKeyTitle: 'issue_currency',
        iconHTML: '<i class="fa fa-bullhorn"></i>'
      },
      1: {
        title: 'Reserve Increase',
        i18nKeyTitle: 'reserve_increase',
        iconHTML: '<i class="fa fa-cubes"></i>'
      },
      2: {
        title: 'Reserve Claim',
        i18nKeyTitle: 'reserve_claim',
        iconHTML: '<i class="fa fa-truck"></i>',
        receiverPage: 'currencies'
      },
      3: {
        title: 'Currency Transfer',
        i18nKeyTitle: 'currency_transfer',
        iconHTML: '<i class="ion-arrow-swap"></i>',
        receiverPage: 'currencies'
      },
      4: {
        title: 'Publish Exchange Offer',
        i18nKeyTitle: 'publish_exchange_offer',
        iconHTML: '<i class="fa fa-list-alt "></i>'
      },
      5: {
        title: 'Currency Buy',
        i18nKeyTitle: 'currency_buy',
        iconHTML: '<i class="ion-arrow-graph-up-right"></i>',
        receiverPage: 'currencies'
      },
      6: {
        title: 'Currency Sell',
        i18nKeyTitle: 'currency_sell',
        iconHTML: '<i class="ion-arrow-graph-down-right"></i>',
        receiverPage: 'currencies'
      },
      7: {
        title: 'Currency Mint',
        i18nKeyTitle: 'mint_currency',
        iconHTML: '<i class="fa fa-money"></i>',
        receiverPage: 'currencies'
      },
      8: {
        title: 'Currency Delete',
        i18nKeyTitle: 'delete_currency',
        iconHTML: '<i class="fa fa-times"></i>'
      }
    }
  },

  // Type 6: Data Cloud
  6: {
    title: 'Data Cloud',
    i18nKeyTitle: 'tagged_data',
    iconHTML: '<i class="fa fa-dashboard"></i>',
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Tagged Data Upload',
        i18nKeyTitle: 'upload_tagged_data',
        iconHTML: '<i class="fa fa-upload"></i>'
      },
      1: {
        title: 'Extend Tagged Data',
        i18nKeyTitle: 'extend_tagged_data',
        iconHTML: '<i class="fa fa-expand"></i>'
      }
    }
  },

  // Type 7: Shuffling
  7: {
    title: 'Shuffling',
    i18nKeyTitle: 'shuffling',
    iconHTML: '<i class="fa fa-random"></i>',
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Shuffling Creation',
        i18nKeyTitle: 'shuffling_creation',
        iconHTML: '<i class="fa fa-plus"></i>'
      },
      1: {
        title: 'Shuffling Registration',
        i18nKeyTitle: 'shuffling_registration',
        iconHTML: '<i class="fa fa-link"></i>'
      },
      2: {
        title: 'Shuffling Processing',
        i18nKeyTitle: 'shuffling_processing',
        iconHTML: '<i class="fa fa-cog"></i>'
      },
      3: {
        title: 'Shuffling Recipients',
        i18nKeyTitle: 'shuffling_recipients',
        iconHTML: '<i class="fa fa-spoon"></i>'
      },
      4: {
        title: 'Shuffling Verification',
        i18nKeyTitle: 'shuffling_verification',
        iconHTML: '<i class="fa fa-check-square"></i>'
      },
      5: {
        title: 'Shuffling Cancellation',
        i18nKeyTitle: 'shuffling_cancellation',
        iconHTML: '<i class="fa fa-thumbs-down"></i>'
      }
    }
  },

  // Type 8: Aliases
  8: {
    title: 'Aliases',
    i18nKeyTitle: 'aliases',
    iconHTML: "<i class='fa fa-envelope-square'></i>",
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Alias Assignment',
        i18nKeyTitle: 'alias_assignment',
        iconHTML: "<i class='fa fa-bookmark'></i>"
      },
      1: {
        title: 'Alias Sale',
        i18nKeyTitle: 'alias_sale_transfer',
        iconHTML: "<i class='fa fa-tag'></i>",
        receiverPage: 'aliases'
      },
      2: {
        title: 'Alias Buy',
        i18nKeyTitle: 'alias_buy',
        iconHTML: "<i class='fa fa-money-bill-alt'></i>",
        receiverPage: 'aliases'
      },
      3: {
        title: 'Alias Deletion',
        i18nKeyTitle: 'alias_deletion',
        iconHTML: "<i class='fa fa-times'></i>"
      }
    }
  },

  // Type 9: Voting
  9: {
    title: 'Voting',
    i18nKeyTitle: 'voting',
    iconHTML: "<i class='fa fa-check-square'></i>",
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Poll Creation',
        i18nKeyTitle: 'poll_creation',
        iconHTML: "<i class='fa fa-check-square'></i>"
      },
      1: {
        title: 'Vote Casting',
        i18nKeyTitle: 'vote_casting',
        iconHTML: "<i class='fa fa-check'></i>"
      },
      2: {
        title: 'Transaction Approval',
        i18nKeyTitle: 'transaction_approval',
        iconHTML: "<i class='fa fa-gavel'></i>",
        receiverPage: 'transactions'
      }
    }
  },

  // Type 10: Account Properties
  10: {
    title: 'Account Properties',
    i18nKeyTitle: 'account_properties',
    iconHTML: "<i class='fa fa-address-card'></i>",
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Account Info',
        i18nKeyTitle: 'account_info',
        iconHTML: "<i class='fa fa-info'></i>"
      },
      1: {
        title: 'Account Property',
        i18nKeyTitle: 'account_property',
        iconHTML: "<i class='fa fa-pencil'></i>",
        receiverPage: 'transactions'
      },
      2: {
        title: 'Account Property Delete',
        i18nKeyTitle: 'account_property_delete',
        iconHTML: "<i class='fa fa-eraser'></i>",
        receiverPage: 'transactions'
      }
    }
  },

  // Type 11: Coin Exchange (child)
  11: {
    title: 'Coin Exchange',
    i18nKeyTitle: 'coin_exchange',
    iconHTML: "<i class='fa fa-exchange-alt'></i>",
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Issue Order',
        i18nKeyTitle: 'issue_order',
        iconHTML: "<i class='fa fa-money-bill-alt'></i>",
        receiverPage: 'open_coin_orders'
      },
      1: {
        title: 'Cancel Order',
        i18nKeyTitle: 'cancel_order',
        iconHTML: "<i class='fa fa-times'></i>",
        receiverPage: 'open_coin_orders'
      }
    }
  },

  // Type 12: Contract Reference
  12: {
    title: 'Contract Reference',
    i18nKeyTitle: 'contract_reference',
    iconHTML: "<i class='fa fa-address-card'></i>",
    chainType: 'child',
    subTypes: {
      0: {
        title: 'Set Contract Reference',
        i18nKeyTitle: 'set_contract_reference',
        iconHTML: "<i class='fa fa-pencil'></i>",
        receiverPage: 'transactions'
      },
      1: {
        title: 'Delete Contract Reference',
        i18nKeyTitle: 'delete_contract_reference',
        iconHTML: "<i class='fa fa-eraser'></i>",
        receiverPage: 'transactions'
      }
    }
  }
}

// ============================================================================
// String-based type/subtype lookup map
// ============================================================================

/**
 * Maps named type strings like "ordinary_payment" to their { type, subtype } pairs.
 * This mimics the NRS.subtype lookup object.
 */
export const TRANSACTION_SUBTYPE_MAP: Record<string, { type: number; subtype: number }> = {
  // Payment
  ordinary_payment: { type: 0, subtype: 0 },

  // Messaging
  arbitrary_message: { type: 1, subtype: 0 },
  alias_assignment: { type: 1, subtype: 1 },
  poll_creation: { type: 1, subtype: 2 },
  vote_casting: { type: 1, subtype: 3 },
  hub_announcement: { type: 1, subtype: 4 },
  account_info: { type: 1, subtype: 5 },
  alias_sale_transfer: { type: 1, subtype: 6 },
  alias_buy: { type: 1, subtype: 7 },
  alias_deletion: { type: 1, subtype: 8 },
  transaction_approval: { type: 1, subtype: 9 },
  account_property: { type: 1, subtype: 10 },
  account_property_delete: { type: 1, subtype: 11 },

  // Asset Exchange
  asset_issuance: { type: 2, subtype: 0 },
  asset_transfer: { type: 2, subtype: 1 },
  ask_order_placement: { type: 2, subtype: 2 },
  bid_order_placement: { type: 2, subtype: 3 },
  ask_order_cancellation: { type: 2, subtype: 4 },
  bid_order_cancellation: { type: 2, subtype: 5 },
  dividend_payment: { type: 2, subtype: 6 },
  delete_asset_shares: { type: 2, subtype: 7 },
  increase_asset_shares: { type: 2, subtype: 8 },
  asset_control: { type: 2, subtype: 9 },
  set_asset_property: { type: 2, subtype: 10 },
  delete_asset_property: { type: 2, subtype: 11 },

  // Marketplace
  marketplace_listing: { type: 3, subtype: 0 },
  marketplace_removal: { type: 3, subtype: 1 },
  marketplace_price_change: { type: 3, subtype: 2 },
  marketplace_quantity_change: { type: 3, subtype: 3 },
  marketplace_purchase: { type: 3, subtype: 4 },
  marketplace_delivery: { type: 3, subtype: 5 },
  marketplace_feedback: { type: 3, subtype: 6 },
  marketplace_refund: { type: 3, subtype: 7 },

  // Account Control (child)
  balance_leasing: { type: 4, subtype: 0 },
  phasing_only: { type: 4, subtype: 1 },

  // Monetary System
  issue_currency: { type: 5, subtype: 0 },
  reserve_increase: { type: 5, subtype: 1 },
  reserve_claim: { type: 5, subtype: 2 },
  currency_transfer: { type: 5, subtype: 3 },
  publish_exchange_offer: { type: 5, subtype: 4 },
  currency_buy: { type: 5, subtype: 5 },
  currency_sell: { type: 5, subtype: 6 },
  mint_currency: { type: 5, subtype: 7 },
  delete_currency: { type: 5, subtype: 8 },

  // Data Cloud
  upload_tagged_data: { type: 6, subtype: 0 },
  extend_tagged_data: { type: 6, subtype: 1 },

  // Shuffling
  shuffling_creation: { type: 7, subtype: 0 },
  shuffling_registration: { type: 7, subtype: 1 },
  shuffling_processing: { type: 7, subtype: 2 },
  shuffling_recipients: { type: 7, subtype: 3 },
  shuffling_verification: { type: 7, subtype: 4 },
  shuffling_cancellation: { type: 7, subtype: 5 },

  // Coin Exchange (child)
  issue_order: { type: 11, subtype: 0 },
  cancel_order: { type: 11, subtype: 1 },

  // Contract Reference
  set_contract_reference: { type: 12, subtype: 0 },
  delete_contract_reference: { type: 12, subtype: 1 }
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Get the TransactionSubType metadata for a given type and subtype.
 *
 * @param type - Transaction type number
 * @param subtype - Transaction subtype number
 * @returns The matching subtype metadata, or undefined if not found
 */
export function getTransactionTypeDef(
  type: number,
  subtype: number
): TransactionSubType | undefined {
  const typeDef = TRANSACTION_TYPES[type]
  if (!typeDef) return undefined
  return typeDef.subTypes[subtype]
}

/**
 * Check if a transaction matches a named type string
 * (e.g., "ordinary_payment", "asset_transfer").
 *
 * @param transaction - Transaction object with type and subtype numbers
 * @param typeStr - Named type string from TRANSACTION_SUBTYPE_MAP
 * @returns True if the transaction matches the named type
 */
export function isOfType(
  transaction: { type: number; subtype: number },
  typeStr: string
): boolean {
  const mapping = TRANSACTION_SUBTYPE_MAP[typeStr]
  if (!mapping) {
    console.warn(`Unknown transaction type string: ${typeStr}`)
    return false
  }
  return transaction.type === mapping.type && transaction.subtype === mapping.subtype
}

/**
 * Get the display title for a transaction type.
 *
 * @param type - Transaction type number
 * @returns The type title, or "Unknown" if not registered
 */
export function getTypeName(type: number): string {
  const typeDef = TRANSACTION_TYPES[type]
  return typeDef ? typeDef.title : 'Unknown'
}

/**
 * Get the display title for a transaction subtype.
 *
 * @param type - Transaction type number
 * @param subtype - Transaction subtype number
 * @returns The subtype title, or "Unknown" if not registered
 */
export function getSubTypeName(type: number, subtype: number): string {
  const subTypeDef = getTransactionTypeDef(type, subtype)
  return subTypeDef ? subTypeDef.title : 'Unknown'
}

// ============================================================================
// 动态常量加载（端口自 nrs.transactions.types.js:574 loadTransactionTypeConstants）
// ============================================================================

/** 未知类型的默认展示元数据（服务端返回了未注册的 type/subtype 时使用） */
const UNKNOWN_TYPE_DEF: TransactionTypeDef = {
  title: 'Unknown',
  i18nKeyTitle: 'unknown',
  iconHTML: '<i class="fa fa-question-circle"></i>',
  chainType: 'child',
  subTypes: {}
}

/** 未知子类型的默认展示元数据 */
const UNKNOWN_SUBTYPE_DEF: TransactionSubType = {
  title: 'Unknown',
  i18nKeyTitle: 'unknown',
  iconHTML: '<i class="fa fa-question-circle"></i>'
}

/**
 * 将 `getConstants` 响应中的交易类型常量合并进静态 `TRANSACTION_TYPES`。
 *
 * 端口自 `nrs.transactions.types.js:574` 的 `NRS.loadTransactionTypeConstants`：
 * - 不覆盖静态已有的 title / i18nKeyTitle / iconHTML / receiverPage（仅注入 `serverConstants`）
 * - 服务端返回了静态表未注册的 type / subtype 时，补一条 "Unknown" 占位
 * - 服务端子类型映射（`response.transactionSubTypes`，按名称索引）由调用方 Store 单独维护
 *
 * @param response - `getConstants` 响应
 * @param baseTypes - 静态基础类型表（默认 `TRANSACTION_TYPES`）
 * @returns 合并后的新类型表（不修改入参，保证 Vue 响应式）
 */
export function loadTransactionTypeConstants(
  response: ServerConstantsResponse,
  baseTypes: Record<number, TransactionTypeDef> = TRANSACTION_TYPES
): Record<number, TransactionTypeDef> {
  if (!response?.genesisAccountId || !response.transactionTypes) {
    return baseTypes
  }

  // 深拷贝基础表，避免修改静态常量
  const merged: Record<number, TransactionTypeDef> = {}
  for (const key of Object.keys(baseTypes)) {
    const numKey = Number(key)
    const base = baseTypes[numKey]
    merged[numKey] = {
      ...base,
      subTypes: { ...base.subTypes }
    }
  }

  // 合并服务端 transactionTypes
  for (const typeIndex of Object.keys(response.transactionTypes)) {
    const numType = Number(typeIndex)
    const serverType = response.transactionTypes[typeIndex]
    if (!(numType in merged)) {
      merged[numType] = {
        ...UNKNOWN_TYPE_DEF,
        subTypes: {}
      }
    }
    const targetType = merged[numType]
    for (const subTypeIndex of Object.keys(serverType.subtypes)) {
      const numSub = Number(subTypeIndex)
      const serverSub = serverType.subtypes[subTypeIndex]
      if (!(numSub in targetType.subTypes)) {
        targetType.subTypes[numSub] = { ...UNKNOWN_SUBTYPE_DEF }
      }
      // 仅注入服务端常量，不覆盖展示元数据
      targetType.subTypes[numSub] = {
        ...targetType.subTypes[numSub],
        serverConstants: serverSub
      }
    }
  }

  return merged
}
