/**
 * NRCS Wallet - API Client
 * 
 * API 请求封装类
 * 负责与后端 NRCS API 的所有通信
 * 
 * P0 增强功能 (参考 NRCS nrs.server.js):
 * - 数据自动预处理 (Trim)
 * - 密码短语安全验证
 * - NXT → NQT 单位转换
 * - GET 请求自动重试机制
 */

class NrcsApi {
  constructor(baseUrl = '/nrcs') {
    this.baseUrl = baseUrl;
    this.timeoutMs = 30000; // 默认超时 30 秒
    this.requestCount = 0;
    this.lastRequestTime = null;
    this.cacheEnabled = true;
    this.cache = new Map();
    this.cacheTtl = 5000; // 缓存有效期 5 秒
  }

  /**
   * 发送通用请求 (主入口)
   * 
   * 参考 NRCS nrs.server.js 第 27-209 行的完整实现:
   * 1. 数据预处理 (自动 Trim)
   * 2. 安全验证 (secretPhrase 匹配检查)
   * 3. 单位转换 (NXT → NQT)
   * 4. 带重试的请求执行
   * 
   * @param {string} requestType - API 请求类型
   * @param {Object} params - 请求参数
   * @param {string} method - HTTP 方法 (GET/POST)
   * @returns {Promise<Object>} 响应数据
   */
  async request(requestType, params = {}, method = 'GET') {
    // Step 1: 数据预处理 (参考 NRCS nrs.server.js 第 53-59 行)
    const processedParams = this.preprocessData(params);
    
    // Step 2: 安全验证 (参考 NRCS nrs.server.js 第 195-208 行)
    if ('secretPhrase' in processedParams) {
      await this.validateSecretPhrase(processedParams.secretPhrase, processedParams);
    }
    
    // Step 3: 单位转换 (参考 NRCS nrs.server.js 第 61-94 行)
    const convertedParams = this.convertUnits(processedParams);
    
    const cacheKey = `${method}:${requestType}:${JSON.stringify(convertedParams)}`;
    
    // 检查缓存 (仅 GET 请求)
    if (this.cacheEnabled && method === 'GET') {
      const cached = this.getFromCache(cacheKey);
      if (cached) return cached;
    }

    try {
      // Step 4: 执行请求（带重试机制）
      const result = await this.executeWithRetry(requestType, convertedParams, method, cacheKey);
      return result;

    } catch (error) {
      console.error(`API Error [${requestType}]:`, error.message);
      
      // 区分错误类型 (参考 NRCS nrs.server.js 第 494-521 行)
      if (error.name === 'AbortError') {
        throw new Error('Request timeout. Please try again.');
      }
      
      throw error;
    }
  }

  /**
   * 数据预处理
   * 
   * 参考 NRCS nrs.server.js 第 53-59 行:
   * $.each(data, function (key, val) {
   *     if (key != "secretPhrase") {
   *       data[key] = $.trim(val);  // 自动 trim
   *     }
   * });
   * 
   * @param {Object} params - 原始参数
   * @returns {Object} 处理后的参数
   */
  preprocessData(params) {
    const processed = {};
    
    for (const [key, value] of Object.entries(params)) {
      // 除密码短语外，所有字符串字段自动 trim (参考 NRCS)
      if (key === 'secretPhrase') {
        processed[key] = value;
      } else if (typeof value === 'string') {
        processed[key] = value.trim();
      } else {
        processed[key] = value;
      }
    }
    
    return processed;
  }

  /**
   * 验证密码短语安全性
   * 
   * 参考 NRCS nrs.server.js 第 195-208 行:
   * 检查 secretPhrase 是否匹配当前登录账户，防止误操作
   * 
   * @param {string} secretPhrase - 用户输入的密码短语
   * @param {Object} params - 完整参数对象
   * @throws {Error} 如果验证失败
   */
  async validateSecretPhrase(secretPhrase, params) {
    // 如果是计算费用或签名请求，跳过验证 (参考 NRCS: data.calculateFee)
    if (params.calculateFee || params.doNotSign) {
      return;
    }
    
    // 如果没有已登录账户，无法验证，允许继续
    const currentAccount = store.state?.accountId || store.state?.account?.id;
    if (!currentAccount) {
      console.debug('No logged-in account found, skipping passphrase validation');
      return;
    }
    
    try {
      // 获取密码短语对应的账户 ID
      const derivedAccountId = await this.getAccountIdFromPassphrase(secretPhrase);
      
      // 验证是否匹配当前账户 (参考 NRCS: accountId != NRS.account)
      if (derivedAccountId && derivedAccountId !== currentAccount) {
        throw new Error(
          'Incorrect passphrase for this account. ' +
          'The passphrase you provided does not match the currently logged-in account.'
        );
      }
      
      console.debug('Passphrase validation passed');
      
    } catch (error) {
      // 如果是无法获取账户 ID 的错误，仅警告不阻止
      if (error.message?.includes('Incorrect passphrase')) {
        throw error;  // 重新抛出关键错误
      }
      
      console.warn('Passphrase validation warning:', error.message);
    }
  }

  /**
   * 从密码短语获取账户 ID
   * 
   * 辅助方法，用于安全验证
   * 
   * @param {string} passphrase - 密码短语
   * @returns {Promise<string|null>} 账户 ID 或 null
   */
  async getAccountIdFromPassphrase(passphrase) {
    try {
      // 调用后端 API 获取账户 ID (参考 NRCS: getAccountId)
      const result = await this.executeRequest('getAccountId', { 
        secretPhrase: passphrase 
      }, 'POST', null, false);  // 不使用缓存和重试
      
      return result.account || null;
      
    } catch (error) {
      console.warn('Failed to get account ID from passphrase:', error.message);
      return null;
    }
  }

  /**
   * 单位转换 (NXT → NQT)
   * 
   * 参考 NRCS nrs.server.js 第 61-94 行:
   * 自动将 NXT 单位转换为 NQT (1 NXT = 100000000 NQT)
   * 
   * 支持的字段映射:
   * - feeNXT → feeNQT
   * - amountNXT → amountNQT
   * - priceNXT → priceNQT
   * - 等等...
   * 
   * @param {Object} params - 参数对象
   * @returns {Object} 转换后的参数
   */
  convertUnits(params) {
    const converted = { ...params };
    
    // NXT 到 NQT 字段映射表 (参考 NRCS nrs.server.js 第 63-78 行)
    // P1-A2 增强: 添加更多字段覆盖
    const nxtToNqtFields = [
      // 基础交易字段
      ['feeNXT', 'feeNQT'],
      ['amountNXT', 'amountNQT'],
      ['priceNXT', 'priceNQT'],
      
      // 退款和折扣
      ['refundNXT', 'refundNQT'],
      ['discountNXT', 'discountNQT'],
      
      // 阶段性投票 (Phasing)
      ['phasingQuorumNXT', 'phasingQuorum'],
      ['phasingMinBalanceNXT', 'phasingMinBalance'],
      
      // 账户控制 (Account Control)
      ['controlQuorumNXT', 'controlQuorum'],
      ['controlMinBalanceNXT', 'controlMinBalance'],
      ['controlMaxFeesNXT', 'controlMaxFees'],
      
      // 最小余额要求
      ['minBalanceNXT', 'minBalance'],
      
      // 洗牌交易 (Shuffling)
      ['shufflingAmountNXT', 'amount'],
      
      // 监控相关 (Monitoring)
      ['monitorAmountNXT', 'amount'],
      ['monitorThresholdNXT', 'threshold'],
    ];
    
    for (const [nxtField, nqtField] of nxtToNqtFields) {
      if (nxtField in converted && converted[nxtField] !== undefined) {
        const originalValue = converted[nxtField];
        converted[nqtField] = this.convertToNQT(originalValue);
        delete converted[nxtField];
        
        console.debug(`Unit conversion: ${nxtField} (${originalValue}) → ${nqtField} (${converted[nqtField]})`);
      }
    }
    
    return converted;
  }

  /**
   * 将 NXT 数值转换为 NQT (增强版)
   * 
   * NRCS 标准: 1 NXT = 10^8 NQT = 100000000 NQT
   * 
   * P1-A2 增强:
   * - 边界值处理 (负数、零、超大数值)
   * - 精度保护 (避免浮点数精度丢失)
   * - 类型安全 (支持字符串和数字输入)
   * 
   * @param {string|number} nxtAmount - NXT 数值
   * @returns {string} NQT 数值 (字符串格式)
   */
  convertToNQT(nxtAmount) {
    // 边界处理: 空值或未定义
    if (nxtAmount === null || nxtAmount === undefined || nxtAmount === '') {
      console.warn('[convertToNQT] Empty value provided, returning "0"');
      return '0';
    }
    
    // 类型转换
    const amount = parseFloat(nxtAmount);
    
    // 边界处理: 非数字输入
    if (isNaN(amount)) {
      console.error(`[convertToNQT] Invalid number: ${nxtAmount}`);
      throw new Error(`Invalid NXT amount: ${nxtAmount}. Must be a valid number.`);
    }
    
    // 边界处理: 负数值 (某些场景允许负数，如退款)
    if (amount < 0) {
      console.warn(`[convertToNQT] Negative value detected: ${amount}`);
      // 允许负数继续处理（用于退款等场景）
    }
    
    // 边界处理: 零值
    if (amount === 0) {
      return '0';
    }
    
    // 边界处理: 超大数值 (防止溢出)
    const MAX_SAFE_NXT = Number.MAX_SAFE_INTEGER / 100000000;  // 约 9e12 NXT
    if (Math.abs(amount) > MAX_SAFE_NXT) {
      console.error(`[convertToNQT] Value too large: ${amount} NXT`);
      throw new Error(`NXT amount too large: ${amount}. Maximum allowed: ${MAX_SAFE_NXT} NXT`);
    }
    
    // 精度保护: 使用整数运算避免浮点数精度问题
    // NRCS: 8 位小数精度 (参考 NRCS constants)
    const NQT_PER_NXT = 100000000;
    
    // 方法1: 对于小数点后位数 ≤ 8 的数字，直接乘法
    // 方法2: 对于更多位数的数字，使用字符串分割保证精度
    let nqtValue;
    
    if (this.countDecimalPlaces(amount) <= 8) {
      // 标准情况: 直接乘法
      nqtValue = Math.round(amount * NQT_PER_NXT);
    } else {
      // 高精度情况: 使用字符串分割
      nqtValue = this.highPrecisionConvert(amount, NQT_PER_NXT);
    }
    
    // 最终验证: 确保结果为有效整数
    if (!Number.isInteger(nqtValue)) {
      console.warn(`[convertToNQT] Rounding applied to non-integer result`);
      nqtValue = Math.round(nqtValue);
    }
    
    return nqtValue.toString();
  }

  /**
   * 将 NQT 数值转换为 NXT (反向转换)
   * 
   * 用于显示目的：将 API 返回的 NQT 值转换为用户友好的 NXT 格式
   * 
   * @param {string|number} nqtAmount - NQT 数值
   * @param {number} decimals - 显示的小数位数 (默认 8 位)
   * @returns {string} 格式化后的 NXT 数值
   */
  convertFromNQT(nqtAmount, decimals = 8) {
    // 边界处理
    if (nqtAmount === null || nqtAmount === undefined || nqtAmount === '') {
      return '0'.padStart(decimals + 2, '0');  // "0.00000000"
    }
    
    const amount = parseFloat(nqtAmount);
    
    if (isNaN(amount)) {
      console.error(`[convertFromNQT] Invalid NQT amount: ${nqtAmount}`);
      return '0'.repeat(decimals + 1).split('').join('').padStart(decimals + 2, '0');
    }
    
    // 反向转换: NQT → NXT
    const NQT_PER_NXT = 100000000;
    const nxtValue = amount / NQT_PER_NXT;
    
    // 格式化为指定小数位数
    return nxtValue.toFixed(decimals);
  }

  /**
   * 计算数字的小数位数
   * 
   * @param {number} num - 数字
   * @returns {number} 小数位数
   */
  countDecimalPlaces(num) {
    const str = Math.abs(num).toString();
    const decimalIndex = str.indexOf('.');
    
    if (decimalIndex === -1) {
      return 0;
    }
    
    return str.length - decimalIndex - 1;
  }

  /**
   * 高精度单位转换 (用于超过 8 位小数的数字)
   * 
   * 使用字符串操作避免 JavaScript 浮点数精度问题
   * 
   * @param {number} amount - 原始数值
   * @param {number} multiplier - 乘数 (如 100000000)
   * @returns {number} 转换后的整数值
   */
  highPrecisionConvert(amount, multiplier) {
    // 转换为字符串并分割整数和小数部分
    const str = amount.toString();
    const [integerPart, decimalPart = ''] = str.split('.');
    
    // 移除可能的负号
    const isNegative = integerPart.startsWith('-');
    const absInteger = isNegative ? integerPart.slice(1) : integerPart;
    
    // 合并整数和小数部分（去掉小数点）
    const combinedDigits = absInteger + decimalPart;
    
    // 计算需要补齐的零的个数
    const decimalPlaces = decimalPart.length;
    const zerosToAdd = multiplier.toString().length - 1 - decimalPlaces;
    
    // 补齐零并转换为数字
    const paddedDigits = zerosToAdd > 0 
      ? combinedDigits + '0'.repeat(zerosToAdd)
      : combinedDigits.slice(0, combinedDigits.length + zerosToAdd);  // 截断
    
    let result = parseInt(paddedDigits, 10);
    
    if (isNegative) {
      result = -result;
    }
    
    return result;
  }

  /**
   * 带重试机制的请求执行器
   * 
   * 参考 NRCS nrs.server.js 第 384-522 行:
   * shouldRetry: (httpMethod == "GET" ? 2 : undefined)
   * GET 请求自动重试 2 次
   * 
   * @param {string} requestType - API 类型
   * @param {Object} params - 参数
   * @param {string} method - HTTP 方法
   * @param {string|null} cacheKey - 缓存键
   * @param {boolean} useRetry - 是否使用重试机制
   * @returns {Promise<Object>} 响应数据
   */
  async executeWithRetry(requestType, params, method, cacheKey, useRetry = true) {
    // 参考 NRCS: GET 请求重试 2 次，POST 请求不重试
    const maxRetries = (method === 'GET' && useRetry) ? 2 : 0;
    let lastError = null;
    
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        // 执行实际请求
        const result = await this.executeRequest(requestType, params, method, cacheKey, true);
        
        // 如果有重试成功，记录日志
        if (attempt > 0) {
          console.log(`API request succeeded after ${attempt} retries: ${requestType}`);
        }
        
        return result;
        
      } catch (error) {
        lastError = error;
        
        // 如果不是 GET 请求或已达到最大重试次数，直接抛出错误
        if (method !== 'GET' || attempt >= maxRetries || !useRetry) {
          throw error;
        }
        
        // 记录重试日志 (参考 NRCS: console.error for retry)
        console.warn(
          `API request failed (attempt ${attempt + 1}/${maxRetries + 1}), retrying... ` +
          `[${requestType}]: ${error.message}`
        );
        
        // 指数退避延迟 (1秒, 2秒...)
        const delayMs = 1000 * Math.pow(2, attempt);
        await new Promise(resolve => setTimeout(resolve, delayMs));
      }
    }
    
    // 理论上不会执行到这里，但为了安全起见
    throw lastError || new Error('Request failed after retries');
  }

  /**
   * 执行实际的 HTTP 请求
   * 
   * @param {string} requestType - API 类型
   * @param {Object} params - 参数
   * @param {string} method - HTTP 方法
   * @param {string|null} cacheKey - 缓存键
   * @param {boolean} enableCache - 是否启用缓存
   * @returns {Promise<Object>} 响应数据
   */
  async executeRequest(requestType, params, method, cacheKey, enableCache = true) {
    this.requestCount++;
    this.lastRequestTime = Date.now();

    let url, options;

    if (method === 'GET') {
      url = `${this.baseUrl}?requestType=${requestType}&${this.buildQueryString(params)}`;
      options = { method: 'GET' };
    } else {
      url = `${this.baseUrl}`;
      options = {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({ requestType, ...params }),
      };
    }

    // 添加超时控制 (参考 NRCS: timeout: 30000)
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeoutMs);
    options.signal = controller.signal;

    const response = await fetch(url, options);
    clearTimeout(timeoutId);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();

    // 缓存响应 (如果启用)
    if (enableCache && this.cacheEnabled && method === 'GET' && cacheKey) {
      this.setCache(cacheKey, data);
    }

    return data;
  }

  /**
   * 构建查询字符串
   * @param {Object} params - 参数对象
   * @returns {string}
   */
  buildQueryString(params) {
    return Object.entries(params)
      .filter(([_, value]) => value !== undefined && value !== null)
      .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`)
      .join('&');
  }

  /**
   * 从缓存获取数据
   */
  getFromCache(key) {
    if (!this.cache.has(key)) return null;
    
    const item = this.cache.get(key);
    const now = Date.now();
    
    if (now - item.timestamp > this.cacheTtl) {
      this.cache.delete(key);
      return null;
    }
    
    return item.data;
  }

  /**
   * 设置缓存数据
   */
  setCache(key, data) {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
    });
    
    // 清理过期缓存
    if (this.cache.size > 100) {
      this.clearExpiredCache();
    }
  }

  /**
   * 清理过期缓存
   */
  clearExpiredCache() {
    const now = Date.now();
    for (const [key, item] of this.cache.entries()) {
      if (now - item.timestamp > this.cacheTtl) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * 清除所有缓存
   */
  clearCache() {
    this.cache.clear();
  }

  // ========== 账户相关 API ==========

  /**
   * 获取账户信息
   * @param {string|number} accountId - 账户 ID 或 RS 地址
   * @returns {Promise<Object>}
   */
  async getAccount(accountId) {
    return this.request('getAccount', { account: accountId });
  }

  /**
   * 获取账户余额
   * @param {string|number} accountId
   * @returns {Promise<Object>}
   */
  async getAccountBalance(accountId) {
    return this.request('getBalance', { account: accountId });
  }

  /**
   * 设置账户信息
   * @param {Object} params
   * @returns {Promise<Object>}
   */
  async setAccountInfo(params) {
    return this.request('setAccountInfo', params, 'POST');
  }

  /**
   * 设置别名
   * @param {Object} params
   * @returns {Promise<Object>}
   */
  async setAlias(params) {
    return this.request('setAlias', params, 'POST');
  }

  // ========== 交易相关 API ==========

  /**
   * 获取区块链交易列表
   * @param {string|number} account - 账户 ID
   * @param {number} firstIndex - 起始索引
   * @param {number} lastIndex - 结束索引
   * @returns {Promise<Object>}
   */
  async getBlockchainTransactions(account, firstIndex = 0, lastIndex = 19) {
    return this.request('getBlockchainTransactions', {
      account,
      firstIndex,
      lastIndex,
    });
  }

  /**
   * 获取未确认交易
   * @param {string|number} account - 账户 ID
   * @returns {Promise<Object>}
   */
  async getUnconfirmedTransactions(account) {
    return this.request('getUnconfirmedTransactions', { account });
  }

  /**
   * 发送 NRC 支付
   * @param {Object} params - { recipient, amountNQT, feeNQT, secretPhrase, ... }
   * @returns {Promise<Object>}
   */
  async sendMoney(params) {
    return this.request('sendMoney', params, 'POST');
  }

  /**
   * 发送消息
   * @param {Object} params
   * @returns {Promise<Object>}
   */
  async sendMessage(params) {
    return this.request('sendMessage', params, 'POST');
  }

  /**
   * 广播交易
   * @param {string} transactionBytes - 交易字节
   * @param {string} transactionJSON - 交易 JSON
   * @returns {Promise<Object>}
   */
  async broadcastTransaction(transactionBytes, transactionJSON) {
    return this.request('broadcastTransaction', {
      transactionBytes,
      transactionJSON,
    }, 'POST');
  }

  // ========== 区块相关 API ==========

  /**
   * 获取区块链状态
   * @returns {Promise<Object>}
   */
  async getBlockchainStatus() {
    return this.request('getBlockchainStatus');
  }

  /**
   * 获取最新区块
   * @returns {Promise<Object>}
   */
  async getLatestBlock() {
    return this.request('getBlock', { height: -1 });
  }

  /**
   * 根据高度获取区块
   * @param {number} height - 区块高度
   * @param {boolean} includeTransactions - 是否包含交易
   * @returns {Promise<Object>}
   */
  async getBlock(height, includeTransactions = false) {
    return this.request('getBlock', { height, includeTransactions });
  }

  /**
   * 获取区块列表
   * @param {number} firstIndex
   * @param {number} lastIndex
   * @returns {Promise<Object>}
   */
  async getBlocks(firstIndex = 0, lastIndex = 99) {
    return this.request('getBlocks', { firstIndex, lastIndex });
  }

  /**
   * 获取指定高度的区块 ID
   * @param {number} height
   * @returns {Promise<Object>}
   */
  async getBlockId(height) {
    return this.request('getBlockId', { height });
  }

  // ========== 网络节点 API ==========

  /**
   * 获取已连接的节点
   * @param {boolean} activeOnly - 仅活跃节点
   * @returns {Promise<Object>}
   */
  async getPeers(activeOnly = false) {
    return this.request('getPeers', { activeOnly, state: 'CONNECTED' });
  }

  /**
   * 获取网络状态
   * @returns {Promise<Object>}
   */
  async getState() {
    return this.request('getState');
  }

  // ========== 其他 API ==========

  /**
   * 全局搜索
   * @param {string} query - 搜索关键词
   * @returns {Promise<Object>}
   */
  async search(query) {
    return this.request('searchAccounts', { query });
  }

  /**
   * 获取时间
   * @returns {Promise<Object>}
   */
  async getTime() {
    return this.request('getTime');
  }

  /**
   * 开始/停止锻造
   * @param {string} secretPhrase
   * @param {boolean} start - 是否开始
   * @returns {Promise<Object>}
   */
  async startStopForging(secretPhrase, start = true) {
    return this.request(start ? 'startForging' : 'stopForging', { secretPhrase }, 'POST');
  }

  /**
   * 批量并行请求
   * @param {Array<{requestType, params?, method?}>} requests - 请求数组
   * @returns {Promise<Array>} 结果数组
   */
  async batchRequests(requests) {
    return Promise.allSettled(
      requests.map(({ requestType, params, method }) =>
        this.request(requestType, params || {}, method || 'GET')
      )
    );
  }

  /**
   * 获取 API 统计信息
   * @returns {Object}
   */
  getStats() {
    return {
      totalRequests: this.requestCount,
      lastRequest: this.lastRequestTime,
      cacheSize: this.cache.size,
    };
  }
}

// 创建全局单例实例
window.api = new NrcsApi();
