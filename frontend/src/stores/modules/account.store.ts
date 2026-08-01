/**
 * NRCS 账户 Store（推荐）
 *
 * 对标参考 nrs.login.js 的登录/账户管理逻辑：
 *   - 本地派生：secretPhrase → publicKey/accountId/accountRS 全部在客户端本地计算，
 *     绝不外发（对标 nrs.login.js 注释 "Processed locally, not submitted to server"）
 *   - 账户占用校验：调 getAccountPublicKey 检查链上公钥与本地派生公钥是否一致
 *     （对标 nrs.login.js:378-388，不一致则报 error_account_taken）
 *   - 多账户管理：savedNrcsAccounts 存 accountRS（不存 secretPhrase），
 *     支持 listAccounts/switchAccount/removeAccount/rememberAccount
 *   - 安全模型：secretPhrase 仅内存暂存，不持久化（对标阶段 0.5）；
 *     rememberMe 只存 accountRS，刷新后为只读模式
 *
 * 复用：
 *   - utils/mnemonic.ts 的 passphraseToAccount（本地派生）
 *   - utils/mnemonic.ts 的 checkPassphraseStrength（密码强度警告）
 *   - api/modules/nrcs.api.ts 的 nrcsApi（getAccount/getAccountPublicKey/getBlockchainStatus）
 */
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { nrcsApi } from '@/api/modules';
import type {
  NrcsAccount,
  NrcsAssetBalance,
  NrcsPhasingOnlyControl,
} from '@/api/modules';
import { passphraseToAccount, checkPassphraseStrength } from '@/utils/mnemonic';
import { storageSelect, storageInsert, storageUpdate, initUserDB } from '@/utils/nrcs-storage';

// localStorage 键（对标参考命名，同时与现有数据兼容）
const STORAGE_KEY_ACCOUNT_RS = 'nrcs-accountRS';
const STORAGE_KEY_ACCOUNT_ID = 'nrcs-accountId';
const STORAGE_KEY_PUBLIC_KEY = 'nrcs-publicKey';
const STORAGE_KEY_SAVED_ACCOUNTS = 'saved_nrcs_accounts'; // 分号分隔的 accountRS 列表
const STORAGE_KEY_LOGGED_IN = 'logged_in'; // 对标 nrs.login.js:594
const STORAGE_KEY_LOGIN_TYPE = 'login_type'; // 'password' | 'account'
// ⛔ 已废弃：secretPhrase 不再持久化（阶段 0.5 安全修复）
// const STORAGE_KEY_SECRET_PHRASE = 'nrcs-secretPhrase';

/** Java Integer.MAX_VALUE（对标 NRS.constants.MAX_INT_JAVA，nrs.js:1398/1458 用） */
const MAX_INT_JAVA = 2147483647;

/**
 * 出租状态计算结果（对标 nrs.js:1394-1489 updateAccountLeasingStatus 的输出）。
 */
export interface LeasingStatus {
  /** 出租标签：leased_out / leased_soon / 空（对标 nrs.js:1407/1415 accountLeasingLabel） */
  label: 'leased_out' | 'leased_soon' | '';
  /** 本地化状态消息（对标 nrs.js:1408/1416/1424 accountLeasingStatus） */
  statusMessage: string;
  /** 下一轮承租方信息（对标 nrs.js:1399-1404 nextLesseeStatus） */
  nextLesseeStatus: string;
  /** 出租方数量（对标 nrs.js:1439 lessors.length） */
  lessorCount: number;
}

/**
 * 资产余额变化（对标 nrs.js:1533-1645 checkAssetDifferences 的输出）。
 */
export interface AssetDifference {
  /** 资产 ID */
  asset: string;
  /** 变化量（正数=收到，负数=卖出，对标 nrs.js:1562/1564/1573 diff[k]） */
  difference: string;
}

export const useAccountStore = defineStore('account', () => {
  // --- State ---
  const accountRS = ref<string>('');
  const accountId = ref<string>('');
  const publicKey = ref<string>('');
  const balanceNQT = ref<string>('0');
  const effectiveBalance = ref<number>(0);
  const unconfirmedBalanceNQT = ref<string>('0');
  const forgedBalanceNQT = ref<string>('0');
  const guaranteedBalanceNQT = ref<string>('0');
  const name = ref<string>('');
  const description = ref<string>('');

  /** secretPhrase 仅内存暂存，绝不持久化（阶段 0.5 安全要求） */
  const secretPhrase = ref<string>('');

  /** 是否为测试网（登录时从 getBlockchainStatus 获取） */
  const isTestNet = ref<boolean>(false);

  /** lockscreen 锁定状态 */
  const isLocked = ref<boolean>(false);

  /** 登录方式：'password'（助记词/密码短语）| 'account'（只读 RS 地址） */
  const loginType = ref<'password' | 'account'>('account');

  /** 登录后的密码强度警告（对标 nrs.login.js:415-425 的 passwordNotice） */
  const loginWarning = ref<string>('');

  /** 已保存的账户 RS 列表（多账户快速切换） */
  const savedAccounts = ref<string[]>([]);

  // === 阶段 1.2 新增：完整账户信息与状态（对标 nrs.js:1101 getAccountInfo） ===

  /** 完整账户信息（对标 NRS.accountInfo，nrs.js:1110） */
  const accountInfo = ref<NrcsAccount | null>(null);

  /** 出租状态计算结果（对标 nrs.js:1394 updateAccountLeasingStatus 输出） */
  const leasingStatus = ref<LeasingStatus>({
    label: '',
    statusMessage: '',
    nextLesseeStatus: '',
    lessorCount: 0,
  });

  /** Phasing Only 控制详情（对标 nrs.js:1505 NRS.accountInfo.phasingOnly） */
  const phasingOnlyControl = ref<NrcsPhasingOnlyControl | null>(null);

  /** 是否设置了 PHASING_ONLY 控制（对标 nrs.js:1497 检测） */
  const hasPhasingOnlyControl = ref<boolean>(false);

  /** 上一次的资产余额（用于 checkAssetDifferences 对比，对标 nrs.js:1137 previous_balances） */
  const previousAssetBalances = ref<NrcsAssetBalance[]>([]);

  /** 资产余额变化通知回调（由 notifications 模块注册） */
  const assetDifferenceCallback = ref<((differences: AssetDifference[]) => void) | null>(null);

  // --- Getters ---
  const isLoggedIn = computed(() => !!accountRS.value);
  const balanceFormatted = computed(() => {
    const nqt = BigInt(balanceNQT.value || '0');
    return Number(nqt) / 100000000;
  });
  /** 是否拥有 secretPhrase（可签名/锻造）；只读账户为 false */
  const hasSecretPhrase = computed(() => !!secretPhrase.value);

  // --- 本地派生与校验 ---

  /**
   * 校验链上公钥与本地派生公钥是否一致（账户占用校验）
   *
   * 对标 nrs.login.js:378-388：若链上已存在公钥且与本地派生不一致，
   * 说明该 secretPhrase 不属于此账户（账户被占用），拒绝登录。
   *
   * @param derivedAccountId 本地派生的数字账户 ID
   * @param derivedPublicKey 本地派生的十六进制公钥
   * @throws 当链上公钥存在且不匹配时抛出"账户已被占用"错误
   */
  async function verifyAccountNotTaken(
    derivedAccountId: string,
    derivedPublicKey: string,
  ): Promise<void> {
    try {
      const resp = await nrcsApi.getAccountPublicKey(derivedAccountId);
      // 链上公钥存在且与本地派生不一致 → 账户被占用
      if (resp && resp.publicKey && resp.publicKey !== derivedPublicKey) {
        throw new Error('账户已被其他密码短语占用（error_account_taken）');
      }
    } catch (e: any) {
      // code=5 表示账户不存在（未在链上发送过交易），属新账户，接受登录
      if (e.code === 5) {
        return;
      }
      // 如果是主动抛出的"账户被占用"错误，向上传播
      if (e.message && e.message.includes('账户已被其他密码短语占用')) {
        throw e;
      }
      // 其他错误（网络等）：不阻断登录，仅记录警告
      console.warn('[account] getAccountPublicKey 校验失败，跳过占用校验:', e);
    }
  }

  /**
   * 拉取并填充账户链上信息（余额、名称、描述等）
   *
   * 对标 nrs.login.js:426 的 getAccountInfo。账户不存在时余额置 0。
   *
   * @param accountRsOrId RS 地址或数字账户 ID
   */
  async function fetchAccountInfo(accountRsOrId: string): Promise<void> {
    try {
      const accountInfo: NrcsAccount = await nrcsApi.getAccount(accountRsOrId);
      balanceNQT.value = accountInfo.balanceNQT || '0';
      unconfirmedBalanceNQT.value = accountInfo.unconfirmedBalanceNQT || '0';
      forgedBalanceNQT.value = accountInfo.forgedBalanceNQT || '0';
      guaranteedBalanceNQT.value = accountInfo.guaranteedBalanceNQT || '0';
      effectiveBalance.value = accountInfo.effectiveBalanceNRCS || 0;
      name.value = accountInfo.name || '';
      description.value = accountInfo.description || '';
    } catch {
      // 账户可能尚未在链上创建（未发送过交易），余额置 0
      balanceNQT.value = '0';
      unconfirmedBalanceNQT.value = '0';
      forgedBalanceNQT.value = '0';
      guaranteedBalanceNQT.value = '0';
      effectiveBalance.value = 0;
      name.value = '';
      description.value = '';
    }
  }

  /**
   * 检测节点是否为测试网（对标 nrs.login.js:319 的 getBlockchainStatus）
   *
   * 失败时不阻断登录，默认 mainnet。
   */
  async function detectTestNet(): Promise<void> {
    try {
      const status = await nrcsApi.getBlockchainStatus();
      isTestNet.value = !!(status as any)?.isTestnet;
    } catch (e) {
      console.warn('[account] getBlockchainStatus 失败，默认 mainnet:', e);
      isTestNet.value = false;
    }
  }

  /**
   * 评估密码短语强度并生成警告信息
   *
   * 对标 nrs.login.js:415-425：
   *   - 长度 < 35：error_passphrase_length_secure
   *   - 长度 < 50 且无大写字母或数字：error_passphrase_strength_secure
   *
   * @param password 密码短语
   * @returns 警告信息（无警告时为空字符串）
   */
  function evaluatePassphraseWarning(password: string): string {
    if (password.length < 35) {
      return '密码短语长度不足 35 字符，安全性较低';
    }
    if (password.length < 50 && (!/[A-Z]/.test(password) || !/[0-9]/.test(password))) {
      return '密码短语少于 50 字符且缺少大写字母或数字，安全性较低';
    }
    return '';
  }

  // --- Actions ---

  /**
   * 使用密码短语（助记词）登录
   *
   * 安全模型：secretPhrase 仅在客户端本地派生 publicKey/accountId/accountRS，
   * 绝不发送到服务端（对标 nrs.login.js 中 getAccountId "Processed locally"）。
   * 派生后校验账户未被占用，再拉取链上信息。
   *
   * @param password 密码短语或 12 词助记词
   * @param options.rememberMe 是否记住账户（只存 accountRS，不存 secretPhrase）
   * @returns 登录结果，含可能的密码强度警告 warning
   */
  async function login(
    password: string,
    options?: { rememberMe?: boolean },
  ): Promise<{ warning?: string }> {
    if (!password || !password.trim()) {
      throw new Error('密码短语不能为空');
    }

    const trimmed = password.trim();

    // 第 1 步：本地派生账户（secretPhrase 不出客户端）
    const derived = passphraseToAccount(trimmed);

    // 第 2 步：账户占用校验（链上公钥与本地派生公钥一致性）
    await verifyAccountNotTaken(derived.accountId, derived.publicKey);

    // 第 3 步：填充状态
    accountRS.value = derived.accountRS;
    accountId.value = derived.accountId;
    publicKey.value = derived.publicKey;
    secretPhrase.value = trimmed; // 仅内存暂存
    loginType.value = 'password';
    loginWarning.value = evaluatePassphraseWarning(trimmed);

    // 第 4 步：初始化账户级 IndexedDB（账户隔离，对标 nrs.localstorage.js initUserDB）
    await initUserDB(derived.accountId);

    // 第 5 步：检测测试网 + 拉取链上信息（并行，失败不阻断）
    await Promise.all([detectTestNet(), fetchAccountInfo(derived.accountRS)]);

    // 第 6 步：持久化只读状态（不持久化 secretPhrase）
    persistReadOnlyState();

    if (options?.rememberMe) {
      rememberAccount(derived.accountRS);
    }

    return { warning: loginWarning.value || undefined };
  }

  /**
   * 使用 RS 账户地址只读登录（watch-only 模式）
   *
   * 对标 nrs.login.js 的 login(false, account)：不存储 secretPhrase，
   * 仅查看账户余额与历史。发送交易时需重新输入 secretPhrase。
   *
   * @param accountRs RS 地址（NRCS-XXXX-XXXX-XXXX-XXXXX）或数字账户 ID
   * @param options.rememberMe 是否记住账户
   */
  async function loginByAccount(
    accountRs: string,
    options?: { rememberMe?: boolean },
  ): Promise<void> {
    if (!accountRs || !accountRs.trim()) {
      throw new Error('账户地址不能为空');
    }

    const trimmed = accountRs.trim();

    // 通过 getAccount 获取链上账户信息（含 accountRS/account/publicKey）
    let accountInfo: NrcsAccount;
    try {
      accountInfo = await nrcsApi.getAccount(trimmed);
    } catch (e: any) {
      if (e.code === 5) {
        throw new Error('区块链上未找到该账户，请检查地址是否正确');
      }
      throw e;
    }

    if (!accountInfo || !accountInfo.accountRS) {
      throw new Error('服务器返回了无效的账户响应');
    }

    accountRS.value = accountInfo.accountRS;
    accountId.value = accountInfo.account;
    publicKey.value = accountInfo.publicKey || '';
    balanceNQT.value = accountInfo.balanceNQT || '0';
    unconfirmedBalanceNQT.value = accountInfo.unconfirmedBalanceNQT || '0';
    forgedBalanceNQT.value = accountInfo.forgedBalanceNQT || '0';
    guaranteedBalanceNQT.value = accountInfo.guaranteedBalanceNQT || '0';
    effectiveBalance.value = accountInfo.effectiveBalanceNRCS || 0;
    name.value = accountInfo.name || '';
    description.value = accountInfo.description || '';
    secretPhrase.value = ''; // 只读模式无 secretPhrase
    loginType.value = 'account';
    loginWarning.value = '';

    // 初始化账户级 IndexedDB（账户隔离，对标 nrs.localstorage.js initUserDB）
    await initUserDB(accountInfo.account);

    await detectTestNet();

    persistReadOnlyState();

    if (options?.rememberMe) {
      rememberAccount(accountInfo.accountRS);
    }
  }

  /**
   * 刷新账户链上数据（余额、名称等）
   */
  async function refreshAccount(): Promise<void> {
    if (!accountRS.value) return;
    await getAccountInfo(accountRS.value, false);
  }

  // === 阶段 1.2：完整账户信息与状态（对标 nrs.js:1101-1645） ===

  /**
   * 拉取完整账户信息并更新 leasing/control/asset 状态。
   *
   * 对标 nrs.js:1101-1325 NRS.getAccountInfo：
   *   1. 调 getAccount（includeAssets/includeCurrencies/includeLessors/includeEffectiveBalance）
   *   2. 保存 accountInfo（对标 nrs.js:1110 NRS.accountInfo = response）
   *   3. 检查 RS 地址一致性（对标 nrs.js:1116-1121）
   *   4. 从 IndexedDB 加载 previous asset_balances，对比触发 checkAssetDifferences（对标 nrs.js:1133-1163）
   *   5. 调用 updateAccountLeasingStatus（对标 nrs.js:1308）
   *   6. 调用 updateAccountControlStatus（对标 nrs.js:1309）
   *
   * @param accountRsOrId RS 地址或数字账户 ID
   * @param isAccountSwitch 是否为账户切换（对标 nrs.js:1128 isAccountSwitch，影响资产差异通知）
   * @param lastBlockHeight 当前区块高度（用于 leasing 状态计算，来自 node.store）
   */
  async function getAccountInfo(
    accountRsOrId: string,
    isAccountSwitch: boolean = false,
    lastBlockHeight: number = 0,
  ): Promise<void> {
    try {
      // 对标 nrs.js:1102-1108 getAccount with includeAssets/includeCurrencies/includeLessors/includeEffectiveBalance
      const response: NrcsAccount = await nrcsApi.getAccount(accountRsOrId, {
        includeLessors: true,
        includeAssets: true,
        includeCurrencies: true,
        includeEffectiveBalance: true,
      });

      // 对标 nrs.js:1110 NRS.accountInfo = response
      accountInfo.value = response;

      // 对标 nrs.js:1116-1121 检查 RS 地址一致性
      if (accountRS.value && response.accountRS && response.accountRS !== accountRS.value) {
        console.warn(
          '[account] 链上 RS 地址与本地不一致，采用链上版本:',
          response.accountRS,
        );
        accountRS.value = response.accountRS;
      }

      // 更新基础字段（兼容旧 fetchAccountInfo 的字段填充）
      balanceNQT.value = response.balanceNQT || '0';
      unconfirmedBalanceNQT.value = response.unconfirmedBalanceNQT || '0';
      forgedBalanceNQT.value = response.forgedBalanceNQT || '0';
      guaranteedBalanceNQT.value = response.guaranteedBalanceNQT || '0';
      effectiveBalance.value = response.effectiveBalanceNRCS || 0;
      name.value = response.name || '';
      description.value = response.description || '';

      // 对标 nrs.js:1133-1163 资产余额对比（非账户切换时）
      if (!isAccountSwitch && response.assetBalances) {
        await compareAndUpdateAssetBalances(response.assetBalances, isAccountSwitch);
      } else if (response.assetBalances) {
        // 账户切换时仅更新存储，不触发通知
        previousAssetBalances.value = response.assetBalances;
      }

      // 对标 nrs.js:1308 updateAccountLeasingStatus
      updateAccountLeasingStatus(lastBlockHeight);

      // 对标 nrs.js:1309 updateAccountControlStatus
      await updateAccountControlStatus();
    } catch (e: any) {
      // 对标 nrs.js:1111-1114 errorCode 分支：账户不存在时清零
      if (e?.code === 5) {
        balanceNQT.value = '0';
        unconfirmedBalanceNQT.value = '0';
        forgedBalanceNQT.value = '0';
        guaranteedBalanceNQT.value = '0';
        effectiveBalance.value = 0;
        name.value = '';
        description.value = '';
        accountInfo.value = null;
        phasingOnlyControl.value = null;
        hasPhasingOnlyControl.value = false;
      } else {
        // 其他错误降级为 fetchAccountInfo（简版）
        console.warn('[account] getAccountInfo 失败，降级为简版:', e);
        await fetchAccountInfo(accountRsOrId);
      }
    }
  }

  /**
   * 对比新旧资产余额并触发通知（对标 nrs.js:1133-1163 + 1533-1645）。
   *
   * 从 IndexedDB 加载上一次的 asset_balances，与当前对比：
   *   - 若有变化且非账户切换，调用 checkAssetDifferences 生成通知
   *   - 更新 IndexedDB 中的 asset_balances
   *
   * @param currentBalances 当前资产余额
   * @param isAccountSwitch 是否为账户切换
   */
  async function compareAndUpdateAssetBalances(
    currentBalances: NrcsAssetBalance[],
    isAccountSwitch: boolean,
  ): Promise<void> {
    try {
      // 对标 nrs.js:1133-1135 storageSelect("data", [{id: "asset_balances"}])
      const stored = await storageSelect<{ id: string; contents: string }>('data', [
        { id: 'asset_balances' },
      ]);

      if (stored && stored.length > 0) {
        const previousStr = stored[0].contents;
        const currentStr = JSON.stringify(currentBalances);

        // 对标 nrs.js:1142 previous_balances != current_balances
        if (previousStr !== currentStr) {
          let previousBalances: NrcsAssetBalance[] = [];
          if (previousStr && previousStr !== 'undefined') {
            try {
              previousBalances = JSON.parse(previousStr);
            } catch {
              previousBalances = [];
            }
          }

          // 对标 nrs.js:1148-1152 storageUpdate
          await storageUpdate(
            'data',
            { contents: currentStr },
            [{ id: 'asset_balances' }],
          );

          previousAssetBalances.value = previousBalances;

          // 对标 nrs.js:1153-1155 showAssetDifference && checkAssetDifferences
          if (!isAccountSwitch) {
            checkAssetDifferences(currentBalances, previousBalances);
          }
        }
      } else {
        // 对标 nrs.js:1157-1162 storageInsert（首次存储）
        await storageInsert('data', 'id', {
          id: 'asset_balances',
          contents: JSON.stringify(currentBalances),
        });
        previousAssetBalances.value = currentBalances;
      }
    } catch (e) {
      console.warn('[account] 资产余额对比失败:', e);
    }
  }

  /**
   * 计算账户出租状态（对标 nrs.js:1394-1489 NRS.updateAccountLeasingStatus）。
   *
   * 根据 currentLeasingHeightFrom/To 和 lastBlockHeight 判断：
   *   - lastBlockHeight >= currentLeasingHeightFrom → leased_out（已出租，剩余 blocks = To - height）
   *   - lastBlockHeight < currentLeasingHeightTo → leased_soon（即将出租，剩余 blocks = From - height）
   *   - 否则 → not_leased_out（未出租）
   *
   * @param lastBlockHeight 当前区块高度（来自 node.store.lastBlockHeight）
   */
  function updateAccountLeasingStatus(lastBlockHeight: number): void {
    const info = accountInfo.value;
    if (!info) {
      leasingStatus.value = {
        label: '',
        statusMessage: '',
        nextLesseeStatus: '',
        lessorCount: 0,
      };
      return;
    }

    // 对标 nrs.js:1406/1414/1423 的条件判断
    // 注意：JS 中 undefined 与数字比较返回 false，TS 中需显式检查 undefined
    const from = info.currentLeasingHeightFrom;
    const to = info.currentLeasingHeightTo;
    const lesseeRS = info.currentLesseeRS || '';

    let label: LeasingStatus['label'] = '';
    let statusMessage = '';

    // 对标 nrs.js:1406-1413 lastBlockHeight >= currentLeasingHeightFrom → leased_out
    if (from !== undefined && lastBlockHeight >= from) {
      label = 'leased_out';
      const blocksLeft = (to ?? 0) - lastBlockHeight;
      statusMessage = `余额已出租，剩余 ${blocksLeft} 块（至高度 ${to}，承租方 ${lesseeRS}）`;
    }
    // 对标 nrs.js:1414-1422 lastBlockHeight < currentLeasingHeightTo → leased_soon
    // NRS 中文翻译（zh-cn/translation.json:135）："您的帐户有效余额将会出租给帐户 __account__ __blocks__ 区块"
    // 标签（zh-cn/translation.json:573）："leased_soon" → "将要出租"
    else if (to !== undefined && lastBlockHeight < to) {
      label = 'leased_soon';
      const blocksUntilLease = (from ?? 0) - lastBlockHeight;
      statusMessage = `余额即将出租，还有 ${blocksUntilLease} 块（从 ${from} 到 ${to}，承租方 ${lesseeRS}）`;
    }
    // 对标 nrs.js:1423-1425 else → not_leased_out
    else {
      statusMessage = '余额未出租';
    }

    // 对标 nrs.js:1398-1404 nextLesseeStatus
    let nextLesseeStatus = '';
    const nextFrom = info.nextLeasingHeightFrom ?? MAX_INT_JAVA;
    if (nextFrom < MAX_INT_JAVA) {
      const nextTo = info.nextLeasingHeightTo ?? MAX_INT_JAVA;
      nextLesseeStatus = `下一承租方：从 ${nextFrom} 到 ${nextTo}`;
    }

    // 对标 nrs.js:1433-1444 lessors
    const lessorCount = info.lessors?.length ?? 0;

    leasingStatus.value = {
      label,
      statusMessage,
      nextLesseeStatus,
      lessorCount,
    };
  }

  /**
   * 检测账户 Phasing Only 控制状态（对标 nrs.js:1491-1531 NRS.updateAccountControlStatus）。
   *
   * 检查 accountControls 是否包含 'PHASING_ONLY'：
   *   - 若是，调 getPhasingOnlyControl 获取详情并存储
   *   - 若否，清除 phasingOnlyControl
   */
  async function updateAccountControlStatus(): Promise<void> {
    const info = accountInfo.value;
    if (!info) {
      phasingOnlyControl.value = null;
      hasPhasingOnlyControl.value = false;
      return;
    }

    // 对标 nrs.js:1497 accountControls 包含 'PHASING_ONLY'
    const controls = info.accountControls || [];
    if (controls.includes('PHASING_ONLY')) {
      try {
        // 对标 nrs.js:1498-1500 getPhasingOnlyControl
        const response = await nrcsApi.getPhasingOnlyControl(accountId.value || accountRS.value);

        // 对标 nrs.js:1501 response.votingModel >= 0
        if (response && response.votingModel >= 0) {
          phasingOnlyControl.value = response;
          hasPhasingOnlyControl.value = true;
          // 同步到 accountInfo.phasingOnly（对标 nrs.js:1505）
          accountInfo.value = {
            ...info,
            phasingOnly: response,
          };
        } else {
          // 对标 nrs.js:1524-1526 onNoPhasingOnly
          clearPhasingOnly();
        }
      } catch (e) {
        console.warn('[account] getPhasingOnlyControl 失败:', e);
        clearPhasingOnly();
      }
    } else {
      // 对标 nrs.js:1528-1530 onNoPhasingOnly
      clearPhasingOnly();
    }
  }

  /**
   * 清除 Phasing Only 控制状态（对标 nrs.js:1492-1496 onNoPhasingOnly）。
   */
  function clearPhasingOnly(): void {
    phasingOnlyControl.value = null;
    hasPhasingOnlyControl.value = false;
    if (accountInfo.value) {
      const { phasingOnly: _omit, ...rest } = accountInfo.value;
      accountInfo.value = rest as NrcsAccount;
    }
  }

  /**
   * 对比新旧资产余额并生成变化通知（对标 nrs.js:1533-1645 NRS.checkAssetDifferences）。
   *
   * 算法：
   *   1. 将新旧余额转为 map（asset → balanceQNT）
   *   2. 遍历旧余额：删除/变化 → diff
   *   3. 遍历新余额：新增 → diff
   *   4. 若 diff 数量 <= 3，逐个生成通知；> 3 则生成"多个资产变动"通知
   *
   * @param currentBalances 当前资产余额
   * @param previousBalances 上一次的资产余额
   */
  function checkAssetDifferences(
    currentBalances: NrcsAssetBalance[],
    previousBalances: NrcsAssetBalance[],
  ): void {
    // 对标 nrs.js:1534-1553 构建 map
    const previousMap: Record<string, string> = {};
    const currentMap: Record<string, string> = {};

    if (previousBalances && previousBalances.length) {
      for (const item of previousBalances) {
        previousMap[item.asset] = item.balanceQNT;
      }
    }

    if (currentBalances && currentBalances.length) {
      for (const item of currentBalances) {
        currentMap[item.asset] = item.balanceQNT;
      }
    }

    // 对标 nrs.js:1555-1575 计算 diff
    const diff: AssetDifference[] = [];

    // 遍历旧余额：删除或变化
    for (const asset in previousMap) {
      if (!(asset in currentMap)) {
        // 对标 nrs.js:1561-1562 资产被移除
        diff.push({ asset, difference: '-' + previousMap[asset] });
      } else if (previousMap[asset] !== currentMap[asset]) {
        // 对标 nrs.js:1563-1564 余额变化（BigNumber 减法）
        const diffValue = bigNumberSubtract(currentMap[asset], previousMap[asset]);
        diff.push({ asset, difference: diffValue });
      }
    }

    // 遍历新余额：新增
    for (const asset in currentMap) {
      if (!(asset in previousMap)) {
        // 对标 nrs.js:1572-1573 新增资产
        diff.push({ asset, difference: currentMap[asset] });
      }
    }

    // 对标 nrs.js:1577-1644 生成通知
    if (diff.length === 0) {
      return;
    }

    // 通过回调通知外部模块（notifications 模块注册）
    if (assetDifferenceCallback.value) {
      assetDifferenceCallback.value(diff);
    }
  }

  /**
   * BigNumber 减法（对标 nrs.js:1564 BigInteger.subtract）。
   *
   * 使用 BigInt 处理大数减法，支持负数结果。
   *
   * @param current 当前余额 QNT
   * @param previous 上一次余额 QNT
   * @returns 差值字符串（可能为负）
   */
  function bigNumberSubtract(current: string, previous: string): string {
    try {
      const cur = BigInt(current || '0');
      const prev = BigInt(previous || '0');
      return (cur - prev).toString();
    } catch {
      // 降级：字符串无法解析为 BigInt 时返回 "0"
      return '0';
    }
  }

  /**
   * 注册资产余额变化回调（由 notifications 模块注册，对标 nrs.js:1584-1638 的通知生成）。
   *
   * @param cb 回调函数，接收 AssetDifference 数组
   */
  function onAssetDifference(cb: (differences: AssetDifference[]) => void): void {
    assetDifferenceCallback.value = cb;
  }

  /**
   * 登出 — 清除当前会话状态（不清除已保存账户列表）
   *
   * 对标 nrs.login.js 的 logout。secretPhrase 从内存清除。
   */
  function logout(): void {
    accountRS.value = '';
    accountId.value = '';
    publicKey.value = '';
    balanceNQT.value = '0';
    effectiveBalance.value = 0;
    unconfirmedBalanceNQT.value = '0';
    forgedBalanceNQT.value = '0';
    guaranteedBalanceNQT.value = '0';
    name.value = '';
    description.value = '';
    secretPhrase.value = '';
    loginWarning.value = '';
    isLocked.value = false;
    // 阶段 1.2：清除完整账户信息与状态
    accountInfo.value = null;
    leasingStatus.value = { label: '', statusMessage: '', nextLesseeStatus: '', lessorCount: 0 };
    phasingOnlyControl.value = null;
    hasPhasingOnlyControl.value = false;
    previousAssetBalances.value = [];

    clearSessionStorage();
  }

  // --- 多账户管理（对标 nrs.login.js:168-289） ---

  /**
   * 从 localStorage 加载已保存的账户 RS 列表
   *
   * 对标 nrs.login.js 的 listAccounts：savedNxtAccounts 以分号分隔。
   */
  function listSavedAccounts(): string[] {
    const raw = localStorage.getItem(STORAGE_KEY_SAVED_ACCOUNTS) || '';
    const accounts = raw
      .split(';')
      .map((a) => a.trim())
      .filter((a) => a.length > 0);
    savedAccounts.value = accounts;
    return accounts;
  }

  /**
   * 记住账户（加入已保存列表，去重）
   *
   * 对标 nrs.login.js:278-289 的 rememberAccount：仅存 accountRS，不存 secretPhrase。
   *
   * @param accountRs RS 地址
   */
  function rememberAccount(accountRs: string): void {
    if (!accountRs) return;
    const accounts = listSavedAccounts();
    if (accounts.includes(accountRs)) return;
    accounts.push(accountRs);
    localStorage.setItem(STORAGE_KEY_SAVED_ACCOUNTS, accounts.join(';'));
    savedAccounts.value = accounts;
  }

  /**
   * 移除已保存的账户
   *
   * 对标 nrs.login.js:268-276 的 removeAccount。
   *
   * @param accountRs RS 地址
   */
  function removeAccount(accountRs: string): void {
    const accounts = listSavedAccounts().filter((a) => a !== accountRs);
    if (accounts.length === 0) {
      localStorage.removeItem(STORAGE_KEY_SAVED_ACCOUNTS);
    } else {
      localStorage.setItem(STORAGE_KEY_SAVED_ACCOUNTS, accounts.join(';'));
    }
    savedAccounts.value = accounts;
  }

  /**
   * 切换到另一个已保存账户（只读模式）
   *
   * 对标 nrs.login.js:202-235 的 switchAccount：重置当前状态后以只读方式登录。
   *
   * @param accountRs RS 地址
   */
  async function switchAccount(accountRs: string): Promise<void> {
    // 重置当前会话（保留已保存账户列表）
    logout();
    await loginByAccount(accountRs);
  }

  // --- Lockscreen 支持 ---

  /**
   * 锁定会话（保留 secretPhrase 在内存，UI 进入锁屏）
   *
   * 对标 nrs.login.js:579 的 showLockscreen。锁定后用户需解锁才能继续操作。
   */
  function lock(): void {
    if (!isLoggedIn.value) return;
    isLocked.value = true;
  }

  /**
   * 解锁会话（验证 secretPhrase 后解除锁屏）
   *
   * @param password 密码短语
   * @throws 密码短语与当前账户不匹配时抛出
   */
  async function unlock(password: string): Promise<void> {
    if (!password) throw new Error('密码短语不能为空');
    const derived = passphraseToAccount(password.trim());
    if (derived.accountRS !== accountRS.value) {
      throw new Error('密码短语与当前账户不匹配');
    }
    secretPhrase.value = password.trim();
    isLocked.value = false;
  }

  /**
   * 直接解锁（无密码，用于会话恢复后已验证场景）
   */
  function unlockDirect(): void {
    isLocked.value = false;
  }

  // --- 持久化 ---

  /**
   * 持久化只读状态到 localStorage（不持久化 secretPhrase）
   *
   * 对标 nrs.login.js 的 localStorage.setItem("logged_in", true)。
   */
  function persistReadOnlyState(): void {
    localStorage.setItem(STORAGE_KEY_ACCOUNT_RS, accountRS.value);
    localStorage.setItem(STORAGE_KEY_ACCOUNT_ID, accountId.value);
    localStorage.setItem(STORAGE_KEY_PUBLIC_KEY, publicKey.value);
    localStorage.setItem(STORAGE_KEY_LOGGED_IN, 'true');
    localStorage.setItem(STORAGE_KEY_LOGIN_TYPE, loginType.value);
  }

  /**
   * 清除会话 localStorage（保留已保存账户列表）
   */
  function clearSessionStorage(): void {
    localStorage.removeItem(STORAGE_KEY_ACCOUNT_RS);
    localStorage.removeItem(STORAGE_KEY_ACCOUNT_ID);
    localStorage.removeItem(STORAGE_KEY_PUBLIC_KEY);
    localStorage.removeItem(STORAGE_KEY_LOGGED_IN);
    localStorage.removeItem(STORAGE_KEY_LOGIN_TYPE);
    // ⛔ 不清除 STORAGE_KEY_SAVED_ACCOUNTS（已保存账户列表保留）
    // ⛔ 不再持久化 secretPhrase，无需清除
  }

  /**
   * ⛔ 废弃：原 persistToStorage 会持久化 secretPhrase 明文，已移除（阶段 0.5）。
   * 保留空函数避免外部调用断裂，实际只持久化只读状态。
   * @deprecated 请使用 persistReadOnlyState
   */
  function persistToStorage(): void {
    persistReadOnlyState();
  }

  /**
   * 从 localStorage 恢复只读会话（应用启动时调用）
   *
   * 注意：secretPhrase 不持久化，恢复后为只读模式；
   * 若需发送交易，用户需重新输入 secretPhrase（通过 lockscreen 或登录页）。
   */
  function initFromStorage(): void {
    const savedRS = localStorage.getItem(STORAGE_KEY_ACCOUNT_RS);
    const savedId = localStorage.getItem(STORAGE_KEY_ACCOUNT_ID);
    const savedPK = localStorage.getItem(STORAGE_KEY_PUBLIC_KEY);
    const savedLoginType = localStorage.getItem(STORAGE_KEY_LOGIN_TYPE);

    if (savedRS) accountRS.value = savedRS;
    if (savedId) accountId.value = savedId;
    if (savedPK) publicKey.value = savedPK;
    if (savedLoginType === 'password' || savedLoginType === 'account') {
      loginType.value = savedLoginType;
    }
    // secretPhrase 不恢复（内存暂存）
    secretPhrase.value = '';

    listSavedAccounts();

    // 若有已保存会话，尝试刷新余额 + 初始化 IndexedDB
    if (savedRS) {
      refreshAccount();
      // 初始化账户级 IndexedDB（账户隔离，对标 nrs.localstorage.js initUserDB）
      if (savedId) {
        initUserDB(savedId).catch(() => {
          // IndexedDB 不可用时静默失败，storage 层会回退到 localStorage
        });
      }
    }
  }

  // 应用启动时自动恢复
  initFromStorage();

  return {
    // state
    accountRS,
    accountId,
    publicKey,
    balanceNQT,
    effectiveBalance,
    unconfirmedBalanceNQT,
    forgedBalanceNQT,
    guaranteedBalanceNQT,
    name,
    description,
    secretPhrase,
    isTestNet,
    isLocked,
    loginType,
    loginWarning,
    savedAccounts,
    // 阶段 1.2：完整账户信息与状态
    accountInfo,
    leasingStatus,
    phasingOnlyControl,
    hasPhasingOnlyControl,
    previousAssetBalances,
    // getters
    isLoggedIn,
    balanceFormatted,
    hasSecretPhrase,
    // actions
    login,
    loginByAccount,
    refreshAccount,
    logout,
    initFromStorage,
    // 多账户管理
    listSavedAccounts,
    rememberAccount,
    removeAccount,
    switchAccount,
    // lockscreen
    lock,
    unlock,
    unlockDirect,
    // 阶段 1.2：账户信息与状态
    getAccountInfo,
    updateAccountLeasingStatus,
    updateAccountControlStatus,
    checkAssetDifferences,
    onAssetDifference,
    // ⛔ deprecated（保留兼容）
    persistToStorage,
  };
});
