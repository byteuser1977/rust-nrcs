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
}

// 创建全局单例实例
window.store = new Store();
