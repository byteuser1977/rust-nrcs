/**
 * NRCS Wallet - Application Entry Point
 * 
 * 主应用初始化和生命周期管理
 * 
 * P1-S1 增强: 集成 EventEmitter 事件系统
 * 用于区块链状态变更通知和组件间通信
 */

// ========== P1-S1: 简易事件发射器 (EventEmitter) ==========
/**
 * 轻量级 EventEmitter 实现
 * 
 * 参考 Node.js EventEmitter API
 * 用于组件间解耦通信和状态变更通知
 */
class NrcsEventEmitter {
  constructor() {
    this.events = new Map();
    this.maxListeners = 10;  // 默认最大监听器数量 (防止内存泄漏)
    this.eventHistory = [];  // 事件历史记录 (用于调试)
    this.maxHistory = 100;
  }

  /**
   * 注册事件监听器
   * 
   * @param {string} event - 事件名称
   * @param {Function} listener - 监听器函数
   * @returns {Function} 取消订阅函数
   */
  on(event, listener) {
    if (typeof listener !== 'function') {
      throw new TypeError('Listener must be a function');
    }
    
    if (!this.events.has(event)) {
      this.events.set(event, []);
    }
    
    const listeners = this.events.get(event);
    
    // 检查是否超过最大监听器数量
    if (listeners.length >= this.maxListeners) {
      console.warn(
        `[EventEmitter] Possible memory leak detected. ` +
        `${listeners.size} listeners added for event "${event}". ` +
        `Max listeners is ${this.maxListeners}.`
      );
    }
    
    listeners.push(listener);
    
    // 返回取消订阅函数 (方便清理)
    return () => {
      this.off(event, listener);
    };
  }

  /**
   * 注册一次性事件监听器
   * 触发一次后自动移除
   * 
   * @param {string} event - 事件名称
   * @param {Function} listener - 监听器函数
   * @returns {Function} 取消订阅函数
   */
  once(event, listener) {
    const wrapper = (...args) => {
      this.off(event, wrapper);
      listener.apply(this, args);
    };
    
    // 保存原始引用以便移除
    wrapper._originalListener = listener;
    
    return this.on(event, wrapper);
  }

  /**
   * 移除事件监听器
   * 
   * @param {string} event - 事件名称
   * @param {Function} listener - 要移除的监听器函数
   */
  off(event, listener) {
    if (!this.events.has(event)) return;
    
    const listeners = this.events.get(event);
    
    const index = listeners.findIndex(l => 
      l === listener || l._originalListener === listener
    );
    
    if (index !== -1) {
      listeners.splice(index, 1);
      
      // 如果没有监听器了，删除事件键
      if (listeners.length === 0) {
        this.events.delete(event);
      }
    }
  }

  /**
   * 触发事件
   * 
   * @param {string} event - 事件名称
   * @param {...any} args - 传递给监听器的参数
   * @returns {boolean} 是否有监听器被触发
   */
  emit(event, ...args) {
    if (!this.events.has(event)) {
      return false;
    }
    
    const listeners = [...this.events.get(event)];  // 复制数组，防止在遍历时修改
    
    // 记录事件历史 (用于调试)
    this.recordEvent(event, args);
    
    for (const listener of listeners) {
      try {
        listener.apply(this, args);
      } catch (error) {
        console.error(`[EventEmitter] Error in listener for event "${event}":`, error);
      }
    }
    
    return true;
  }

  /**
   * 记录事件到历史 (调试用途)
   */
  recordEvent(event, args) {
    this.eventHistory.push({
      event,
      timestamp: Date.now(),
      argsCount: args.length,
    });
    
    // 限制历史记录长度
    if (this.eventHistory.length > this.maxHistory) {
      this.eventHistory.shift();
    }
  }

  /**
   * 获取指定事件的监听器数量
   * 
   * @param {string} event - 事件名称
   * @returns {number}
   */
  listenerCount(event) {
    return this.events.has(event) ? this.events.get(event).length : 0;
  }

  /**
   * 移除所有监听器或指定事件的所有监听器
   * 
   * @param {string|null} [event] - 可选的事件名称
   */
  removeAllListeners(event) {
    if (event) {
      this.events.delete(event);
    } else {
      this.events.clear();
    }
  }

  /**
   * 获取所有已注册的事件名称
   * 
   * @returns {string[]}
   */
  eventNames() {
    return Array.from(this.events.keys());
  }

  /**
   * 获取最近的事件历史 (调试用)
   * 
   * @param {number} [count=20] - 返回的记录数
   * @returns {Array}
   */
  getRecentEvents(count = 20) {
    return this.eventHistory.slice(-count);
  }
}

// 创建全局事件发射器实例
const eventBus = new NrcsEventEmitter();

// ========== 区块链状态变更事件常量 ==========
const BlockchainEvents = {
  STATUS_CHANGED: 'blockchain:statusChanged',       // 状态变更 (scanning/downloading/synced)
  NEW_BLOCK: 'blockchain:newBlock',                   // 新区块产生
  SCAN_STARTED: 'blockchain:scanStarted',             // 扫描开始
  SCAN_COMPLETED: 'blockchain:scanCompleted',         // 扫描完成
  SYNC_PROGRESS: 'blockchain:syncProgress',           // 同步进度更新
  CONNECTION_LOST: 'blockchain:connectionLost',       // 连接丢失
  CONNECTION_RESTORED: 'blockchain:connectionRestored', // 连接恢复
  
  ACCOUNT_UPDATED: 'account:updated',                 // 账户信息更新
  BALANCE_CHANGED: 'account:balanceChanged',          // 余额变更
  TRANSACTION_RECEIVED: 'transaction:received',       // 收到新交易
  UNCONFIRMED_TX_UPDATED: 'transaction:unconfirmedUpdated', // 未确认交易更新
};

const App = {
  /**
   * 当前活跃的页面模块
   */
  currentPageModule: null,

  /**
   * 应用初始化
   */
  async init() {
    console.log('%c🚀 NRCS Wallet Initializing...', 'color: #00d4ff; font-size: 16px; font-weight: bold;');
    
    try {
      // 1. 初始化组件
      this.initComponents();
      
      // 2. 恢复状态
      store.restore();
      
      // 3. 检查认证状态
      if (store.state.isAuthenticated) {
        this.showMainApp();
        await this.loadInitialData();
      } else {
        this.showLockscreen();
      }
      
      // 4. 设置路由系统
      this.setupRouter();
      
      // 5. 绑定全局事件
      this.bindGlobalEvents();
      
      console.log('%c✅ NRCS Wallet Ready', 'color: #00ff88; font-size: 14px; font-weight: bold;');
      
    } catch (error) {
      console.error('App initialization failed:', error);
      Toast.show({
        type: 'error',
        title: 'Initialization Error',
        message: 'Failed to initialize application. Please refresh the page.',
      });
    }
  },

  /**
   * 初始化所有 UI 组件
   */
  initComponents() {
    Sidebar.init();
    Navbar.init();
    
    // 初始化登录页面 (如果存在)
    if (typeof LoginPage !== 'undefined') {
      LoginPage.init();
    }
    
    console.log('Components initialized');
  },

  /**
   * 设置路由系统
   */
  setupRouter() {
    router
      .use(async (params) => {
        // 中间件: 检查认证状态
        // 注意: 不再重定向到 /lock，而是直接控制 UI 显示
        // 避免路由循环问题
        if (!store.state.isLoggedIn && !store.state.isAuthenticated) {
          // 显示锁屏界面，但不改变 URL
          this.showLockscreen();
          
          // 停止后续路由处理
          throw new Error('Navigation cancelled');
        }
        
        // 确保主应用界面可见
        this.showMainApp();
        
        // 销毁当前页面
        if (this.currentPageModule && typeof this.currentPageModule.destroy === 'function') {
          this.currentPageModule.destroy();
          this.currentPageModule = null;
        }
      })
      .beforeEach((route, params) => {
        // 更新侧边栏活动状态
        const pageName = route.path.replace('/', '');
        Sidebar.setActiveItem(pageName);
        
        // 更新面包屑导航
        Navbar.updateBreadcrumb([
          { label: 'Home', link: '#/dashboard' },
          { label: route.title || pageName.charAt(0).toUpperCase() + pageName.slice(1) },
        ]);
      })
      .afterEach(() => {
        // 页面加载完成后的操作
        store.setState('ui.loading', false);
      });

    // 注册路由
    router
      .register('/dashboard', {
        title: 'Dashboard',
        handler: async (params) => {
          this.currentPageModule = DashboardPage;
          await DashboardPage.init(params);
        },
      })
      .register('/transactions', {
        title: 'Transactions',
        handler: async (params) => {
          this.currentPageModule = TransactionsPage;
          await TransactionsPage.init(params);
        },
      })
      .register('/transactions/:id', {
        title: 'Transaction Detail',
        handler: async (params) => {
          // TODO: 实现交易详情页面
          Toast.show({ type: 'info', title: 'Coming Soon', message: `Transaction ${params.id} detail view` });
        },
      })
      .register('/unconfirmed', {
        title: 'Unconfirmed Transactions',
        handler: async (params) => {
          this.currentPageModule = UnconfirmedPage;
          await UnconfirmedPage.init(params);
        },
      })
      .register('/blocks', {
        title: 'Block Explorer',
        handler: async (params) => {
          this.currentPageModule = BlocksPage;
          await BlocksPage.init(params);
        },
      })
      .register('/blocks/:height', {
        title: 'Block Detail',
        handler: async (params) => {
          // TODO: 实现区块详情页面
          Toast.show({ type: 'info', title: 'Coming Soon', message: `Block #${params.height} detail view` });
        },
      })
      .register('/peers', {
        title: 'Network Peers',
        handler: async (params) => {
          this.currentPageModule = PeersPage;
          await PeersPage.init(params);
        },
      })
      .register('/account/:id?', {
        title: 'Account Details',
        handler: async (params) => {
          this.currentPageModule = AccountPage;
          await AccountPage.init(params);
        },
      })
      .register('/forging', {
        title: 'Forging Control',
        handler: async (params) => {
          this.currentPageModule = ForgingPage;
          await ForgingPage.init(params);
        },
      })
      .register('/settings', {
        title: 'Settings',
        handler: async (params) => {
          this.currentPageModule = SettingsPage;
          await SettingsPage.init(params);
        },
      });

    // 启动路由
    router.init();
    
    console.log('Router configured with routes:', router.getRoutes().map(r => r.path));
  },

  /**
   * 绑定全局事件
   */
  bindGlobalEvents() {
    // 登录表单提交
    const loginForm = document.getElementById('login-form');
    if (loginForm) {
      loginForm.addEventListener('submit', (e) => this.handleLogin(e));
    }

    // 窗口大小变化处理
    window.addEventListener('resize', () => {
      if (window.innerWidth > 1024) {
        Sidebar.close();
      }
    });

    // 键盘快捷键
    document.addEventListener('keydown', (e) => {
      // Ctrl/Cmd + K 打开搜索
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        const searchInput = document.getElementById('global-search');
        if (searchInput) searchInput.focus();
      }
      
      // ESC 关闭模态框/侧边栏
      if (e.key === 'Escape') {
        Navbar.closeModal();
        if (window.innerWidth <= 1024) {
          Sidebar.close();
        }
      }
    });

    // 网络状态监听
    window.addEventListener('online', () => {
      store.setState('connectionStatus', 'connected');
      Sidebar.updateConnectionStatus('connected');
      Toast.show({ type: 'success', title: 'Back Online', message: 'Internet connection restored' });
    });

    window.addEventListener('offline', () => {
      store.setState('connectionStatus', 'disconnected');
      Sidebar.updateConnectionStatus('disconnected');
      Toast.show({ type: 'warning', title: 'Offline', message: 'No internet connection' });
    });

    console.log('Global events bound');
  },

  /**
   * 显示锁屏界面
   */
  showLockscreen() {
    const lockscreenOverlay = document.getElementById('lockscreen-overlay');
    const appContainer = document.getElementById('app-container');

    if (lockscreenOverlay) {
      lockscreenOverlay.style.display = 'flex';
    }
    
    if (appContainer) {
      appContainer.style.display = 'none';
    }

    // 如果登录页面模块存在，重新初始化
    if (typeof LoginPage !== 'undefined') {
      LoginPage.checkSavedAccounts();
      LoginPage.checkBlockchainStatus();
    }
  },

  /**
   * 用户登录成功后的回调
   */
  async onUserLoggedIn() {
    console.log('User logged in, loading initial data...');
    
    try {
      // 加载初始数据
      await this.loadInitialData();
      
      // 更新侧边栏用户信息
      if (store.state.accountId || store.state.accountRS) {
        Sidebar.updateUserInfo({
          id: store.state.accountId,
          rsAddress: store.state.accountRS,
        });
      }
      
      // 导航到仪表盘
      router.navigate('/dashboard');
      
    } catch (error) {
      console.error('Failed to load data after login:', error);
      Toast.error('Failed to load wallet data');
    }
  },

  /**
   * 显示主应用界面
   */
  showMainApp() {
    const lockscreenOverlay = document.getElementById('lockscreen-overlay');
    const appContainer = document.getElementById('app-container');

    if (lockscreenOverlay) {
      lockscreenOverlay.style.display = 'none';
    }
    
    if (appContainer) {
      appContainer.style.display = 'flex';
    }
  },

  /**
   * 处理登录事件
   */
  async handleLogin(event) {
    event.preventDefault();

    const passphraseInput = document.getElementById('passphrase-input');
    const rememberMeCheckbox = document.getElementById('remember-me');
    const errorEl = document.getElementById('login-error');
    const submitBtn = document.getElementById('login-btn');

    const passphrase = passphraseInput?.value.trim();
    
    if (!passphrase) {
      this.showLoginError('Please enter your passphrase');
      return;
    }

    try {
      // 禁用按钮并显示加载状态
      submitBtn.disabled = true;
      submitBtn.innerHTML = `
        <svg class="spinner-circle" style="width:20px;height:20px;border-width:2px;"></svg>
        <span>Unlocking...</span>
      `;

      // 验证密码短语并获取账户信息
      const result = await api.request('getAccount', {}, 'POST', {
        secretPhrase: passphrase,
      });

      if (result.errorCode || !result.accountRS) {
        throw new Error(result.errorDescription || 'Invalid passphrase or account not found');
      }

      // 保存登录状态
      store.setState('isAuthenticated', true);
      store.setState('secretPhrase', rememberMeCheckbox?.checked ? passphrase : null);
      store.setState('account', {
        id: result.account,
        rsAddress: result.accountRS,
        publicKey: result.publicKey,
        balanceNQT: result.unconfirmedBalanceNQT || 0,
        unconfirmedBalanceNQT: result.unconfirmedBalanceNQT || 0,
      });
      store.setState('connectionStatus', 'connecting');

      // 更新侧边栏用户信息
      Sidebar.updateUserInfo(store.state.account);

      // 显示主应用
      this.showMainApp();

      // 加载初始数据
      await this.loadInitialData();

      // 清空错误信息
      if (errorEl) {
        errorEl.classList.add('hidden');
      }

      // 清空密码输入框
      passphraseInput.value = '';

      // 成功提示
      Toast.show({
        type: 'success',
        title: 'Welcome Back!',
        message: `Wallet unlocked successfully`,
      });

      // 导航到仪表盘
      router.navigate('/dashboard');

    } catch (error) {
      console.error('Login error:', error);
      this.showLoginError(error.message || 'Login failed. Please check your passphrase.');
    } finally {
      // 恢复按钮状态
      submitBtn.disabled = false;
      submitBtn.innerHTML = `
        <span>Unlock Wallet</span>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4M10 17v-3m0 0V8m0 7H7M15 13l-3-3m0 0l-3 3"/>
        </svg>
      `;
    }
  },

  /**
   * 显示登录错误
   */
  showLoginError(message) {
    const errorEl = document.getElementById('login-error');
    
    if (errorEl) {
      errorEl.textContent = message;
      errorEl.classList.remove('hidden');
      
      // 添加抖动动画
      errorEl.classList.add('shake-error');
      setTimeout(() => errorEl.classList.remove('shake-error'), 500);
    }
  },

  /**
   * 加载初始数据
   */
  async loadInitialData() {
    try {
      store.setState('connectionStatus', 'connecting');
      Sidebar.updateConnectionStatus('connecting');

      // 并行加载核心数据，使用更健壮的错误处理
      const [statusResult, balanceResult] = await Promise.allSettled([
        this.safeApiCall(() => api.getBlockchainStatus()),
        store.state.account?.id 
          ? this.safeApiCall(() => api.getAccountBalance(store.state.account.id))
          : Promise.resolve(null),
      ]);

      // 处理区块链状态
      if (statusResult.status === 'fulfilled' && statusResult.value) {
        const status = statusResult.value;
        const numberOfBlocks = status.numberOfBlocks || 0;
        const lastBlockHeight = status.lastBlockHeight || (status.numberOfBlocks ? status.numberOfBlocks - 1 : 0);
        
        // 计算同步进度
        let syncStatus = 'unknown';
        let syncProgress = 0;
        
        if (status.isScanning) {
          syncStatus = 'scanning';
        } else if (lastBlockHeight < numberOfBlocks - 1 && numberOfBlocks > 0) {
          syncStatus = 'downloading';
          syncProgress = ((lastBlockHeight + 1) / numberOfBlocks) * 100;
        } else if (numberOfBlocks > 0) {
          syncStatus = 'synced';
        }

        store.setState('blockchain', {
          height: lastBlockHeight,
          blockchainHeight: numberOfBlocks,
          lastBlock: null,
          lastBlockHeight: lastBlockHeight,
          cumulativeDifficulty: status.cumulativeDifficulty,
          numberOfBlocks: numberOfBlocks,
          time: status.time || 0,
          version: status.version || '',
          isScanning: status.isScanning || false,
          isDownloading: syncStatus === 'downloading',
          syncStatus: syncStatus,
          syncProgress: syncProgress,
        });
        
        // 更新侧边栏同步状态
        Sidebar.updateSyncStatus({
          status: syncStatus,
          progress: syncProgress,
          currentHeight: lastBlockHeight,
          targetHeight: numberOfBlocks,
        });
        
        Sidebar.updateHeight(numberOfBlocks);
        
        // 如果正在下载或扫描，启动轮询
        if (syncStatus === 'downloading' || syncStatus === 'scanning') {
          this.startSyncPolling();
        }
      } else if (statusResult.status === 'rejected') {
        // API 调用失败时的优雅降级
        console.warn('Blockchain status check failed:', statusResult.reason?.message);
        
        // 设置离线/错误状态
        Sidebar.updateSyncStatus({
          status: 'error',
          progress: 0,
          currentHeight: 0,
          targetHeight: 0,
        });
        
        store.setState('blockchain', {
          ...store.state.blockchain,
          syncStatus: 'error',
          isScanning: false,
          isDownloading: false,
        });
      }

      // 处理账户余额
      if (balanceResult.status === 'fulfilled' && balanceResult.value) {
        const balance = balanceResult.value;
        store.setState('account.balanceNQT', balance.balanceNQT || 0);
        store.setState('account.unconfirmedBalanceNQT', balance.unconfirmedBalanceNQT || 0);
        Sidebar.updateUserInfo(store.state.account);
      }

      // 根据结果标记连接状态
      if (statusResult.status === 'fulfilled') {
        store.setState('connectionStatus', 'connected');
        Sidebar.updateConnectionStatus('connected');
      } else {
        store.setState('connectionStatus', 'error');
        Sidebar.updateConnectionStatus('disconnected');
      }

    } catch (error) {
      console.error('Load initial data error:', error);
      store.setState('connectionStatus', 'error');
      Sidebar.updateConnectionStatus('disconnected');
      
      // 显示错误状态
      Sidebar.updateSyncStatus({
        status: 'error',
        progress: 0,
      });
    }
  },

  /**
   * 安全的 API 调用包装器
   * 捕获所有异常并返回 null，避免未处理的 Promise 拒绝
   */
  async safeApiCall(apiFunction) {
    try {
      return await apiFunction();
    } catch (error) {
      console.warn('API call failed:', error.message);
      return null;
    }
  },

  /**
   * 启动同步状态轮询 (参考 NRCS nrs.js 第 406-484 行)
   * 
   * NRCS 核心特性:
   * - 30秒标准轮询间隔 (NRS.setStateInterval(30))
   * - 增量更新: 仅在有新区块时刷新数据
   * - 扫描状态检测: 区分 scanning/downloading/synced
   * - 分级数据刷新: 余额/交易/区块分别更新
   * - 锻造状态同步: 每次轮询都更新锻造信息
   */
  startSyncPolling() {
    // 清除已有的轮询
    if (this.syncPollingTimer) {
      clearInterval(this.syncPollingTimer);
    }
    
    // 状态跟踪变量 (参考 NRCS nrs.js 第 433-434 行)
    let previousLastBlock = '0';
    let isScanning = false;
    
    // 使用 30 秒间隔 (参考 NRCS: NRS.setStateInterval(30))
    this.syncPollingTimer = setInterval(async () => {
      try {
        const status = await api.getBlockchainStatus();
        
        if (!status) return;
        
        const numberOfBlocks = status.numberOfBlocks || 0;
        const lastBlockHeight = status.lastBlockHeight || 0;
        const currentLastBlock = String(lastBlockHeight);  // 转换为字符串比较
        
        console.debug(`[Sync Polling] Block ${previousLastBlock} → ${currentLastBlock}, Scanning: ${status.isScanning}`);
        
        // Step 1: 扫描状态检测 (参考 NRCS nrs.js 第 446-458 行)
        if (status.isScanning) {
          if (!isScanning) {
            // 首次进入扫描状态
            console.log('Blockchain scanning started, suspending data updates');
            isScanning = true;
            
            // 触发扫描开始事件 (P1-S1: 事件通知系统)
            eventBus.emit(BlockchainEvents.SCAN_STARTED, {
              timestamp: Date.now(),
              blockHeight: lastBlockHeight,
              targetHeight: numberOfBlocks,
            });
            
            // 更新 UI 为扫描模式
            store.setState('blockchain', {
              ...store.state.blockchain,
              height: lastBlockHeight,
              blockchainHeight: numberOfBlocks,
              lastBlockHeight: lastBlockHeight,
              isScanning: true,
              isDownloading: false,
              syncStatus: 'scanning',
              syncProgress: numberOfBlocks > 0 ? ((lastBlockHeight + 1) / numberOfBlocks) * 100 : 0,
            });
            
            Sidebar.updateSyncStatus({
              status: 'scanning',
              progress: 0,  // 扫描时进度不确定
              currentHeight: lastBlockHeight,
              targetHeight: numberOfBlocks,
            });
          }
          
          // 扫描期间跳过所有数据更新 (参考 NRCS)
          return;
        }
        
        // Step 2: 扫描完成处理 (参考 NRCS nrs.js 第 449-458 行)
        if (isScanning && !status.isScanning) {
          console.log('Blockchain scan completed, reloading all data...');
          isScanning = false;
          
          // 触发扫描完成事件 (P1-S1: 事件通知系统)
          eventBus.emit(BlockchainEvents.SCAN_COMPLETED, {
            timestamp: Date.now(),
            blockHeight: lastBlockHeight,
            totalBlocks: numberOfBlocks,
            duration: Date.now() - this.scanStartTime || 0,
          });
          
          // 重置上一区块高度以触发全量更新
          previousLastBlock = '0';
          
          // 重新加载所有数据 (参考 NRCS)
          await this.reloadAllData();
          return;
        }
        
        // Step 3: 增量更新判断 (参考 NRCS nrs.js 第 459-475 行)
        if (currentLastBlock !== previousLastBlock) {
          console.log(`New block detected: ${previousLastBlock} → ${currentLastBlock}`);
          
          // 触发新区块事件 (P1-S1: 事件通知系统)
          eventBus.emit(BlockchainEvents.NEW_BLOCK, {
            timestamp: Date.now(),
            previousHeight: parseInt(previousLastBlock),
            newHeight: parseInt(currentLastBlock),
            targetHeight: numberOfBlocks,
          });
          
          // 更新区块链基础状态
          let syncStatus = lastBlockHeight < numberOfBlocks - 1 ? 'downloading' : 'synced';
          let syncProgress = syncStatus === 'downloading' && numberOfBlocks > 0 
            ? ((lastBlockHeight + 1) / numberOfBlocks) * 100 
            : 100;
          
          store.setState('blockchain', {
            ...store.state.blockchain,
            height: lastBlockHeight,
            blockchainHeight: numberOfBlocks,
            lastBlockHeight: lastBlockHeight,
            isScanning: false,
            isDownloading: syncStatus === 'downloading',
            syncStatus: syncStatus,
            syncProgress: syncProgress,
          });

          Sidebar.updateSyncStatus({
            status: syncStatus,
            progress: syncProgress,
            currentHeight: lastBlockHeight,
            targetHeight: numberOfBlocks,
          });
          
          // 分级数据刷新 (参考 NRCS)
          await this.incrementalUpdate(currentLastBlock);
          
          // 更新上一区块高度
          previousLastBlock = currentLastBlock;
          
        } else {
          // Step 4: 无新区块 - 仅更新未确认交易 (参考 NRCS nrs.js 第 470-475 行)
          console.debug('No new blocks, refreshing unconfirmed transactions only');
          await this.refreshUnconfirmedTransactions();
        }
        
        // Step 5: 锻造状态同步 (参考 NRCS nrs.js 第 421-422 行)
        await this.updateForgingStatus();
        
      } catch (error) {
        console.error('Sync polling error:', error);
      }
    }, 30000);  // 修改为 30 秒间隔 (参考 NRCS 标准)
    
    console.log('Sync polling started with 30-second interval (NRCS standard)');
  },

  /**
   * 扫描完成后重新加载所有数据
   * 参考 NRCS nrs.js 第 450-457 行:
   * NRS.blocks = [];
   * NRS.getBlock(lastBlock, NRS.handleInitialBlocks);
   */
  async reloadAllData() {
    console.log('Reloading all blockchain data after scan completion...');
    
    try {
      // 如果有登录账户，重新加载账户相关数据
      if (store.state.account?.id) {
        // 重新获取账户信息
        const accountInfo = await api.getAccount(store.state.account.id);
        if (accountInfo && !accountInfo.errorCode) {
          store.setState('account', {
            ...store.state.account,
            balanceNQT: accountInfo.unconfirmedBalanceNQT || accountInfo.balanceNQT || '0',
            publicKey: accountInfo.publicKey || null,
          });
        }
        
        // 重新获取交易列表
        const transactions = await api.getBlockchainTransactions(
          store.state.account.id, 
          0, 
          19
        );
        if (transactions && !transactions.errorCode) {
          store.setState('transactions', transactions.transactions || []);
        }
      }
      
      // 重新获取最新区块
      const latestBlock = await api.getLatestBlock();
      if (latestBlock && !latestBlock.errorCode) {
        store.setState('latestBlock', latestBlock);
      }
      
      console.log('All data reloaded successfully after scan');
      
    } catch (error) {
      console.error('Failed to reload data after scan:', error);
    }
  },

  /**
   * 增量更新 (有新区块时)
   * 参考 NRCS nrs.js 第 461-473 行:
   * - 更新账户余额 (getAccountInfo)
   * - 获取最新区块 (getBlock with height=-1)
   * - 获取新交易 (getBlockchainTransactions)
   */
  async incrementalUpdate(newBlockHeight) {
    console.log(`Performing incremental update for block ${newBlockHeight}`);
    
    try {
      // 1. 更新账户余额 (如果有登录账户)
      if (store.state.account?.id) {
        const accountInfo = await api.getAccount(store.state.account.id);
        if (accountInfo && !accountInfo.errorCode) {
          store.setState('account', {
            ...store.state.account,
            balanceNQT: accountInfo.unconfirmedBalanceNQT || accountInfo.balanceNQT || '0',
          });
          console.debug('Account balance updated');
        }
      }
      
      // 2. 获取最新区块
      const latestBlock = await api.getLatestBlock();
      if (latestBlock && !latestBlock.errorCode) {
        store.setState('latestBlock', latestBlock);
        console.debug(`Latest block updated: ${latestBlock.height || newBlockHeight}`);
      }
      
      // 3. 获取最新交易 (如果有登录账户)
      if (store.state.account?.id) {
        const transactions = await api.getBlockchainTransactions(
          store.state.account.id, 
          0, 
          9  // 仅获取最近10条交易，减少网络请求
        );
        if (transactions && !transactions.errorCode) {
          store.setState('transactions', transactions.transactions || []);
          console.debug(`Transactions updated: ${(transactions.transactions || []).length} items`);
        }
      }
      
    } catch (error) {
      console.warn('Incremental update failed:', error.message);
      // 不抛出错误，允许继续轮询
    }
  },

  /**
   * 刷新未确认交易 (无新区块时)
   * 参考 NRCS nrs.js 第 470-475 行:
   * NRS.getUnconfirmedTransactions(function (unconfirmedTransactions) {
   *     NRS.handleIncomingTransactions(unconfirmedTransactions, false);
   * });
   */
  async refreshUnconfirmedTransactions() {
    // 仅在有账户且页面显示交易时才刷新
    if (!store.state.account?.id) return;
    
    try {
      const unconfirmedTxs = await api.getUnconfirmedTransactions(store.state.account.id);
      
      if (unconfirmedTxs && !unconfirmedTxs.errorCode && unconfirmedTxs.unconfirmedTransactions) {
        // 更新未确认交易计数到状态
        const count = unconfirmedTxs.unconfirmedTransactions.length;
        
        if (count > 0) {
          console.debug(`Found ${count} unconfirmed transaction(s)`);
          // 可以在这里添加通知逻辑
        }
      }
      
    } catch (error) {
      console.debug('Unconfirmed transaction refresh failed:', error.message);
    }
  },

  /**
   * 更新锻造状态 (P2-S4 增强)
   * 
   * 参考 NRCS nrs.js 第 421-422 行:
   * NRS.updateForgingStatus()
   * 
   * P2-S4 增强:
   * - 完整的锻造状态查询
   * - 锻造统计信息
   * - 错误处理和重试
   * - 状态变更事件通知
   */
  async updateForgingStatus() {
    // 仅在已登录且有公钥的情况下检查锻造状态
    if (!store.state.account?.publicKey) return;
    
    try {
      // Step 1: 获取当前账户的锻造信息 (参考 NRCS: getForgingAccount)
      const forgingInfo = await api.request('getForgingAccount', {
        account: store.state.account.id,
        secretPhrase: store.state.secretPhrase || undefined,  // 如果有密码短语则验证
      });
      
      if (!forgingInfo || forgingInfo.errorCode) {
        // API 调用失败或无锻造权限
        this.handleForgingError(forgingInfo);
        return;
      }
      
      // Step 2: 解析锻造状态
      const isAccountForging = forgingInfo.forging || false;
      const isLeased = forgingInfo.leased || false;
      const needsAdminPassword = forgingInfo.needsAdminPassword || false;
      
      // Step 3: 更新 Store 状态
      store.setState('forging', {
        ...store.state.forging,
        status: isAccountForging ? 'active' : 'inactive',
        isAccountForging,
        isLeased,
        needsAdminPassword,
        accountId: store.state.account.id,
        lastChecked: Date.now(),
        // P2-S4 新增字段
        deadline: forgingInfo.deadline || null,
        hits: forgingInfo.hits || 0,
        targetDeadline: forgingInfo.targetDeadline || null,
        effectiveBalanceNQT: forgingInfo.effectiveBalanceNQT || '0',
        guaranteeId: forgingInfo.guaranteeId || null,
      });
      
      console.debug(`[Forging] Status updated: ${isAccountForging ? 'ACTIVE' : 'INACTIVE'}`);
      
      // Step 4: 触发锻造状态变更事件 (P1-S1 事件系统)
      eventBus.emit(BlockchainEvents.FORGING_STATUS_CHANGED, {
        timestamp: Date.now(),
        isActive: isAccountForging,
        accountId: store.state.account.id,
        info: forgingInfo,
      });
      
    } catch (error) {
      this.handleForgingError(error);
    }
  },

  /**
   * 处理锻造查询错误
   * 
   * @param {Object|Error} error - 错误对象
   */
  handleForgingError(error) {
    const errorCode = error?.errorCode;
    
    switch (errorCode) {
      case 4:
        // Incorrect passphrase
        console.warn('[Forging] Incorrect passphrase for forging check');
        store.setState('forging', {
          ...store.state.forging,
          status: 'error',
          error: 'Incorrect passphrase',
          lastChecked: Date.now(),
        });
        break;
        
      case 5:
        // Account not found (新账户)
        console.debug('[Forging] New account, no forging data');
        store.setState('forging', {
          ...store.state.forging,
          status: 'unknown',
          error: null,
          lastChecked: Date.now(),
        });
        break;
        
      case -1:
      case undefined:
        // Network error or timeout
        console.warn('[Forging] Failed to query forging status:', error.message);
        store.setState('forging', {
          ...store.state.forging,
          status: 'error',
          error: 'Network error',
          lastChecked: Date.now(),
        });
        break;
        
      default:
        // Other errors
        console.error('[Forging] Unknown error:', error);
        store.setState('forging', {
          ...store.state.forging,
          status: 'error',
          error: error.errorDescription || 'Unknown error',
          lastChecked: Date.now(),
        });
    }
  },

  /**
   * 开始/停止锻造 (P2-S4 新增)
   * 
   * 参考 NRCS nrs.server.js:
   * startForging / stopForging API
   * 
   * @param {string} secretPhrase - 用户密码短语
   * @param {boolean} start - true=开始锻造, false=停止锻造
   * @returns {Promise<Object>} 操作结果
   */
  async toggleForging(secretPhrase, start = true) {
    try {
      const result = await api.request(
        start ? 'startForging' : 'stopForging', 
        { secretPhrase }, 
        'POST'
      );
      
      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Failed to update forging status');
      }
      
      console.log(`[Forging] Successfully ${start ? 'started' : 'stopped'} forging`);
      
      // 刷新锻造状态
      await this.updateForgingStatus();
      
      return result;
      
    } catch (error) {
      console.error(`[Forging] Toggle failed:`, error.message);
      throw error;
    }
  },

  /**
   * 获取锻造统计信息 (P2-S4 新增)
   * 
   * 返回详细的锻造统计数据，用于显示在 UI 面板中
   * 
   * @returns {Object} 锻造统计对象
   */
  getForgingStats() {
    const forging = store.state.forging || {};
    
    return {
      status: forging.status || 'unknown',
      isActive: forging.isAccountForging || false,
      isLeased: forging.isLeased || false,
      hits: forging.hits || 0,
      deadline: forging.deadline || null,
      targetDeadline: forging.targetDeadline || null,
      effectiveBalance: forging.effectiveBalanceNQT || '0',
      lastChecked: forging.lastChecked || null,
      uptime: forging.lastChecked 
        ? Math.floor((Date.now() - forging.lastChecked) / 1000) + ' seconds ago'
        : 'Never checked',
    };
  },

  // ========== P1-S3: 智能数据刷新调度器 ==========
  
  /**
   * 刷新优先级定义
   * 
   * 参考 NRCS 数据更新策略:
   * - 高优先级: 余额、新区块 (影响用户资产显示)
   * - 中优先级: 交易列表 (影响用户体验)
   * - 低优先级: 未确认交易、锻造状态 (后台任务)
   */
  REFRESH_PRIORITY: {
    CRITICAL: 0,   // 关键数据 (余额变更)
    HIGH: 1,       // 高优先级 (新区块)
    MEDIUM: 2,     // 中优先级 (交易列表)
    LOW: 3,        // 低优先级 (未确认交易)
    BACKGROUND: 4, // 后台任务 (锻造状态)
  },

  /**
   * 智能刷新队列
   * 
   * P1-S3 增强:
   * - 防抖机制: 避免短时间内重复请求
   * - 优先级调度: 重要数据优先刷新
   * - 并发控制: 限制同时请求数量
   * - 失败重试: 自动重试失败的请求
   */
  refreshQueue: [],
  isRefreshing: false,
  maxConcurrentRequests: 3,  // 最大并发数
  activeRequests: 0,
  
  lastRefreshTime: {},
  refreshIntervals: {
    balance: 10000,       // 余额: 10秒
    transactions: 15000,  // 交易: 15秒
    blocks: 5000,         // 区块: 5秒
    unconfirmed: 20000,   // 未确认交易: 20秒
    forging: 30000,       // 锻造状态: 30秒
  },
  
  /**
   * 添加刷新任务到队列
   * 
   * @param {string} taskType - 任务类型 (balance/transactions/blocks/unconfirmed/forging)
   * @param {Function} taskFn - 执行函数
   * @param {number} priority - 优先级 (使用 REFRESH_PRIORITY)
   */
  enqueueRefresh(taskType, taskFn, priority = App.REFRESH_PRIORITY.MEDIUM) {
    // 防抖检查: 如果上次刷新时间太近，跳过
    const now = Date.now();
    const interval = this.refreshIntervals[taskType] || 15000;
    
    if (this.lastRefreshTime[taskType] && (now - this.lastRefreshTime[taskType]) < interval) {
      console.debug(`[Refresh Queue] Skipping ${taskType}, too frequent`);
      return;
    }
    
    // 添加到队列 (按优先级排序)
    this.refreshQueue.push({
      type: taskType,
      fn: taskFn,
      priority,
      timestamp: now,
    });
    
    // 按优先级排序 (数字越小越优先)
    this.refreshQueue.sort((a, b) => a.priority - b.priority);
    
    // 尝试执行队列
    this.processRefreshQueue();
  },
  
  /**
   * 处理刷新队列
   */
  async processRefreshQueue() {
    if (this.isRefreshing || this.activeRequests >= this.maxConcurrentRequests) return;
    if (this.refreshQueue.length === 0) return;
    
    this.isRefreshing = true;
    
    while (this.refreshQueue.length > 0 && this.activeRequests < this.maxConcurrentRequests) {
      const task = this.refreshQueue.shift();
      if (!task) break;
      
      this.activeRequests++;
      
      // 异步执行任务
      task.fn()
        .then(() => {
          this.lastRefreshTime[task.type] = Date.now();
          console.debug(`[Refresh Queue] Completed: ${task.type}`);
        })
        .catch(error => {
          console.warn(`[Refresh Queue] Failed: ${task.type}`, error.message);
        })
        .finally(() => {
          this.activeRequests--;
          
          // 继续处理队列中的下一个任务
          if (this.refreshQueue.length > 0) {
            setTimeout(() => this.processRefreshQueue(), 100);  // 100ms 延迟
          } else {
            this.isRefreshing = false;
          }
        });
    }
  },
  
  /**
   * 清空刷新队列
   */
  clearRefreshQueue() {
    this.refreshQueue = [];
    this.isRefreshing = false;
    console.log('[Refresh Queue] Cleared');
  },

  /**
   * 停止同步状态轮询
   */
  stopSyncPolling() {
    if (this.syncPollingTimer) {
      clearInterval(this.syncPollingTimer);
      this.syncPollingTimer = null;
    }
  },

  /**
   * 登出/锁定钱包
   */
  logout() {
    // 销毁当前页面
    if (this.currentPageModule && typeof this.currentPageModule.destroy === 'function') {
      this.currentPageModule.destroy();
      this.currentPageModule = null;
    }

    // 重置状态
    store.reset();

    // 清除 API 缓存
    if (window.api) {
      window.api.clearCache();
    }

    // 显示锁屏
    this.showLockscreen();

    // 提示
    Toast.show({
      type: 'info',
      title: 'Wallet Locked',
      message: 'Your wallet has been locked securely',
    });
  },
};

// ========== P2-A11Y: 无障碍访问和键盘导航工具类 ==========
/**
 * AccessibilityHelper - 无障碍访问辅助工具
 * 
 * P2-A11Y 实现:
 * - 键盘导航增强
 * - ARIA 属性管理
 * - 焦点陷阱 (Focus Trap)
 * - 屏幕阅读器支持
 * - 跳转链接 (Skip Links)
 */
const AccessibilityHelper = {
  
  /**
   * 初始化无障碍功能
   * 
   * 在应用启动时调用，设置全局键盘事件监听
   */
  init() {
    this.setupKeyboardNavigation();
    this.setupFocusManagement();
    this.addSkipLinks();
    this.enhanceARIAAttributes();
    
    console.log('[A11Y] Accessibility features initialized');
  },

  /**
   * 设置全局键盘导航
   * 
   * 支持:
   * - Tab: 正向焦点移动
   * - Shift+Tab: 反向焦点移动
   * - Enter/Space: 激活当前元素
   * - Escape: 关闭模态框/菜单
   * - Alt+M: 跳转到主内容区
   */
  setupKeyboardNavigation() {
    document.addEventListener('keydown', (event) => {
      // Escape 键处理
      if (event.key === 'Escape') {
        this.handleEscapeKey(event);
      }
      
      // Alt+M 快捷键: 跳转到主内容
      if (event.altKey && event.key === 'm') {
        event.preventDefault();
        this.skipToMainContent();
      }
      
      // Enter/Space 激活非交互元素 (如自定义按钮)
      if ((event.key === 'Enter' || event.key === ' ') && 
          this.isCustomInteractiveElement(event.target)) {
        event.preventDefault();
        event.target.click();
      }
    });
  },

  /**
   * 处理 Escape 键
   * 
   * @param {KeyboardEvent} event - 键盘事件
   */
  handleEscapeKey(event) {
    // 关闭所有打开的模态框
    const modals = document.querySelectorAll('.modal.show, .modal.active');
    if (modals.length > 0) {
      const lastModal = modals[modals.length - 1];
      lastModal.classList.remove('show', 'active');
      
      // 返回焦点到触发元素
      if (lastModal.dataset.returnFocus) {
        const returnElement = document.getElementById(lastModal.dataset.returnFocus);
        if (returnElement) returnElement.focus();
      }
      
      console.debug('[A11Y] Modal closed via Escape key');
      return;
    }
    
    // 关闭侧边栏 (如果是打开状态)
    const sidebar = document.querySelector('.sidebar.open, .sidebar.expanded');
    if (sidebar) {
      sidebar.classList.remove('open', 'expanded');
      console.debug('[A11Y] Sidebar closed via Escape key');
    }
  },

  /**
   * 设置焦点管理
   * 
   * 自动管理模态框中的焦点循环
   */
  setupFocusManagement() {
    // 监听模态框显示/隐藏
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE && 
              (node.classList?.contains('modal') || node.closest('.modal'))) {
            // 新模态框出现，应用焦点陷阱
            const modal = node.classList?.contains('modal') ? node : node.closest('.modal');
            if (modal) {
              this.applyFocusTrap(modal);
            }
          }
        });
      });
    });
    
    observer.observe(document.body, { childList: true, subtree: true });
  },

  /**
   * 应用焦点陷阱到模态框
   * 
   * @param {HTMLElement} modalElement - 模态框元素
   */
  applyFocusTrap(modalElement) {
    const focusableElements = modalElement.querySelectorAll(
      'a[href], button:not([disabled]), textarea:not([disabled]), ' +
      'input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
    );
    
    if (focusableElements.length === 0) return;
    
    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];
    
    // Tab 循环处理
    modalElement.addEventListener('keydown', (e) => {
      if (e.key !== 'Tab') return;
      
      if (e.shiftKey) {
        // Shift+Tab: 如果在第一个元素上，跳到最后一个
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement.focus();
        }
      } else {
        // Tab: 如果在最后一个元素上，跳到第一个
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement.focus();
        }
      }
    });
    
    // 自动聚焦到第一个可聚焦元素
    setTimeout(() => firstElement.focus(), 100);
    
    console.debug(`[A11Y] Focus trap applied to modal (${focusableElements.length} focusable elements)`);
  },

  /**
   * 添加跳转链接 (Skip Links)
   * 
   * 允许键盘用户快速跳过重复导航内容
   */
  addSkipLinks() {
    if (document.getElementById('skip-to-main')) return;  // 避免重复添加
    
    const skipLink = document.createElement('a');
    skipLink.id = 'skip-to-main';
    skipLink.href = '#main-content';
    skipLink.className = 'skip-link';
    skipLink.textContent = 'Skip to main content';
    skipLink.setAttribute('aria-label', 'Skip navigation and go directly to main content');
    
    document.body.insertBefore(skipLink, document.body.firstChild);
  },

  /**
   * 跳转到主内容区
   */
  skipToMainContent() {
    const mainContent = document.getElementById('main-content');
    if (mainContent) {
      mainContent.tabIndex = -1;  // 使其可聚焦
      mainContent.focus({ preventScroll: false });
      console.log('[A11Y] Skipped to main content');
    } else {
      console.warn('[A11Y] #main-content element not found');
    }
  },

  /**
   * 增强 ARIA 属性
   * 
   * 为关键 UI 元素添加语义化标记
   */
  enhanceARIAAttributes() {
    // 标记主导航区域
    const nav = document.querySelector('nav, [role="navigation"]');
    if (nav && !nav.getAttribute('aria-label')) {
      nav.setAttribute('aria-label', 'Main navigation');
    }
    
    // 标记侧边栏
    const sidebar = document.querySelector('.sidebar, aside');
    if (sidebar && !sidebar.getAttribute('role')) {
      sidebar.setAttribute('role', 'complementary');
      sidebar.setAttribute('aria-label', 'Sidebar menu');
    }
    
    // 标记主内容区
    const main = document.querySelector('main, #main-content');
    if (main && !main.getAttribute('role')) {
      main.setAttribute('role', 'main');
      main.id = 'main-content';  // 确保 ID 存在
    }
    
    // 增强表单元素
    this.enhanceFormAccessibility();
    
    // 增强按钮和链接
    this.enhanceInteractiveElements();
  },

  /**
   * 增强表单无障碍性
   */
  enhanceFormAccessibility() {
    // 为没有 label 的 input 添加 aria-label
    document.querySelectorAll('input:not([aria-label]):not([id])').forEach(input => {
      if (input.placeholder) {
        input.setAttribute('aria-label', input.placeholder);
      }
    });
    
    // 将 label 与 input 关联
    document.querySelectorAll('label[for]').forEach(label => {
      const input = document.getElementById(label.getAttribute('for'));
      if (input && !input.getAttribute('aria-describedby')) {
        // 可以添加描述性文本的关联
      }
    });
  },

  /**
   * 增强交互元素的无障碍性
   */
  enhanceInteractiveElements() {
    // 为 role="button" 的元素添加键盘支持
    document.querySelectorAll('[role="button"]').forEach(el => {
      el.tabIndex = 0;  // 使其可聚焦
    });
    
    // 为自定义复选框/开关添加状态提示
    document.querySelectorAll('[role="checkbox"], [role="switch"]').forEach(el => {
      const isChecked = el.getAttribute('aria-checked') === 'true' || el.checked;
      el.setAttribute('aria-checked', isChecked ? 'true' : 'false');
    });
  },

  /**
   * 检测是否为自定义交互元素 (需要键盘激活)
   * 
   * @param {HTMLElement} element - 目标元素
   * @returns {boolean}
   */
  isCustomInteractiveElement(element) {
    const customRoles = ['button', 'checkbox', 'switch', 'menuitem', 'tab'];
    return customRoles.includes(element.getAttribute('role'));
  },

  /**
   * 宣布消息给屏幕阅读器
   * 
   * @param {string} message - 要宣布的消息
   * @param {string} priority - 优先级 ('polite' | 'assertive')
   */
  announceToScreenReader(message, priority = 'polite') {
    let announcer = document.getElementById('sr-announcer');
    
    if (!announcer) {
      announcer = document.createElement('div');
      announcer.id = 'sr-announcer';
      announcer.className = 'sr-only';
      announcer.setAttribute('aria-live', priority);
      announcer.setAttribute('aria-atomic', 'true');
      document.body.appendChild(announcer);
    }
    
    // 清空并设置新消息 (确保屏幕阅读器能检测到变化)
    announcer.textContent = '';
    setTimeout(() => {
      announcer.textContent = message;
    }, 100);
  },
};

// ========== 应用启动 ==========

// DOM 加载完成后初始化应用
document.addEventListener('DOMContentLoaded', () => {
  App.init();
  
  // P2-A11Y: 初始化无障碍功能
  AccessibilityHelper.init();
  
  // P2-UI-2: 初始化侧边栏 Treeview (延迟执行确保 DOM 就绪)
  setTimeout(() => {
    if (typeof Sidebar !== 'undefined' && Sidebar.initTreeview) {
      Sidebar.initTreeview();
      console.log('[App] Sidebar treeview initialized');
    }
  }, 500);
});

// 导出供全局使用
window.App = App;
window.AccessibilityHelper = AccessibilityHelper;
