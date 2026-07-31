/******************************************************************************
 * NRCS Shuffling 交易附件渲染器（type 7）—— 对标 nrs.modals.transaction.js:1213-1324。
 *
 * subtype 列表（参考 NRS.isOfType 判断）：
 *   - 0: ShufflingCreation — 需要拉取 getShufflingParticipants + getShufflers + getShuffling
 *   - 1: ShufflingRegistration
 *   - 2: ShufflingProcessing
 *   - 3: ShufflingRecipients
 *   - 4: ShufflingVerification
 *   - 5: ShufflingCancellation
 ******************************************************************************/
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import type { InfoRow } from './transaction-info-table'
import { mergeMaps } from './transaction-info-table'
import type { RenderOptions, RenderResult } from './transaction-attachment-renderer'
import {
  getTransactionLink,
  getAccountLink,
  getAssetLink,
  getCurrencyLink,
  type LinkRef,
} from './transaction-links'
import { formatAmount, formatQuantity } from './format'

// ============================================================================
// renderShufflingAttachment —— 主入口
// ============================================================================

/**
 * 渲染 Shuffling 类型交易附件（对标 nrs.modals.transaction.js:1213-1324）。
 *
 * @param transaction 交易对象
 * @param subtype 子类型
 * @param options 渲染选项
 * @returns 渲染结果
 */
export async function renderShufflingAttachment(
  transaction: NrcsTransaction,
  subtype: number,
  options: RenderOptions,
): Promise<RenderResult> {
  switch (subtype) {
    // subtype 0: Shuffling 创建
    case 0:
      return renderShufflingCreation(transaction, options)

    // subtype 1: Shuffling 注册
    case 1: {
      const attachment = transaction.attachment || {}
      const data = mergeMaps(attachment, { type: 'shuffling_registration' }, { 'version.ShufflingRegistration': true })
      // createInfoTable 会自动跳过 version.* 字段
      const rows: InfoRow[] = [{ label: 'Type', value: 'shuffling_registration' }]
      return { rows, incorrect: false, async: false }
    }

    // subtype 2: Shuffling 处理
    case 2: {
      const rows: InfoRow[] = [{ label: 'Type', value: 'shuffling_processing' }]
      return { rows, incorrect: false, async: false }
    }

    // subtype 3: Shuffling 接收者
    case 3: {
      const attachment = transaction.attachment || {}
      const rows: InfoRow[] = [
        { label: 'Type', value: 'shuffling_recipients' },
        { label: 'Shuffling State Hash', value: attachment.shufflingStateHash ?? '' },
        { label: 'Shuffling', value: getTransactionLink(attachment.shuffling ?? '0') },
      ]
      // 接收者公钥列表（对标 listPublicKeys :1605-1616）
      if (Array.isArray(attachment.recipientPublicKeys) && attachment.recipientPublicKeys.length > 0) {
        rows.push({
          label: 'Recipients',
          value: attachment.recipientPublicKeys.join(', '),
        })
      }
      return { rows, incorrect: false, async: false }
    }

    // subtype 4: Shuffling 验证
    case 4: {
      const rows: InfoRow[] = [{ label: 'Type', value: 'shuffling_verification' }]
      return { rows, incorrect: false, async: false }
    }

    // subtype 5: Shuffling 取消
    case 5: {
      const rows: InfoRow[] = [{ label: 'Type', value: 'shuffling_cancellation' }]
      return { rows, incorrect: false, async: false }
    }

    default:
      return { rows: [], incorrect: true, async: false }
  }
}

// ============================================================================
// renderShufflingCreation —— 对标 nrs.modals.transaction.js:1213-1294
// ============================================================================

/**
 * 渲染 Shuffling 创建交易（对标 nrs.modals.transaction.js:1213-1294）。
 *
 * 拉取：
 *   - getShufflingParticipants：参与者列表
 *   - getShufflers：本地 shuffler 状态
 *   - getShuffling：shuffling 阶段/计数/分配者等
 *
 * @param transaction 交易对象
 * @param options 渲染选项
 * @returns 渲染结果
 */
async function renderShufflingCreation(
  transaction: NrcsTransaction,
  options: RenderOptions,
): Promise<RenderResult> {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')

  const rows: InfoRow[] = [
    { label: 'Type', value: 'shuffling_creation' },
    { label: 'Period', value: String(attachment.registrationPeriod ?? 0) },
    { label: 'Holding Type', value: String(attachment.holdingType ?? 0) },
    { label: 'Sender', value: senderLink },
  ]

  // holding 和 amount（对标 :1219-1235）
  if (attachment.holding && attachment.holding !== '0') {
    const holdingType = attachment.holdingType
    try {
      if (holdingType === 1) {
        // 资产
        const asset = await nrcsApi.getAsset(attachment.holding)
        rows.push({
          label: 'Holding',
          value: getAssetLink(attachment.holding, asset?.name),
        })
        rows.push({
          label: 'Amount',
          value: { quantity: attachment.amount ?? '0', decimals: asset?.decimals ?? 0 },
        })
      } else if (holdingType === 2) {
        // 货币
        const currency = await nrcsApi.getCurrency(attachment.holding)
        rows.push({
          label: 'Holding',
          value: getCurrencyLink(attachment.holding, currency?.code),
        })
        rows.push({
          label: 'Amount',
          value: { quantity: attachment.amount ?? '0', decimals: currency?.decimals ?? 0 },
        })
      }
    } catch {
      rows.push({ label: 'Holding', value: getTransactionLink(attachment.holding) })
      rows.push({ label: 'Amount', value: attachment.amount ?? '0' })
    }
  } else {
    rows.push({ label: 'Amount', value: attachment.amount ?? '0' })
  }

  // 参与者列表（对标 :1236-1254）
  try {
    const participantsResp = await nrcsApi.getShufflingParticipants(transaction.transaction ?? '')
    if (participantsResp.participants && participantsResp.participants.length > 0) {
      // 简化：展示参与者数量和账户列表（完整表格由组件层渲染）
      const participantLinks: LinkRef[] = participantsResp.participants
        .map((p: any) => getAccountLink(p, 'account'))
        .filter((l): l is LinkRef => typeof l !== 'string')
      if (participantLinks.length > 0) {
        rows.push({ label: 'Participants', value: participantLinks })
      } else {
        rows.push({ label: 'Participants', value: 'no_matching_participants' })
      }
    } else {
      rows.push({ label: 'Participants', value: 'no_matching_participants' })
    }
  } catch {
    // 忽略
  }

  // 本地 shuffler 状态（对标 :1255-1275）
  try {
    const shufflersResp = await nrcsApi.getShufflers(options.currentAccount)
    const shuffler = (shufflersResp as any).shufflers?.find(
      (s: any) => s.shufflingFullHash === transaction.fullHash,
    ) as Record<string, any> | undefined
    if (shuffler) {
      rows.push({ label: 'Shuffler', value: 'running' })
      const recipientLink = getAccountLink(shuffler, 'recipient')
      rows.push({
        label: 'Shuffler Recipient',
        value: typeof recipientLink === 'string' ? recipientLink : recipientLink,
      })
      if (shuffler.failedTransaction) {
        rows.push({
          label: 'Failed Transaction',
          value: getTransactionLink(String(shuffler.recipient ?? '')),
        })
        rows.push({ label: 'Failure Cause', value: String(shuffler.failureCause ?? '') })
      }
    } else {
      rows.push({ label: 'Shuffler', value: (shufflersResp as any).errorCode ? 'unknown' : 'not_started' })
    }
  } catch {
    // 忽略（非本地节点或无权限）
  }

  // shuffling 详情（对标 :1276-1292）
  try {
    const shufflingResp = await nrcsApi.getShuffling(transaction.transaction ?? '') as any
    if (shufflingResp) {
      rows.push({ label: 'Stage', value: getShufflingStageName(shufflingResp.stage) })
      rows.push({
        label: 'Count',
        value: `${shufflingResp.registrantCount ?? 0} / ${shufflingResp.participantCount ?? 0}`,
      })
      rows.push({ label: 'Blocks Remaining', value: String(shufflingResp.blocksRemaining ?? 0) })
      // issuer / assignee 字段不在 getAccountLink 的 field 联合类型中，使用对象透传方式
      const issuerLink = getAccountLink(
        { account: shufflingResp.issuer, accountRS: shufflingResp.issuerRS },
        'account',
      )
      rows.push({ label: 'Issuer', value: issuerLink === '-' ? '-' : issuerLink })
      if (shufflingResp.assignee) {
        const assigneeLink = getAccountLink(
          { account: shufflingResp.assignee, accountRS: shufflingResp.assigneeRS },
          'account',
        )
        rows.push({ label: 'Assignee', value: assigneeLink === '-' ? '-' : assigneeLink })
      }
      if (shufflingResp.shufflingStateHash) {
        rows.push({ label: 'Shuffling State Hash', value: shufflingResp.shufflingStateHash })
      }
    }
  } catch {
    // 忽略
  }

  return { rows, incorrect: false, async: true }
}

// ============================================================================
// 辅助函数
// ============================================================================

/**
 * 获取 Shuffling 阶段名称（对标 NRS.getShufflingStage）。
 *
 * @param stage 阶段编号
 * @returns 阶段名称
 */
function getShufflingStageName(stage?: number): string {
  switch (stage) {
    case 0:
      return 'shuffling_stage_registration'
    case 1:
      return 'shuffling_stage_processing'
    case 2:
      return 'shuffling_stage_blame'
    case 3:
      return 'shuffling_stage_done'
    case 4:
      return 'shuffling_stage_cancelled'
    default:
      return String(stage ?? 0)
  }
}

/**
 * 获取 Shuffling 参与者状态名称（对标 NRS.getShufflingParticipantState）。
 *
 * @param state 状态编号
 * @returns 状态名称
 */
export function getShufflingParticipantStateName(state?: number): string {
  switch (state) {
    case 0:
      return 'participant_state_registered'
    case 1:
      return 'participant_state_processing'
    case 2:
      return 'participant_state_verified'
    case 3:
      return 'participant_state_done'
    default:
      return String(state ?? 0)
  }
}
