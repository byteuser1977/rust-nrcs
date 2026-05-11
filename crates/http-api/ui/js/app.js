/**
 * NRCS Wallet - Application Entry Point
 * 
 * 主应用初始化和生命周期管理
 */

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
        if (!store.state.isAuthenticated) {
          router.replace('/lock');
          return;
        }
        
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
      if (Store.state.accountId || Store.state.accountRS) {
        Sidebar.updateUserInfo({
          id: Store.state.accountId,
          rsAddress: Store.state.accountRS,
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

      // 并行加载核心数据
      const [statusResult, balanceResult] = await Promise.allSettled([
        api.getBlockchainStatus(),
        store.state.account.id 
          ? api.getAccountBalance(store.state.account.id)
          : Promise.resolve(null),
      ]);

      // 处理区块链状态
      if (statusResult.status === 'fulfilled') {
        const status = statusResult.value;
        const numberOfBlocks = status.numberOfBlocks || 0;
        const lastBlockHeight = status.lastBlockHeight || 0;
        
        // 计算同步进度
        let syncStatus = 'unknown';
        let syncProgress = 0;
        
        if (status.isScanning) {
          syncStatus = 'scanning';
        } else if (lastBlockHeight < numberOfBlocks - 1) {
          syncStatus = 'downloading';
          syncProgress = numberOfBlocks > 0 ? ((lastBlockHeight + 1) / numberOfBlocks) * 100 : 0;
        } else {
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
      }

      // 处理账户余额
      if (balanceResult.status === 'fulfilled' && balanceResult.value) {
        const balance = balanceResult.value;
        store.setState('account.balanceNQT', balance.balanceNQT || 0);
        store.setState('account.unconfirmedBalanceNQT', balance.unconfirmedBalanceNQT || 0);
        Sidebar.updateUserInfo(store.state.account);
      }

      // 标记为已连接
      store.setState('connectionStatus', 'connected');
      Sidebar.updateConnectionStatus('connected');

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
   * 启动同步状态轮询
   * 定期检查区块链同步进度
   */
  startSyncPolling() {
    // 清除已有的轮询
    if (this.syncPollingTimer) {
      clearInterval(this.syncPollingTimer);
    }
    
    // 每5秒检查一次
    this.syncPollingTimer = setInterval(async () => {
      try {
        const status = await api.getBlockchainStatus();
        
        if (!status) return;
        
        const numberOfBlocks = status.numberOfBlocks || 0;
        const lastBlockHeight = status.lastBlockHeight || 0;
        
        let syncStatus = Store.state.blockchain?.syncStatus || 'unknown';
        let syncProgress = 0;
        
        if (status.isScanning) {
          syncStatus = 'scanning';
        } else if (lastBlockHeight < numberOfBlocks - 1) {
          syncStatus = 'downloading';
          syncProgress = numberOfBlocks > 0 ? ((lastBlockHeight + 1) / numberOfBlocks) * 100 : 0;
        } else {
          syncStatus = 'synced';
          
          // 同步完成，停止轮询
          this.stopSyncPolling();
          
          Toast.success('Blockchain synchronization completed!');
        }

        // 更新状态和UI
        Store.setState('blockchain', {
          ...Store.state.blockchain,
          height: lastBlockHeight,
          blockchainHeight: numberOfBlocks,
          lastBlockHeight: lastBlockHeight,
          isScanning: status.isScanning || false,
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
        
      } catch (error) {
        console.error('Sync polling error:', error);
      }
    }, 5000);  // 5秒间隔
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

// ========== 应用启动 ==========

// DOM 加载完成后初始化应用
document.addEventListener('DOMContentLoaded', () => {
  App.init();
});

// 导出供全局使用
window.App = App;
