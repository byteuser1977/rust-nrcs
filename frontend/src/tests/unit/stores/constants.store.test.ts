import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// Mock nrcsApi.getConstants（避免真实网络请求）
vi.mock('@/api/modules/nrcs.api', () => ({
  nrcsApi: {
    getConstants: vi.fn()
  }
}))

// Mock convertNumericToRSAccountFormat（避免依赖完整地址编解码链路）
vi.mock('@/utils/converters', () => ({
  convertNumericToRSAccountFormat: vi.fn((id: string) => `NRCS-${id.slice(-4)}`)
}))

import { nrcsApi } from '@/api/modules/nrcs.api'
import { useConstantsStore } from '@/stores/modules/constants.store'
import {
  loadTransactionTypeConstants,
  TRANSACTION_TYPES,
  type TransactionTypeDef
} from '@/constants/transaction-types'
import type { ServerConstantsResponse } from '@/constants/server-constants'

/** 构造一份最小但结构完整的 getConstants 响应 */
function buildResponse(): ServerConstantsResponse {
  return {
    genesisAccountId: '1739068987193023818',
    genesisBlockId: '-36992752662354113',
    epochBeginning: 1385294400000,
    maxPrunableMessageLength: 43008,
    maxTaggedDataDataLength: 43008,
    maxArbitraryMessageLength: 160,
    maxBlockPayloadLength: 44880,
    maxPhasingDuration: 20160,
    votingModels: { NQT: 1, ACCOUNT: 0, NONE: -1, HASH: 5 },
    minBalanceModels: { NQT: 1, NONE: 0 },
    holdingTypes: { NXT: 0, ASSET: 1 },
    hashAlgorithms: { SHA256: 2, SHA3: 3 },
    phasingHashAlgorithms: { SHA256: 2, RIPEMD160_SHA256: 62 },
    mintingHashAlgorithms: { SHA256: 2, SCRYPT: 5 },
    currencyTypes: { EXCHANGEABLE: 1, MINTABLE: 16 },
    peerStates: { CONNECTED: 1, NON_CONNECTED: 0 },
    shufflingStages: { REGISTRATION: 0, PROCESSING: 1 },
    shufflingParticipantStates: { REGISTERED: 0, PROCESSED: 1 },
    apiTags: { NETWORK: { name: 'Networking', enabled: true }, DEBUG: { name: 'Debug', enabled: false } },
    proxyNotForwardedRequests: ['getState', 'getPeers'],
    disabledAPIs: [],
    disabledAPITags: [],
    transactionSubTypes: {
      OrdinaryPayment: {
        isPhasable: true, subtype: 0, mustHaveRecipient: true,
        name: 'OrdinaryPayment', canHaveRecipient: true, type: 0, isPhasingSafe: true
      }
    },
    transactionTypes: {
      '0': {
        subtypes: {
          '0': {
            isPhasable: true, subtype: 0, mustHaveRecipient: true,
            name: 'OrdinaryPayment', canHaveRecipient: true, type: 0, isPhasingSafe: true
          }
        }
      },
      // 服务端返回了一个静态表里没有的 type（用于验证 Unknown 占位）
      '99': {
        subtypes: {
          '3': {
            isPhasable: false, subtype: 3, mustHaveRecipient: false,
            name: 'FutureType', canHaveRecipient: false, type: 99, isPhasingSafe: false
          }
        }
      }
    },
    requestTypes: {
      sendMoney: {
        allowRequiredBlockParameters: false, requireFullClient: false,
        requirePassword: false, requireBlockchain: true, requirePost: true, enabled: true
      },
      getState: {
        allowRequiredBlockParameters: false, requireFullClient: false,
        requirePassword: false, requireBlockchain: true, requirePost: false, enabled: true
      },
      startForging: {
        allowRequiredBlockParameters: false, requireFullClient: true,
        requirePassword: false, requireBlockchain: true, requirePost: true, enabled: true
      },
      disabledEndpoint: {
        allowRequiredBlockParameters: true, requireFullClient: false,
        requirePassword: false, requireBlockchain: true, requirePost: false, enabled: false
      }
    }
  } as ServerConstantsResponse
}

describe('loadTransactionTypeConstants', () => {
  it('注入 serverConstants 但不覆盖静态展示元数据', () => {
    const merged = loadTransactionTypeConstants(buildResponse(), TRANSACTION_TYPES)
    const ordinary = merged[0]?.subTypes[0]
    expect(ordinary).toBeDefined()
    // 静态展示元数据保留
    expect(ordinary!.title).toBe('Ordinary Payment')
    expect(ordinary!.i18nKeyTitle).toBe('ordinary_payment')
    // 服务端常量已注入
    expect(ordinary!.serverConstants).toBeDefined()
    expect(ordinary!.serverConstants!.isPhasable).toBe(true)
    expect(ordinary!.serverConstants!.mustHaveRecipient).toBe(true)
  })

  it('为服务端新增的 type/subtype 补 Unknown 占位', () => {
    const merged = loadTransactionTypeConstants(buildResponse(), TRANSACTION_TYPES)
    const futureType = merged[99]
    expect(futureType).toBeDefined()
    expect(futureType!.title).toBe('Unknown')
    const futureSub = futureType!.subTypes[3]
    expect(futureSub).toBeDefined()
    expect(futureSub!.title).toBe('Unknown')
    expect(futureSub!.serverConstants!.name).toBe('FutureType')
  })

  it('genesisAccountId 缺失时原样返回基础表', () => {
    const response = buildResponse()
    ;(response as { genesisAccountId: string }).genesisAccountId = ''
    const merged = loadTransactionTypeConstants(response, TRANSACTION_TYPES)
    // 原样返回（同一引用）
    expect(merged).toBe(TRANSACTION_TYPES)
  })

  it('不修改静态 TRANSACTION_TYPES', () => {
    const before = JSON.stringify(Object.keys(TRANSACTION_TYPES))
    loadTransactionTypeConstants(buildResponse(), TRANSACTION_TYPES)
    const after = JSON.stringify(Object.keys(TRANSACTION_TYPES))
    expect(after).toBe(before)
    // 静态表的子类型不应被注入 serverConstants
    expect(TRANSACTION_TYPES[0]?.subTypes[0]?.serverConstants).toBeUndefined()
  })
})

describe('useConstantsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('loadServerConstants 成功后填充各运行时容器', async () => {
    const response = buildResponse()
    vi.mocked(nrcsApi.getConstants).mockResolvedValue(response)

    const store = useConstantsStore()
    expect(store.loaded).toBe(false)

    await store.loadServerConstants()

    expect(store.loaded).toBe(true)
    expect(store.loading).toBe(false)
    expect(store.loadError).toBe('')
    expect(store.genesis).toBe('1739068987193023818')
    expect(store.epochBeginning).toBe(1385294400000)
    expect(store.maxPrunableMessageLength).toBe(43008)
    expect(store.votingModels).toEqual(response.votingModels)
    expect(store.hashAlgorithms).toEqual(response.hashAlgorithms)
    expect(store.requestTypes).toEqual(response.requestTypes)
    expect(store.apiTags).toEqual(response.apiTags)
    expect(store.currencyTypes).toEqual(response.currencyTypes)
    // genesisRS 由 mock 的 convertNumericToRSAccountFormat 生成
    expect(store.genesisRS).toContain('NRCS-')
    // 交易类型已合并 serverConstants
    expect(store.transactionTypes[0]?.subTypes[0]?.serverConstants).toBeDefined()
    expect(store.subtypeMap.OrdinaryPayment).toBeDefined()
  })

  it('loadServerConstants 失败时记录错误且不阻断', async () => {
    vi.mocked(nrcsApi.getConstants).mockRejectedValue(new Error('network down'))

    const store = useConstantsStore()
    await store.loadServerConstants()

    expect(store.loaded).toBe(false)
    expect(store.loading).toBe(false)
    expect(store.loadError).toBe('network down')
  })

  it('并发调用只发一次请求', async () => {
    vi.mocked(nrcsApi.getConstants).mockResolvedValue(buildResponse())

    const store = useConstantsStore()
    await Promise.all([store.loadServerConstants(), store.loadServerConstants()])

    expect(nrcsApi.getConstants).toHaveBeenCalledTimes(1)
  })

  describe('判定函数（端口自 nrs.constants.js）', () => {
    let store: ReturnType<typeof useConstantsStore>

    beforeEach(async () => {
      vi.mocked(nrcsApi.getConstants).mockResolvedValue(buildResponse())
      store = useConstantsStore()
      await store.loadServerConstants()
    })

    it('isRequireBlockchain / isRequireFullClient / isRequirePost', () => {
      expect(store.isRequireBlockchain('sendMoney')).toBe(true)
      expect(store.isRequirePost('sendMoney')).toBe(true)
      expect(store.isRequireFullClient('sendMoney')).toBe(false)
      expect(store.isRequireFullClient('startForging')).toBe(true)
      // 未注册的 requestType 隐式返回 false
      expect(store.isRequireBlockchain('unknownRequest')).toBe(false)
    })

    it('isRequestForwardable：排除 requireFullClient 与 proxyNotForwardedRequests', () => {
      // sendMoney: requireBlockchain 且非 requireFullClient 且不在不转发列表 → 可转发
      expect(store.isRequestForwardable('sendMoney')).toBe(true)
      // startForging: requireFullClient → 不可转发
      expect(store.isRequestForwardable('startForging')).toBe(false)
      // getState: 在 proxyNotForwardedRequests 列表 → 不可转发
      expect(store.isRequestForwardable('getState')).toBe(false)
    })

    it('isRequestTypeEnabled：未知请求放行、+ 前缀剥离、已注册即视为启用', () => {
      // 注意：参考 nrs.constants.js:222 的 isRequestTypeEnabled 仅检查 requestType
      // 是否存在于 REQUEST_TYPES（!!REQUEST_TYPES[requestType]），不看其 enabled 字段。
      // enabled 字段由 isApiEnabled（tags/apis 依赖）间接判定。
      expect(store.isRequestTypeEnabled('sendMoney')).toBe(true)
      // disabledEndpoint 已注册（即便 enabled=false），故视为启用
      expect(store.isRequestTypeEnabled('disabledEndpoint')).toBe(true)
      // + 前缀剥离：复合请求 sendMoney+foo 应剥离为 sendMoney
      expect(store.isRequestTypeEnabled('sendMoney+foo')).toBe(true)
      // 未注册的请求 → false
      expect(store.isRequestTypeEnabled('scheduleCurrencyBuy')).toBe(false)
    })

    it('未加载时 isRequestTypeEnabled 隐式放行', () => {
      const empty = useConstantsStore()
      empty.reset()
      expect(empty.isRequestTypeEnabled('anything')).toBe(true)
    })

    it('isSubmitPassphrase / isScheduleRequest', () => {
      expect(store.isSubmitPassphrase('startForging')).toBe(true)
      expect(store.isSubmitPassphrase('sendMoney')).toBe(false)
      expect(store.isScheduleRequest('scheduleCurrencyBuy')).toBe(true)
      expect(store.isScheduleRequest('sendMoney')).toBe(false)
    })

    it('isApiEnabled：任一依赖未启用则返回 false', () => {
      expect(store.isApiEnabled()).toBe(true)
      expect(
        store.isApiEnabled({ tags: [{ name: 'Networking', enabled: true }] })
      ).toBe(true)
      expect(
        store.isApiEnabled({ tags: [{ name: 'Debug', enabled: false }] })
      ).toBe(false)
      expect(
        store.isApiEnabled({
          apis: [
            { allowRequiredBlockParameters: true, requireFullClient: false, requirePassword: false, requireBlockchain: true, requirePost: false, enabled: true },
            { allowRequiredBlockParameters: true, requireFullClient: false, requirePassword: false, requireBlockchain: true, requirePost: false, enabled: false }
          ]
        })
      ).toBe(false)
    })

    it('getKeyByValue / getVotingModelName / getHashAlgorithm', () => {
      expect(store.getVotingModelName(1)).toBe('NQT')
      expect(store.getVotingModelName(5)).toBe('HASH')
      expect(store.getVotingModelName(999)).toBeNull()
      expect(store.getVotingModelCode('ACCOUNT')).toBe(0)
      expect(store.getHashAlgorithm(2)).toBe('SHA256')
      expect(store.getShufflingStage(0)).toBe('REGISTRATION')
      expect(store.getPeerState(1)).toBe('CONNECTED')
      expect(store.getCurrencyType(16)).toBe('MINTABLE')
    })

    it('getECBlock：主网/测试网回退值', () => {
      expect(store.getECBlock(false).id).toBe('3488276486778630462')
      expect(store.getECBlock(true).id).toBe('3488276486778630462')
    })

    it('getFileUploadConfig', () => {
      const uploadCfg = store.getFileUploadConfig('uploadTaggedData')
      expect(uploadCfg?.requestParam).toBe('file')
      expect(uploadCfg?.maxSize).toBe(43008)
      expect(uploadCfg?.errorDescription).toBe('error_file_too_big')

      const dgsCfg = store.getFileUploadConfig('dgsListing')
      expect(dgsCfg?.requestParam).toBe('messageFile')

      const msgCfg = store.getFileUploadConfig('sendMessage', { encrypt_message: true })
      expect(msgCfg?.requestParam).toBe('encryptedMessageFile')
      const msgCfgPlain = store.getFileUploadConfig('sendMessage', {})
      expect(msgCfgPlain?.requestParam).toBe('messageFile')

      expect(store.getFileUploadConfig('sendMoney')).toBeNull()
    })

    it('getAlgorithmOptions', () => {
      const opts = store.getAlgorithmOptions(false)
      expect(opts).toContainEqual({ label: 'SHA256', value: 2 })
      const phasing = store.getAlgorithmOptions(true)
      expect(phasing).toContainEqual({ label: 'RIPEMD160_SHA256', value: 62 })
    })
  })

  describe('reset', () => {
    it('清空所有运行时容器', async () => {
      vi.mocked(nrcsApi.getConstants).mockResolvedValue(buildResponse())
      const store = useConstantsStore()
      await store.loadServerConstants()
      expect(store.loaded).toBe(true)

      store.reset()

      expect(store.loaded).toBe(false)
      expect(store.server).toBeNull()
      expect(store.votingModels).toEqual({})
      expect(store.requestTypes).toEqual({})
      expect(store.transactionTypes[0]?.subTypes[0]?.serverConstants).toBeUndefined()
      expect(store.subtypeMap).toEqual({})
      expect(store.genesis).toBe('')
    })
  })
})
