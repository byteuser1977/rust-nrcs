/**
 * 收款人实时校验 Composable
 *
 * 完整移植自 NRCS 参考实现 `nrs.recipient.js`（386 行）：
 *   - `checkRecipient`（:199-295）：失焦实时校验，支持 RS/数字 ID/联系人名/alias 四种输入
 *   - `correctAddressMistake`（:195）：点击纠正建议自动填充
 *   - `getAccountError`（:132-193）：调 getAccount 检测公钥/余额，返回 info/warning/danger 三类状态
 *   - `checkRecipientAlias`（:296-370）：alias → account 解析（acct:xxx@nxt / nacc:xxx 格式）
 *   - `checkForMerchant`（:372-383）：商家描述字段解析，含 "merchant" 关键字时自动勾选消息
 *   - `automaticallyCheckRecipient`（:21-49）：为收款人输入框绑定 blur 校验
 *
 * 复用：
 *   - `utils/nrs-address.ts` 的 NrsAddress.set/guess/formatGuess（RS 校验与纠错）
 *   - `api/modules/nrcs.api.ts` 的 nrcsApi.getAccount/getAlias/getAccountPublicKey
 *   - `utils/nrcs-storage.ts` 的 storageSelect（联系人查询，按账户隔离）
 *   - `utils/format.ts` 的 formatAmount/formatTimestamp
 *
 * 安全模型：本模块不涉及 secretPhrase，仅做收款方账户查询与展示。
 */
import { ref, type Ref } from 'vue'
import { nrcsApi, type NrcsAccount, type NrcsAlias } from '@/api/modules'
import { NrsAddress } from '@/utils/nrs-address'
import { storageSelect } from '@/utils/nrcs-storage'
import { formatAmount, formatTimestamp } from '@/utils/format'

/** 校验结果类型（对标 nrs.recipient.js response.type） */
export type RecipientCheckType = 'info' | 'warning' | 'danger' | 'success'

/** 联系人记录（IndexedDB contacts 表结构） */
interface ContactRecord {
  accountId?: string
  accountRS?: string
  name?: string
  email?: string
  description?: string
}

/** 单条收款人校验结果 */
export interface RecipientCheckResult {
  /** 状态类型：info（已知账户有公钥）/ warning（无公钥或未知）/ danger（格式错误）/ success（自身账户） */
  type: RecipientCheckType
  /** 展示给用户的消息（已转义，可含 HTML 用于纠正建议高亮） */
  message: string
  /** 链上账户数据（getAccount 响应；未知账户为 null） */
  account: NrcsAccount | null
  /** 是否缺少公钥（需要接收方提供 publicKey 或广播后才能安全发送） */
  noPublicKey?: boolean
  /** 纠正建议列表（RS 格式错误时，NrsAddress.guess 返回的候选地址） */
  suggestions?: Array<{ address: string; formatted: string }>
  /** 解析后的账户 RS（联系人名/alias 转换后的真实 RS 地址） */
  convertedAccount?: string
  /** 商家信息（账户 description 含 "merchant" 时填充） */
  merchantInfo?: string
  /** 是否需要展示 recipientPublicKey 输入框 */
  showRecipientPublicKey?: boolean
}

/** 校验状态：idle（未校验）/ checking（校验中）/ done（校验完成） */
export type CheckStatus = 'idle' | 'checking' | 'done'

/**
 * 收款人实时校验 Composable
 *
 * 用法：
 * ```vue
 * const { result, status, checkRecipient, correctAddressMistake } = useRecipientCheck()
 * // 输入框 blur 时调用
 * await checkRecipient(form.recipient, { requestType: 'sendMoney', selfAccountRS: accountStore.accountRS })
 * // 点击纠正建议
 * function onSuggestionClick(addr: string) { form.recipient = addr; checkRecipient(form.recipient, ...) }
 * ```
 *
 * @returns 响应式结果、状态、校验方法、纠正方法
 */
export function useRecipientCheck() {
  /** 当前校验结果（null 表示无输入或已清空） */
  const result: Ref<RecipientCheckResult | null> = ref(null)
  /** 校验状态 */
  const status: Ref<CheckStatus> = ref('idle')

  /** RS 地址格式正则（对标 nrs.recipient.js:212） */
  const RS_REGEX = /^(NRCS\-)?[A-Z0-9]+\-[A-Z0-9]+\-[A-Z0-9]+\-[A-Z0-9]+/i
  /** 纯数字正则 */
  const NUMERIC_REGEX = /^\d+$/
  /** 字母数字正则（联系人名/alias 名校验） */
  const ALPHANUM_REGEX = /^[a-z0-9]+$/i
  /** alias URI 中账户解析正则（acct:xxx@nxt 或 nacc:xxx） */
  const ALIAS_ACCT_REGEX_1 = /acct:(.*)@nxt/
  const ALIAS_ACCT_REGEX_2 = /nacc:(.*)/

  /**
   * 调 getAccount 检测收款方账户状态
   *
   * 对标 nrs.recipient.js:132-193 getAccountError：
   *   - 有 publicKey + 有 name → info: "收款方：{name}，余额：{balance}"
   *   - 有 publicKey + 无 name → info: "收款方余额：{balance}"
   *   - errorCode == 4 → danger: "收款地址格式错误"（+ 非 RS 时建议使用 alias）
   *   - errorCode == 5 → warning: "未知账户，无公钥"
   *   - 其他 errorCode → danger: "收款方问题：{errorDescription}"
   *   - 无 publicKey 无 errorCode → warning: "账户无公钥，余额：{balance}"
   *
   * @param accountId RS 地址或数字账户 ID
   * @param selfAccountRS 当前登录账户 RS（用于判断是否为自身账户）
   * @returns 校验结果
   */
  async function getAccountError(
    accountId: string,
    selfAccountRS?: string,
  ): Promise<RecipientCheckResult> {
    try {
      const account: NrcsAccount = await nrcsApi.getAccount(accountId)

      // 有公钥：正常账户
      if (account.publicKey) {
        const balance = formatAmount(account.unconfirmedBalanceNQT, false, undefined, true)
        const name = account.name ? unescapeRespStr(account.name) : ''
        const message = name
          ? `收款方：${name}，余额：${balance} NRCS`
          : `收款方余额：${balance} NRCS`
        return {
          type: 'info',
          message,
          account,
          noPublicKey: false,
          showRecipientPublicKey: false,
        }
      }

      // 无公钥但账户存在（未发送过交易）
      const balance = formatAmount(account.unconfirmedBalanceNQT, false, undefined, true)
      return {
        type: 'warning',
        message: `账户无公钥，余额：${balance} NRCS。发送到此账户需提供公钥或广播后才能确认。`,
        account,
        noPublicKey: true,
        showRecipientPublicKey: selfAccountRS ? accountId !== selfAccountRS : true,
      }
    } catch (e: any) {
      // errorCode 分类处理
      const code = e.code
      if (code === 4) {
        // 格式错误
        const isRS = /^NRCS\-/i.test(accountId)
        return {
          type: 'danger',
          message: `收款地址格式错误${isRS ? '' : '，您可以使用 alias 名称代替'}`,
          account: null,
        }
      }
      if (code === 5) {
        // 未知账户
        return {
          type: 'warning',
          message: '未知账户，该账户尚未在链上发送过交易（无公钥）',
          account: null,
          noPublicKey: true,
          showRecipientPublicKey: true,
        }
      }
      return {
        type: 'danger',
        message: `收款方问题：${e.description || e.message || '未知错误'}`,
        account: null,
      }
    }
  }

  /**
   * 校验 alias 名称并解析为账户
   *
   * 对标 nrs.recipient.js:296-370 checkRecipientAlias：
   *   1. 调 getAlias(aliasName) 查询 alias
   *   2. 解析 aliasURI（acct:xxx@nxt 或 nacc:xxx 格式）
   *   3. 解析出的账户转 RS 后调 getAccountError
   *   4. 返回含 alias 最后调整时间的完整信息
   *
   * @param aliasName alias 名称
   * @param selfAccountRS 当前登录账户 RS
   * @returns 校验结果
   */
  async function checkRecipientAlias(
    aliasName: string,
    selfAccountRS?: string,
  ): Promise<RecipientCheckResult> {
    try {
      const aliasInfo: NrcsAlias = await nrcsApi.getAlias(undefined, aliasName)

      if (!aliasInfo || !aliasInfo.aliasURI) {
        return {
          type: 'danger',
          message: aliasInfo?.aliasName
            ? '该 alias 的 URI 为空'
            : `alias 错误：${aliasInfo || '未找到'}`,
          account: null,
        }
      }

      const aliasUri = String(aliasInfo.aliasURI)
      const timestamp = aliasInfo.timestamp

      // 解析 acct:xxx@nxt 或 nacc:xxx 格式
      let match = aliasUri.match(ALIAS_ACCT_REGEX_1)
      if (!match) {
        match = aliasUri.match(ALIAS_ACCT_REGEX_2)
      }

      if (!match || !match[1]) {
        return {
          type: 'danger',
          message: `alias 未关联账户${aliasUri ? `，URI 为：${aliasUri}` : '，URI 为空'}`,
          account: null,
        }
      }

      // 解析出的账户 ID 转大写
      let resolvedAccount = String(match[1]).toUpperCase()

      // 若为纯数字，转为 RS 地址
      if (NUMERIC_REGEX.test(resolvedAccount)) {
        const addr = new NrsAddress()
        if (addr.set(resolvedAccount)) {
          resolvedAccount = addr.toString()
        } else {
          return {
            type: 'danger',
            message: 'alias 关联的账户 ID 无效',
            account: null,
          }
        }
      }

      // 调 getAccountError 查询账户状态
      const accountResult = await getAccountError(resolvedAccount, selfAccountRS)
      const timeStr = formatTimestamp(timestamp)

      return {
        ...accountResult,
        message: `alias "${aliasName}" → ${resolvedAccount}。${accountResult.message}（alias 最后调整：${timeStr}）`,
        convertedAccount:
          accountResult.type === 'info' || accountResult.type === 'warning'
            ? resolvedAccount
            : undefined,
      }
    } catch (e: any) {
      return {
        type: 'danger',
        message: `无效的账户 ID 或 alias：${e.description || e.message || '未找到'}`,
        account: null,
      }
    }
  }

  /**
   * 检测账户描述是否含商家信息
   *
   * 对标 nrs.recipient.js:372-383 checkForMerchant：
   * 仅 sendMoney/transferAsset 请求类型生效；description 含 "merchant" 关键字时
   * 返回商家信息并提示自动勾选消息。
   *
   * @param accountInfo 账户描述
   * @param requestType 当前请求类型
   * @returns 商家信息（无则空字符串）
   */
  function checkForMerchant(accountInfo: string, requestType: string): string {
    if (requestType !== 'sendMoney' && requestType !== 'transferAsset') {
      return ''
    }
    if (accountInfo && /merchant/i.test(accountInfo)) {
      return accountInfo
    }
    return ''
  }

  /**
   * 主校验函数：支持 RS/数字 ID/联系人名/alias 四种输入
   *
   * 对标 nrs.recipient.js:199-294 checkRecipient：
   *   1. 匹配 RS 格式 → NrsAddress.set 校验：
   *      - 成功 → getAccountError + checkForMerchant
   *      - 失败 → 根据 guess 返回纠正建议（1 个/多个/无）
   *   2. 非纯数字 → 查联系人（storageSelect contacts by name）：
   *      - 找到 → getAccountError(contact.accountRS)
   *      - 未找到 + 纯字母数字 → checkRecipientAlias
   *      - 未找到 + 非字母数字 → "收款地址格式错误"
   *   3. 以 @ 开头 + 纯字母数字@ → 去掉 @ 后 checkRecipientAlias
   *   4. 纯数字 → "不允许使用数字账户 ID"
   *
   * @param account 用户输入的收款方（RS/数字 ID/联系人名/alias）
   * @param options.selfAccountRS 当前登录账户 RS（用于判断自身账户与无公钥提示）
   * @param options.requestType 请求类型（sendMoney/transferAsset 等，用于商家检测）
   * @returns 校验结果（null 表示输入为空）
   */
  async function checkRecipient(
    account: string,
    options: { selfAccountRS?: string; requestType?: string } = {},
  ): Promise<RecipientCheckResult | null> {
    const trimmed = (account || '').trim()

    // 空输入清空结果
    if (!trimmed) {
      result.value = null
      status.value = 'idle'
      return null
    }

    status.value = 'checking'

    const { selfAccountRS, requestType = 'sendMoney' } = options
    let res: RecipientCheckResult

    try {
      // 情况 1：RS 地址格式（NRCS-XXXX-XXXX-XXXX-XXXXX）
      if (RS_REGEX.test(trimmed)) {
        const addr = new NrsAddress()
        if (addr.set(trimmed)) {
          // 校验成功
          res = await getAccountError(trimmed, selfAccountRS)

          // 自身账户特殊提示
          if (selfAccountRS && trimmed.toUpperCase() === selfAccountRS.toUpperCase()) {
            res = {
              ...res,
              type: 'success',
              message: '这是您自己的账户',
            }
          }

          // 商家信息检测
          if (res.account?.description) {
            const merchant = checkForMerchant(res.account.description, requestType)
            if (merchant) {
              res.merchantInfo = merchant
            }
          }
        } else {
          // 校验失败 → 纠正建议
          const guesses = addr.guess
          if (guesses.length === 1) {
            res = {
              type: 'danger',
              message: `收款地址格式错误，您是否想输入：`,
              account: null,
              suggestions: [
                {
                  address: guesses[0],
                  formatted: addr.formatGuess(guesses[0], trimmed),
                },
              ],
            }
          } else if (guesses.length > 1) {
            res = {
              type: 'danger',
              message: `收款地址格式错误，找到 ${guesses.length} 个可能的纠正：`,
              account: null,
              suggestions: guesses.map((g) => ({
                address: g,
                formatted: addr.formatGuess(g, trimmed),
              })),
            }
          } else {
            res = {
              type: 'danger',
              message: '收款地址格式错误，无法识别',
              account: null,
            }
          }
        }
      } else if (!NUMERIC_REGEX.test(trimmed)) {
        // 情况 2/3：非纯数字 → 联系人或 alias
        if (trimmed.charAt(0) === '@') {
          // 以 @ 开头 → alias
          const aliasName = trimmed.substring(1)
          if (ALPHANUM_REGEX.test(aliasName)) {
            res = await checkRecipientAlias(aliasName, selfAccountRS)
            if (res.account?.description) {
              const merchant = checkForMerchant(res.account.description, requestType)
              if (merchant) res.merchantInfo = merchant
            }
          } else {
            res = { type: 'danger', message: '收款地址格式错误', account: null }
          }
        } else {
          // 查联系人（按名称）
          const contacts = await storageSelect<ContactRecord>('contacts', [{ name: trimmed }])
          if (contacts && contacts.length > 0) {
            const contact = contacts[0]
            const contactRS = contact.accountRS || contact.accountId || ''
            if (contactRS) {
              res = await getAccountError(contactRS, selfAccountRS)
              res = {
                ...res,
                message: `联系人 "${trimmed}" → ${contactRS}。${res.message}`,
                convertedAccount:
                  res.type === 'info' || res.type === 'warning' ? contactRS : undefined,
              }
              if (res.account?.description) {
                const merchant = checkForMerchant(res.account.description, requestType)
                if (merchant) res.merchantInfo = merchant
              }
            } else {
              res = { type: 'danger', message: `联系人 "${trimmed}" 缺少账户信息`, account: null }
            }
          } else if (ALPHANUM_REGEX.test(trimmed)) {
            // 未找到联系人，尝试 alias
            res = await checkRecipientAlias(trimmed, selfAccountRS)
            if (res.account?.description) {
              const merchant = checkForMerchant(res.account.description, requestType)
              if (merchant) res.merchantInfo = merchant
            }
          } else {
            res = { type: 'danger', message: '收款地址格式错误', account: null }
          }
        }
      } else {
        // 情况 4：纯数字 → 不允许
        res = {
          type: 'danger',
          message: '不允许使用纯数字账户 ID，请输入 NRCS-XXXX-XXXX-XXXX-XXXXX 格式的 RS 地址',
          account: null,
        }
      }
    } catch (e: any) {
      res = {
        type: 'danger',
        message: `校验失败：${e.message || e.description || '未知错误'}`,
        account: null,
      }
    }

    result.value = res
    status.value = 'done'
    return res
  }

  /**
   * 应用纠正建议
   *
   * 对标 nrs.recipient.js:195-197 correctAddressMistake：
   * 点击纠正建议后，将建议地址填入输入框并重新校验。
   *
   * @param suggestedAddress 建议的地址
   * @param options 校验选项（同 checkRecipient）
   * @returns 重新校验的结果
   */
  async function correctAddressMistake(
    suggestedAddress: string,
    options: { selfAccountRS?: string; requestType?: string } = {},
  ): Promise<RecipientCheckResult | null> {
    return checkRecipient(suggestedAddress, options)
  }

  /**
   * 清空校验结果
   */
  function reset(): void {
    result.value = null
    status.value = 'idle'
  }

  return {
    result,
    status,
    checkRecipient,
    correctAddressMistake,
    getAccountError,
    checkRecipientAlias,
    checkForMerchant,
    reset,
  }
}

/**
 * 反转义服务端响应字符串
 *
 * 对标 nrs.js 的 NRS.unescapeRespStr：将 &amp; &lt; &gt; &quot; &#39; 反转义。
 *
 * @param s 服务端返回的字符串
 * @returns 反转义后的字符串
 */
function unescapeRespStr(s: string): string {
  if (!s) return ''
  return String(s)
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
}
