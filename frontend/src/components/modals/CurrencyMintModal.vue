<template>
  <el-dialog
    v-model="visible"
    :title="t('monetary.mintCurrencyTitle', { code: currency?.code || '' })"
    width="560px"
    :close-on-click-modal="false"
    destroy-on-close
    class="nrcs-modal"
    @close="handleClose"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <!-- 货币信息（对标 mint_currency_modal 显示） -->
      <el-descriptions :column="2" border size="small" class="mb-16">
        <el-descriptions-item :label="t('monetary.code')">{{ currency?.code || '-' }}</el-descriptions-item>
        <el-descriptions-item :label="t('monetary.decimals')">{{ currency?.decimals ?? 0 }}</el-descriptions-item>
        <el-descriptions-item :label="t('monetary.currentSupply')">
          {{ formatQNT(currency?.currentSupply, currency?.decimals ?? 0) }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('monetary.maxSupply')" v-if="currency?.maxSupply">
          {{ formatQNT(currency?.maxSupply, currency?.decimals ?? 0) }}
        </el-descriptions-item>
      </el-descriptions>

      <el-form-item :label="t('monetary.mintUnits')" prop="units">
        <el-input
          v-model="form.units"
          type="number"
          :placeholder="t('monetary.mintUnitsPlaceholder')"
          clearable
          :disabled="minting"
        >
          <template #append>{{ currency?.code || 'UNITS' }}</template>
        </el-input>
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('common.fee')" prop="feeNXT">
            <el-input v-model="form.feeNXT" placeholder="1" type="number" clearable :disabled="minting">
              <template #append>NRC</template>
            </el-input>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item :label="t('common.deadline')" prop="deadline">
            <el-input v-model="form.deadline" placeholder="1440" type="number" clearable :disabled="minting">
              <template #append>{{ t('common.minutes') }}</template>
            </el-input>
          </el-form-item>
        </el-col>
      </el-row>
      <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable :disabled="minting" />
      </el-form-item>

      <!-- 工作量证明进度（对标 mintingInProgress） -->
      <el-progress v-if="minting" :percentage="progress" :status="progressStatus" :indeterminate="progress === 0" />
      <el-alert v-if="mintingInfo" :title="mintingInfo" type="info" :closable="false" show-icon class="mt-8" />
    </el-form>
    <template #footer>
      <el-button @click="handleClose" :disabled="minting">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading || minting" @click="handleSubmit">
        {{ t('monetary.mint') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * CurrencyMintModal.vue —— 铸造货币（currencyMint）弹窗。
 *
 * 对标参考：
 *   - `nrs.server.js:1218` 的 currencyMint 交易验证（type=5, subtype=7）
 *   - `CurrencyMinting.java` 的工作量证明算法
 *   - `GetMintingTarget.java` 的 getMintingTarget API
 *
 * 工作量证明流程（对标 CurrencyMinting.getHash + meetsTarget）：
 *   1. 调用 `getMintingTarget(currency, account, unitsQNT)` 获取 { targetBytes, counter, difficulty }
 *   2. 生成随机 nonce（8 字节小端序）
 *   3. 循环递增 counter，计算 hash(nonce || currencyId || units || counter || accountId)
 *   4. 比较 hash 与 target（从高位字节开始，hash <= target 即满足）
 *   5. 满足后通过 `submitForm('currencyMint', { currency, nonce, units, counter, ... })` 本地签名
 *
 * 哈希算法由货币的 algorithm 字段指定（对标 HashFunction.getHashFunction）：
 *   - SHA256 = 2（CryptoJS 支持）
 *   - SHA3 = 3（CryptoJS 支持）
 *   - SCRYPT = 5（需额外库，暂不支持）
 *   - Keccak25 = 25（需额外库，暂不支持）
 *
 * 仅当 `isMintable(type)` 时可操作（对标参考 disabled 条件）。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import CryptoJS from 'crypto-js'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { qntToQntf, qntfToQnt } from '@/utils/format'
import { convertRSToNumericAccount } from '@/utils/nrs-address'

const props = defineProps<{ currency: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const formRef = ref<FormInstance>()
const loading = ref(false)
const minting = ref(false)
const progress = ref(0)
const progressStatus = ref<'success' | 'warning' | 'exception' | undefined>(undefined)
const mintingInfo = ref('')
const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

/** 哈希算法常量（对标 constants.js mintingHashAlgorithms） */
const HASH_ALGO = {
  SHA256: 2,
  SHA3: 3,
  SCRYPT: 5,
  KECCAK25: 25,
} as const

const form = reactive({
  units: '',
  feeNXT: '1',
  deadline: '1440',
  secretPhrase: '',
})

const rules = computed<FormRules>(() => ({
  units: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  feeNXT: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

/**
 * 打开时重置表单（对标 mint_currency_modal show.bs.modal）。
 */
watch(visible, (val) => {
  if (!val) return
  form.units = ''
  form.feeNXT = '1'
  form.deadline = '1440'
  form.secretPhrase = accountStore.secretPhrase || ''
  minting.value = false
  progress.value = 0
  progressStatus.value = undefined
  mintingInfo.value = ''
})

/** 格式化 QNT 数量（对标 NRS.formatQuantity） */
function formatQNT(qnt: string | undefined, decimals: number): string {
  if (!qnt) return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

/**
 * 将 64 位无符号整数转为 8 字节小端序 Uint8Array（对标 ByteBuffer.putLong + LITTLE_ENDIAN）。
 */
function u64ToBytesLE(value: bigint): Uint8Array {
  const buf = new Uint8Array(8)
  let v = value
  for (let i = 0; i < 8; i++) {
    buf[i] = Number(v & 0xffn)
    v >>= 8n
  }
  return buf
}

/**
 * 计算铸造哈希（对标 CurrencyMinting.getHash）。
 *
 * hash = hashFunction(nonce[8 LE] || currencyId[8 LE] || units[8 LE] || counter[8 LE] || accountId[8 LE])
 *
 * @param algorithm 哈希算法（2=SHA256, 3=SHA3, 5=SCRYPT, 25=Keccak25）
 * @param nonce 随机数
 * @param currencyId 货币 ID
 * @param units 铸造单位数（QNT）
 * @param counter 计数器
 * @param accountId 账户 ID
 * @returns 32 字节哈希
 */
function computeMintHash(
  algorithm: number,
  nonce: bigint,
  currencyId: bigint,
  units: bigint,
  counter: bigint,
  accountId: bigint,
): Uint8Array {
  const buffer = new Uint8Array(40)
  buffer.set(u64ToBytesLE(nonce), 0)
  buffer.set(u64ToBytesLE(currencyId), 8)
  buffer.set(u64ToBytesLE(units), 16)
  buffer.set(u64ToBytesLE(counter), 24)
  buffer.set(u64ToBytesLE(accountId), 32)

  // 对标 HashFunction.getHashFunction(algorithm)
  let hashWords: CryptoJS.lib.WordArray
  const wordArray = byteArrayToWordArray(buffer)
  if (algorithm === HASH_ALGO.SHA256) {
    hashWords = CryptoJS.SHA256(wordArray)
  } else if (algorithm === HASH_ALGO.SHA3) {
    hashWords = CryptoJS.SHA3(wordArray, { outputLength: 256 })
  } else {
    throw new Error(`Unsupported hash algorithm: ${algorithm}`)
  }
  return wordArrayToByteArray(hashWords)
}

/**
 * 检查哈希是否满足目标（对标 CurrencyMinting.meetsTarget）。
 *
 * Java 实现：从 hash[31] 到 hash[0] 逐字节比较（小端序比较）。
 * hash[i] <= target[i] 时继续，hash[i] < target[i] 时满足，hash[i] > target[i] 时不满足。
 *
 * @param hash 32 字节哈希
 * @param target 32 字节目标（小端序）
 */
function meetsTarget(hash: Uint8Array, target: Uint8Array): boolean {
  for (let i = hash.length - 1; i >= 0; i--) {
    const h = hash[i] & 0xff
    const tgt = target[i] & 0xff
    if (h > tgt) return false
    if (h < tgt) return true
  }
  return true
}

/** CryptoJS 字节转换工具（对标 nrs.util.js byteArrayToWordArray / wordArrayToByteArray） */
function byteArrayToWordArray(ba: Uint8Array): CryptoJS.lib.WordArray {
  const wa: number[] = []
  for (let i = 0; i < ba.length; i += 4) {
    wa.push(
      ((ba[i] << 24) >>> 0) |
        ((ba[i + 1] << 16) >>> 0) |
        ((ba[i + 2] << 8) >>> 0) |
        (ba[i + 3] >>> 0),
    )
  }
  return CryptoJS.lib.WordArray.create(wa as any, ba.length)
}

function wordArrayToByteArray(wa: CryptoJS.lib.WordArray): Uint8Array {
  const ba: number[] = []
  const words = wa.words
  for (let i = 0; i < wa.sigBytes; i += 4) {
    const word = words[i / 4] >>> 0
    ba.push((word >>> 24) & 0xff)
    ba.push((word >>> 16) & 0xff)
    ba.push((word >>> 8) & 0xff)
    ba.push(word & 0xff)
  }
  return new Uint8Array(ba)
}

/**
 * 将 hex 字符串转为 Uint8Array（对标 converters.hexStringToByteArray）。
 */
function hexToBytes(hex: string): Uint8Array {
  const clean = hex.replace(/^0x/, '')
  const bytes = new Uint8Array(clean.length / 2)
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.substr(i * 2, 2), 16)
  }
  return bytes
}

/**
 * 生成随机 nonce（8 字节，对标参考 Long 随机数）。
 */
function randomNonce(): bigint {
  const buf = new Uint8Array(8)
  if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
    crypto.getRandomValues(buf)
  } else {
    for (let i = 0; i < 8; i++) buf[i] = Math.floor(Math.random() * 256)
  }
  let v = 0n
  for (let i = 7; i >= 0; i--) {
    v = (v << 8n) | BigInt(buf[i])
  }
  return v
}

/**
 * 异步延迟，避免阻塞 UI（对标 Web Worker 的效果）。
 */
function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * 提交铸造（对标 NRS.forms.currencyMint + CurrencyMinting 工作量证明）。
 *
 * 流程：
 *   1. 调用 getMintingTarget 获取 { targetBytes, counter }
 *   2. 生成随机 nonce
 *   3. 循环递增 counter 计算 hash，直到 hash <= target
 *   4. 通过 submitForm('currencyMint', { currency, nonce, units, counter, ... }) 本地签名
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const currencyId = String(props.currency?.currency || '')
      if (!currencyId) throw new Error(t('monetary.pleaseSelectCurrency'))

      const decimals = Number(props.currency?.decimals ?? 0)
      const algorithm = Number(props.currency?.algorithm ?? HASH_ALGO.SHA256)
      const accountRS = accountStore.accountRS
      if (!accountRS) throw new Error(t('common.accountRequired'))

      // QNTf → QNT（对标 NRS.convertToQNT(units, decimals)）
      const unitsQNT = qntfToQnt(form.units, decimals)
      const unitsBig = BigInt(unitsQNT)

      // 将账户 RS 转为数字 ID（对标 ParameterParser.getAccountId）
      let accountIdBig: bigint
      try {
        const numericStr = convertRSToNumericAccount(accountRS)
        accountIdBig = BigInt(numericStr)
        // RS 转换可能返回有符号值，转为无符号
        if (accountIdBig < 0n) accountIdBig += 1n << 64n
      } catch {
        throw new Error(t('validation.invalidAddress'))
      }

      const currencyIdBig = BigInt(currencyId)

      // ── 第 1 步：获取铸造目标（对标 getMintingTarget API） ─────────────
      minting.value = true
      progress.value = 0
      mintingInfo.value = t('monetary.mintingInProgress')
      await delay(50)

      const target = await nrcsApi.getMintingTarget(currencyId, accountRS, unitsQNT)
      if (!target?.targetBytes) {
        throw new Error('getMintingTarget returned empty target')
      }

      const targetBytes = hexToBytes(target.targetBytes)
      let counter = BigInt(target.counter || '0')
      const nonce = randomNonce()

      // ── 第 2 步：工作量证明循环（对标 CurrencyMinting.meetsTarget） ────
      const MAX_ITERATIONS = 5_000_000 // 防止无限循环
      let found = false
      let iterations = 0
      const batchSize = 1000 // 每 1000 次迭代检查 UI

      for (let i = 0; i < MAX_ITERATIONS; i++) {
        const hash = computeMintHash(algorithm, nonce, currencyIdBig, unitsBig, counter, accountIdBig)
        if (meetsTarget(hash, targetBytes)) {
          found = true
          break
        }
        counter++
        iterations++
        // 每 batchSize 次迭代让出 UI 线程并更新进度
        if (iterations % batchSize === 0) {
          progress.value = Math.min(99, Math.floor(iterations / MAX_ITERATIONS * 100))
          await delay(0)
        }
      }

      if (!found) {
        throw new Error(t('monetary.mintError') + ' (proof of work timeout)')
      }

      progress.value = 100
      progressStatus.value = 'success'
      mintingInfo.value = ''
      minting.value = false

      // ── 第 3 步：提交铸造交易（对标 NRS.forms.currencyMint） ────────────
      await submitForm(
        'currencyMint',
        {
          currency: currencyId,
          nonce: nonce.toString(),
          units: unitsQNT,
          counter: counter.toString(),
          feeNXT: form.feeNXT,
          deadline: form.deadline,
          secretPhrase: form.secretPhrase,
        },
        {
          successMessage: t('monetary.mintSuccess'),
        },
      )
      emit('success')
      handleClose()
    } catch (e: any) {
      minting.value = false
      progressStatus.value = 'exception'
      ElMessage.error(e?.message || t('monetary.mintError'))
    } finally {
      loading.value = false
    }
  })
}

function handleClose(): void {
  formRef.value?.resetFields()
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.mb-16 {
  margin-bottom: 16px;
}
.mt-8 {
  margin-top: 8px;
}
</style>
