/**
 * NRCS Wallet - API Client
 * 
 * API 请求封装类
 * 负责与后端 NRCS API 的所有通信
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
   * 发送通用请求
   * @param {string} requestType - API 请求类型
   * @param {Object} params - 请求参数
   * @param {string} method - HTTP 方法 (GET/POST)
   * @returns {Promise<Object>} 响应数据
   */
  async request(requestType, params = {}, method = 'GET') {
    const cacheKey = `${method}:${requestType}:${JSON.stringify(params)}`;
    
    // 检查缓存 (仅 GET 请求)
    if (this.cacheEnabled && method === 'GET') {
      const cached = this.getFromCache(cacheKey);
      if (cached) return cached;
    }

    try {
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

      // 添加超时控制
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.timeoutMs);
      options.signal = controller.signal;

      const response = await fetch(url, options);
      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();

      // 缓存响应
      if (this.cacheEnabled && method === 'GET') {
        this.setCache(cacheKey, data);
      }

      return data;
    } catch (error) {
      console.error(`API Error [${requestType}]:`, error.message);
      
      // 区分错误类型
      if (error.name === 'AbortError') {
        throw new Error('Request timeout. Please try again.');
      }
      
      throw error;
    }
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
