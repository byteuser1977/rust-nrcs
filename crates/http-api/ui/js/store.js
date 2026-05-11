/**
 * NRCS Wallet - State Store
 * 
 * 简单的状态管理器
 * 使用观察者模式实现响应式状态更新
 */

class Store {
  constructor() {
    this.state = {
      // 认证状态 (新增登录相关)
      isLoggedIn: false,
      isAuthenticated: false,
      secretPhrase: null,
      
      // 账户信息 (扩展)
      accountId: null,
      accountRS: null,
      publicKey: null,
      isPassphraseLogin: false,
      lastLoginTime: null,
      
      // 兼容旧版账户结构
      account: {
        id: null,
        rsAddress: null,
        publicKey: null,
        balanceNQT: 0,
        unconfirmedBalanceNQT: 0,
      },
      
      // 区块链状态 (增强同步状态管理)
      blockchain: {
        height: 0,
        blockchainHeight: 0,  // 网络总高度
        lastBlock: null,
        lastBlockHeight: 0,  // 本地最新区块高度
        cumulativeDifficulty: null,
        numberOfBlocks: 0,
        time: 0,
        version: '',
        isScanning: false,
        isDownloading: false,
        syncStatus: 'unknown',  // unknown | scanning | downloading | synced | error
        syncProgress: 0,       // 0-100
      },
      
      // 最新区块
      latestBlock: null,
      
      // 交易数据
      transactions: [],
      unconfirmedTransactions: [],
      
      // 节点信息
      peers: {
        connected: 0,
        total: 0,
        list: [],
      },
      
      // 连接状态
      connectionStatus: 'disconnected', // disconnected, connecting, connected, error
      
      // 铸造状态
      forgingStatus: false,
      
      // UI 状态
      ui: {
        sidebarOpen: false,
        currentPage: 'dashboard',
        loading: false,
        error: null,
      },
      
      // ========== P1-R3: 全局分页状态 (参考 NRCS nrs.js 第 620-630 行) ==========
      // NRCS 分页变量:
      // - NRS.pageNumber: 当前页码
      // - NRS.hasMorePages: 是否有更多页
      // - NRS.showPageNumbers: 是否显示分页控件
      // - NRS.currentSubPage: 当前子页面
      pagination: {
        currentPage: 1,           // 当前页码 (参考 NRCS: NRS.pageNumber)
        itemsPerPage: 50,         // 每页项目数 (NRCS 默认值)
        hasMorePages: false,       // 是否有更多页 (参考 NRCS: NRS.hasMorePages)
        showPageNumbers: false,    // 是否显示分页数字 (参考 NRCS: NRS.showPageNumbers)
        currentSubPage: '',        // 当前子页面标识 (参考 NRCS: NRS.currentSubPage)
        totalItems: 0,             // 总项目数
        totalPages: 0,             // 总页数
        firstIndex: 0,             // 起始索引 (用于 API 请求)
        lastIndex: 49,             // 结束索引 (用于 API 请求)
      },
    };
    
    this.listeners = new Map();
    this.history = [];
    this.maxHistory = 50;
    
    // 自动持久化到 localStorage
    this.persistKeys = ['isAuthenticated', 'account', 'ui.currentPage'];
  }

  /**
   * 获取当前状态的快照
   * @returns {Object} 当前状态
   */
  getState() {
    return { ...this.state };
  }

  /**
   * 更新状态 (支持深度合并)
   * @param {string|Object} pathOrState - 属性路径或状态对象
   * @param {*} value - 新值
   */
  setState(pathOrState, value) {
    let newState;
    
    if (typeof pathOrState === 'string') {
      // 单个属性更新
      newState = this.setNestedValue(this.state, pathOrState, value);
    } else if (typeof pathOrState === 'object') {
      // 批量更新
      newState = { ...this.state, ...pathOrState };
    } else {
      console.error('Invalid state update');
      return;
    }
    
    const prevState = this.state;
    this.state = newState;
    
    // 记录历史
    this.recordHistory(prevState, newState);
    
    // 持久化
    this.persist();
    
    // 通知订阅者
    this.notify(prevState);
  }

  /**
   * 设置嵌套属性值
   */
  setNestedValue(obj, path, value) {
    const keys = path.split('.');
    let current = obj;
    
    for (let i = 0; i < keys.length - 1; i++) {
      if (!(keys[i] in current)) {
        current[keys[i]] = {};
      }
      current[keys[i]] = { ...current[keys[i]] };
      current = current[keys[i]];
    }
    
    current[keys[keys.length - 1]] = value;
    return { ...obj };
  }

  /**
   * 订阅状态变化
   * @param {Function} listener - 监听函数
   * @param {string|null} key - 可选的监听键名
   * @returns {Function} 取消订阅函数
   */
  subscribe(listener, key = null) {
    if (!this.listeners.has(key)) {
      this.listeners.set(key, new Set());
    }
    
    this.listeners.get(key).add(listener);
    
    // 返回取消订阅函数
    return () => {
      const listeners = this.listeners.get(key);
      if (listeners) {
        listeners.delete(listener);
      }
    };
  }

  /**
   * 通知所有订阅者
   */
  notify(prevState) {
    for (const [key, listeners] of this.listeners.entries()) {
      listeners.forEach(listener => {
        try {
          listener(this.state, prevState);
        } catch (error) {
          console.error('Store listener error:', error);
        }
      });
    }
  }

  /**
   * 记录状态变更历史
   */
  recordHistory(prevState, newState) {
    const changes = this.getChanges(prevState, newState);
    
    if (changes.length > 0) {
      this.history.push({
        timestamp: Date.now(),
        changes,
        state: newState,
      });
      
      // 限制历史记录数量
      if (this.history.length > this.maxHistory) {
        this.history.shift();
      }
    }
  }

  /**
   * 比较两个状态的差异
   */
  getChanges(oldState, newState) {
    const changes = [];
    
    function compare(obj1, obj2, path = '') {
      const keys = new Set([...Object.keys(obj1), ...Object.keys(obj2)]);
      
      for (const key of keys) {
        const currentPath = path ? `${path}.${key}` : key;
        
        if (obj1[key] !== obj2[key]) {
          if (
            typeof obj1[key] === 'object' &&
            typeof obj2[key] === 'object' &&
            obj1[key] !== null &&
            obj2[key] !== null
          ) {
            compare(obj1[key], obj2[key], currentPath);
          } else {
            changes.push({
              path: currentPath,
              from: obj1[key],
              to: obj2[key],
            });
          }
        }
      }
    }
    
    compare(oldState, newState);
    return changes;
  }

  /**
   * 持久化状态到 localStorage
   */
  persist() {
    try {
      const dataToSave = {};
      
      this.persistKeys.forEach(key => {
        const value = this.getNestedValue(this.state, key);
        if (value !== undefined && value !== null) {
          dataToSave[key] = value;
        }
      });
      
      localStorage.setItem('nrcs_wallet_state', JSON.stringify(dataToSave));
    } catch (error) {
      console.warn('Failed to persist state:', error.message);
    }
  }

  /**
   * 从 localStorage 恢复状态
   */
  restore() {
    try {
      const saved = localStorage.getItem('nrcs_wallet_state');
      
      if (saved) {
        const data = JSON.parse(saved);
        
        Object.keys(data).forEach(key => {
          this.state = this.setNestedValue(this.state, key, data[key]);
        });
      }
    } catch (error) {
      console.warn('Failed to restore state:', error.message);
    }
  }

  /**
   * 获取嵌套属性值
   */
  getNestedValue(obj, path) {
    return path.split('.').reduce((current, key) => {
      return current?.[key];
    }, obj);
  }

  /**
   * 重置状态
   */
  reset() {
    localStorage.removeItem('nrcs_wallet_state');
    localStorage.removeItem('nrcs_saved_accounts');
    this.state = {
      isLoggedIn: false,
      isAuthenticated: false,
      secretPhrase: null,
      
      // 登录相关
      accountId: null,
      accountRS: null,
      publicKey: null,
      isPassphraseLogin: false,
      lastLoginTime: null,
      
      account: {
        id: null,
        rsAddress: null,
        publicKey: null,
        balanceNQT: 0,
        unconfirmedBalanceNQT: 0,
      },
      blockchain: {
        height: 0,
        blockchainHeight: 0,
        lastBlock: null,
        lastBlockHeight: 0,
        cumulativeDifficulty: null,
        numberOfBlocks: 0,
        time: 0,
        version: '',
        isScanning: false,
        isDownloading: false,
        syncStatus: 'unknown',
        syncProgress: 0,
      },
      latestBlock: null,
      transactions: [],
      unconfirmedTransactions: [],
      peers: {
        connected: 0,
        total: 0,
        list: [],
      },
      connectionStatus: 'disconnected',
      forgingStatus: false,
      ui: {
        sidebarOpen: false,
        currentPage: 'dashboard',
        loading: false,
        error: null,
      },
    };
    this.notify(this.state);
  }

  /**
   * 获取历史记录
   * @returns {Array}
   */
  getHistory() {
    return [...this.history];
  }

  /**
   * 导出状态为 JSON
   * @returns {string}
   */
  exportState() {
    return JSON.stringify(this.state, null, 2);
  }

  /**
   * 调试方法 - 打印当前状态
   */
  debug() {
    console.group('%cStore State', 'color: #00d4ff; font-weight: bold;');
    console.log('Current State:', this.state);
    console.log('Listeners:', this.listeners.size, 'groups');
    console.log('History Length:', this.history.length);
    console.groupEnd();
  }

  // ========== P1-R3: 分页状态管理方法 (参考 NRCS nrs.js 第 770-781 行) ==========

  /**
   * 获取当前分页状态
   * 
   * @returns {Object} 分页状态对象
   */
  getPaginationState() {
    return { ...this.state.pagination };
  }

  /**
   * 设置当前页码
   * 
   * 参考 NRCS nrs.js 第 776 行:
   * NRS.goToPageNumber = function (pageNumber) {
   *     NRS.pageNumber = pageNumber;
   *     NRS.pageLoading();
   *     NRS.pages[NRS.currentPage]();
   * };
   * 
   * @param {number} pageNumber - 目标页码 (从 1 开始)
   */
  setPageNumber(pageNumber) {
    if (!Number.isInteger(pageNumber) || pageNumber < 1) {
      console.warn('Invalid page number:', pageNumber);
      return;
    }
    
    const pagination = { ...this.state.pagination };
    pagination.currentPage = pageNumber;
    
    // 计算新的索引范围 (用于 API 请求)
    pagination.firstIndex = (pageNumber - 1) * pagination.itemsPerPage;
    pagination.lastIndex = pagination.firstIndex + pagination.itemsPerPage - 1;
    
    // 更新状态
    this.setState('pagination', pagination);
    
    console.debug(`[Store] Page changed to ${pageNumber}, index range: [${pagination.firstIndex}-${pagination.lastIndex}]`);
  }

  /**
   * 跳转到下一页
   * 
   * @returns {boolean} 是否成功跳转
   */
  nextPage() {
    const { currentPage, totalPages, hasMorePages } = this.state.pagination;
    
    if (!hasMorePages && currentPage >= totalPages) {
      console.debug('[Store] Already on last page');
      return false;
    }
    
    this.setPageNumber(currentPage + 1);
    return true;
  }

  /**
   * 跳转到上一页
   * 
   * @returns {boolean} 是否成功跳转
   */
  prevPage() {
    const { currentPage } = this.state.pagination;
    
    if (currentPage <= 1) {
      console.debug('[Store] Already on first page');
      return false;
    }
    
    this.setPageNumber(currentPage - 1);
    return true;
  }

  /**
   * 设置每页项目数
   * 
   * @param {number} itemsPerPage - 每页数量 (建议值: 10, 20, 50, 100)
   */
  setItemsPerPage(itemsPerPage) {
    const validSizes = [10, 20, 50, 100];
    
    if (!validSizes.includes(itemsPerPage)) {
      console.warn(`Invalid itemsPerPage: ${itemsPerPage}. Valid values: ${validSizes.join(', ')}`);
      return;
    }
    
    const pagination = { ...this.state.pagination };
    pagination.itemsPerPage = itemsPerPage;
    
    // 重置到第一页（因为每页数量改变）
    pagination.currentPage = 1;
    pagination.firstIndex = 0;
    pagination.lastIndex = itemsPerPage - 1;
    
    this.setState('pagination', pagination);
    
    console.debug(`[Store] Items per page changed to ${itemsPerPage}, reset to page 1`);
  }

  /**
   * 更新分页元数据 (总项目数、总页数等)
   * 
   * 通常在 API 响应后调用，用于更新分页控件显示
   * 
   * @param {Object} meta - 分页元数据
   * @param {number} meta.totalItems - 总项目数
   * @param {boolean} meta.hasMorePages - 是否有更多页
   * @param {boolean} meta.showPageNumbers - 是否显示分页数字
   */
  updatePaginationMeta(meta = {}) {
    const pagination = { ...this.state.pagination };
    
    if (meta.totalItems !== undefined) {
      pagination.totalItems = meta.totalItems;
      pagination.totalPages = Math.ceil(meta.totalItems / pagination.itemsPerPage);
    }
    
    if (meta.hasMorePages !== undefined) {
      pagination.hasMorePages = meta.hasMorePages;
    }
    
    if (meta.showPageNumbers !== undefined) {
      pagination.showPageNumbers = meta.showPageNumbers;
    }
    
    this.setState('pagination', pagination);
    
    console.debug(`[Store] Pagination meta updated:`, meta);
  }

  /**
   * 重置分页状态到默认值
   * 
   * 参考 NRCS nrs.js 第 622-624 行:
   * NRS.currentSubPage = "";
   * NRS.pageNumber = 1;
   * NRS.showPageNumbers = false;
   */
  resetPagination() {
    this.setState('pagination', {
      currentPage: 1,
      itemsPerPage: 50,
      hasMorePages: false,
      showPageNumbers: false,
      currentSubPage: '',
      totalItems: 0,
      totalPages: 0,
      firstIndex: 0,
      lastIndex: 49,
    });
    
    console.debug('[Store] Pagination state reset to defaults');
  }

  /**
   * 获取 API 请求参数中的分页参数
   * 
   * 方便在调用 API 时直接使用:
   * api.getBlockchainTransactions(account, store.getPaginationParams().firstIndex, store.getPaginationParams().lastIndex)
   * 
   * @returns {Object} { firstIndex, lastIndex }
   */
  getPaginationParams() {
    const { firstIndex, lastIndex } = this.state.pagination;
    return { firstIndex, lastIndex };
  }
}

// 创建全局单例实例
window.store = new Store();
