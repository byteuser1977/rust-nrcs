/**
 * NRCS Wallet - Settings Page
 * 
 * 设置页面模块
 * 展示: 账户信息编辑、偏好设置、系统配置等
 */

const SettingsPage = {
  /**
   * 初始化设置页面
   */
  async init() {
    console.log('Settings page initialized');
    this.render();
    await this.loadAccountData();
    
    // 初始化终端控制台
    this.initTerminal();
  },

  /**
   * 销毁页面
   */
  destroy() {
    // 清理工作
  },

  /**
   * 渲染页面结构
   */
  render() {
    const container = document.getElementById('page-content');
    
    container.innerHTML = `
      <!-- 页面标题 -->
      <div class="page-header">
        <h1 class="page-title">
          <span class="page-title-icon">⚙️</span>
          Settings
        </h1>
        <p class="page-subtitle">Manage your account preferences and configuration</p>
      </div>

      <!-- 设置内容网格 -->
      <div class="grid grid-cols-2 gap-6">
        
        <!-- 左侧: 账户设置 -->
        <div class="space-y-6 stagger-item delay-1">
          
          <!-- 账户信息卡片 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">👤</span>
                Account Information
              </h3>
            </div>
            <div class="card-body">
              <form id="account-info-form" class="space-y-5">
                
                <div class="form-group">
                  <label for="account-name-input" class="form-label">Account Name</label>
                  <input type="text" 
                         id="account-name-input" 
                         class="form-input" 
                         placeholder="Enter display name"
                         value="">
                </div>

                <div class="form-group">
                  <label for="account-desc-input" class="form-label">Description</label>
                  <textarea id="account-desc-input" 
                            class="form-textarea" 
                            rows="3"
                            placeholder="Add a description..."></textarea>
                </div>

                <button type="submit" class="btn btn-primary" id="save-account-btn">
                  Save Changes
                </button>
              </form>
            </div>
          </div>

          <!-- 公钥显示 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">🔐</span>
                Public Key
              </h3>
            </div>
            <div class="card-body">
              <div class="flex items-center gap-3 p-4 rounded-lg bg-elevated font-mono text-xs break-all"
                   id="public-key-display">
                Loading...
              </div>
              <p class="text-xs text-muted mt-3">
                This is your public key. Share it freely to receive payments.
              </p>
            </div>
          </div>

          <!-- 别名管理 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">🏷️</span>
                Aliases
              </h3>
            </div>
            <div class="card-body">
              <div id="aliases-list" class="space-y-2 mb-4">
                <!-- 动态加载 -->
              </div>
              
              <form id="add-alias-form" class="flex gap-2">
                <input type="text" 
                       class="form-input flex-1" 
                       placeholder="New alias name..."
                       id="new-alias-input">
                <button type="submit" class="btn btn-primary">Add</button>
              </form>
            </div>
          </div>

        </div>

        <!-- 右侧: 系统设置 -->
        <div class="space-y-6 stagger-item delay-2">
          
          <!-- 安全设置 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">🔒</span>
                Security Settings
              </h3>
            </div>
            <div class="card-body space-y-5">
              
              <div class="flex items-center justify-between">
                <div>
                  <div class="font-medium">Auto-lock Timeout</div>
                  <div class="text-sm text-muted mt-1">Automatically lock wallet after inactivity</div>
                </div>
                <select class="form-select" style="width: auto;" id="autolock-timeout">
                  <option value="0">Never</option>
                  <option value="300">5 minutes</option>
                  <option value="600" selected>10 minutes</option>
                  <option value="1800">30 minutes</option>
                  <option value="3600">1 hour</option>
                </select>
              </div>

              <div class="border-t border-color-light pt-4"></div>

              <div class="flex items-center justify-between">
                <div>
                  <div class="font-medium">Remember Passphrase</div>
                  <div class="text-sm text-muted mt-1">Save passphrase locally (not recommended)</div>
                </div>
                <label class="toggle-switch" id="remember-toggle">
                  <input type="checkbox" id="remember-passphrase-checkbox">
                </label>
              </div>

              <div class="border-t border-color-light pt-4"></div>

              <button class="btn btn-danger btn-sm" onclick="SettingsPage.changePassphrase()">
                Change Passphrase
              </button>
            </div>
          </div>

          <!-- 显示设置 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">🎨</span>
                Display Preferences
              </h3>
            </div>
            <div class="card-body space-y-5">
              
              <div class="flex items-center justify-between">
                <div>
                  <div class="font-medium">Currency Display</div>
                  <div class="text-sm text-muted mt-1">How to show amounts</div>
                </div>
                <select class="form-select" style="width: auto;" id="currency-display">
                  <option value="nrc" selected>NRC (8 decimals)</option>
                  <option value="nqt">NQT (raw units)</option>
                </select>
              </div>

              <div class="border-t border-color-light pt-4"></div>

              <div class="flex items-center justify-between">
                <div>
                  <div class="font-medium">Date Format</div>
                  <div class="text-sm text-muted mt-1">Preferred date/time format</div>
                </div>
                <select class="form-select" style="width: auto;" id="date-format">
                  <option value="relative" selected>Relative (e.g., "2m ago")</option>
                  <option value="absolute">Absolute (e.g., "Jan 15, 2024")</option>
                </select>
              </div>

              <div class="border-t border-color-light pt-4"></div>

              <div class="flex items-center justify-between">
                <div>
                  <div class="font-medium">Animations</div>
                  <div class="text-sm text-muted mt-1">Enable UI animations and transitions</div>
                </div>
                <label class="toggle-switch active" id="animations-toggle">
                  <input type="checkbox" checked id="enable-animations-checkbox">
                </label>
              </div>
            </div>
          </div>

          <!-- 网络状态 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">☁️</span>
                Network Status
              </h3>
            </div>
            <div class="card-body">
              <div class="space-y-3" id="network-status-list">
                
                <div class="flex items-center justify-between p-3 rounded-md bg-hover">
                  <span class="text-sm">Connection Status</span>
                  <span class="badge badge-success" id="net-status-badge">Connected</span>
                </div>

                <div class="flex items-center justify-between p-3 rounded-md bg-hover">
                  <span class="text-sm">API Endpoint</span>
                  <code class="text-xs text-secondary" id="api-endpoint-display">${window.api?.baseUrl || '/nrcs'}</code>
                </div>

                <div class="flex items-center justify-between p-3 rounded-md bg-hover">
                  <span class="text-sm">Requests Made</span>
                  <span class="text-sm font-mono" id="requests-count-display">0</span>
                </div>

              </div>
            </div>
          </div>

          <!-- 危险区域 -->
          <div class="card" style="border-color: rgba(255, 71, 87, 0.3);">
            <div class="card-header">
              <h3 class="card-title text-danger">
                <span class="card-title-icon" style="background: var(--gradient-danger);">⚠️</span>
                Danger Zone
              </h3>
            </div>
            <div class="card-body">
              <div class="space-y-3">
                <p class="text-sm text-secondary">
                  These actions are irreversible. Please proceed with caution.
                </p>
                
                <div class="flex gap-3">
                  <button class="btn btn-ghost btn-sm" onclick="SettingsPage.clearCache()">
                    Clear Cache
                  </button>
                  <button class="btn btn-danger btn-sm" onclick="App.logout()">
                    Lock & Logout
                  </button>
                </div>
              </div>
            </div>
          </div>

        </div>
      </div>

      <!-- 终端控制台 -->
      <div class="mt-8 stagger-item delay-3">
        <div class="page-header mb-4">
          <h2 class="text-lg font-semibold flex items-center gap-2">
            <span>⌨️</span>
            Console Terminal
          </h2>
          <p class="text-sm text-muted mt-1">
            Execute commands: clear, save, history, help
          </p>
        </div>
        <div id="terminal-container"></div>
      </div>
    `;

    // 触发入场动画
    setTimeout(() => {
      document.querySelectorAll('.stagger-item').forEach(item => {
        item.classList.add('animate');
      });
    }, 100);

    // 绑定表单事件
    this.bindEvents();
  },

  /**
   * 绑定事件监听器
   */
  bindEvents() {
    // 账户信息表单提交
    const accountForm = document.getElementById('account-info-form');
    if (accountForm) {
      accountForm.addEventListener('submit', (e) => this.handleAccountInfoSubmit(e));
    }

    // 别名表单提交
    const aliasForm = document.getElementById('add-alias-form');
    if (aliasForm) {
      aliasForm.addEventListener('submit', (e) => this.handleAliasSubmit(e));
    }

    // 切换开关事件
    this.setupToggleSwitches();

    // 选择框变化监听
    const selects = ['autolock-timeout', 'currency-display', 'date-format'];
    selects.forEach(id => {
      const el = document.getElementById(id);
      if (el) {
        el.addEventListener('change', () => this.savePreferences());
      }
    });
  },

  /**
   * 设置切换开关交互
   */
  setupToggleSwitches() {
    const toggles = [
      { toggleId: 'remember-toggle', checkboxId: 'remember-passphrase-checkbox' },
      { toggleId: 'animations-toggle', checkboxId: 'enable-animations-checkbox' },
    ];

    toggles.forEach(({ toggleId, checkboxId }) => {
      const toggle = document.getElementById(toggleId);
      const checkbox = document.getElementById(checkboxId);

      if (toggle && checkbox) {
        toggle.addEventListener('click', (e) => {
          if (e.target === toggle || e.target.tagName !== 'INPUT') {
            checkbox.checked = !checkbox.checked;
            toggle.classList.toggle('active', checkbox.checked);
            this.savePreferences();
          }
        });

        checkbox.addEventListener('change', () => {
          toggle.classList.toggle('active', checkbox.checked);
          this.savePreferences();
        });
      }
    });
  },

  /**
   * 加载账户数据
   */
  async loadAccountData() {
    try {
      store.setState('ui.loading', true);

      // 获取账户信息
      if (store.state.account.id) {
        const result = await api.getAccount(store.state.account.id);

        if (!result.errorCode && result.account) {
          const account = result.account;

          // 填充表单
          const nameInput = document.getElementById('account-name-input');
          const descInput = document.getElementById('account-desc-input');

          if (nameInput && account.name) {
            nameInput.value = account.name;
          }
          if (descInput && account.description) {
            descInput.value = account.description;
          }

          // 显示公钥
          const pkDisplay = document.getElementById('public-key-display');
          if (pkDisplay && account.publicKey) {
            pkDisplay.textContent = account.publicKey;
          }
        }
      }

      // 加载别名列表
      await this.loadAliases();

      // 更新网络状态
      this.updateNetworkStatus();

    } catch (error) {
      console.error('Load account data error:', error);
      Toast.show({
        type: 'error',
        title: 'Error',
        message: 'Failed to load account settings',
      });
    } finally {
      store.setState('ui.loading', false);
    }
  },

  /**
   * 加载别名列表
   */
  async loadAliases() {
    try {
      // TODO: 实现获取别名 API
      const aliasesListEl = document.getElementById('aliases-list');
      
      if (aliasesListEl) {
        aliasesListEl.innerHTML = `
          <div class="text-sm text-muted text-center py-4">
            No aliases configured yet
          </div>
        `;
      }
    } catch (error) {
      console.error('Load aliases error:', error);
    }
  },

  /**
   * 处理账户信息保存
   */
  async handleAccountInfoSubmit(event) {
    event.preventDefault();

    const name = document.getElementById('account-name-input')?.value.trim();
    const description = document.getElementById('account-desc-input')?.value.trim();

    const saveBtn = document.getElementById('save-account-btn');

    try {
      saveBtn.disabled = true;
      saveBtn.innerHTML = '<span>Saving...</span>';

      const result = await api.setAccountInfo({
        secretPhrase: store.state.secretPhrase,
        ...(name ? { name } : {}),
        ...(description ? { description } : {}),
      });

      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Failed to save');
      }

      Toast.show({
        type: 'success',
        title: 'Saved',
        message: 'Account information updated successfully',
      });

    } catch (error) {
      console.error('Save account info error:', error);
      Toast.show({
        type: 'error',
        title: 'Error',
        message: error.message,
      });
    } finally {
      saveBtn.disabled = false;
      saveBtn.innerHTML = 'Save Changes';
    }
  },

  /**
   * 处理添加别名
   */
  async handleAliasSubmit(event) {
    event.preventDefault();

    const input = document.getElementById('new-alias-input');
    const aliasName = input?.value.trim();

    if (!aliasName) {
      Toast.show({ type: 'warning', title: 'Warning', message: 'Please enter an alias name' });
      return;
    }

    try {
      const result = await api.setAlias({
        aliasName,
        secretPhrase: store.state.secretPhrase,
      });

      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Failed to create alias');
      }

      Toast.show({
        type: 'success',
        title: 'Created',
        message: `Alias "${aliasName}" created successfully`,
      });

      input.value = '';
      await this.loadAliases();

    } catch (error) {
      console.error('Create alias error:', error);
      Toast.show({
        type: 'error',
        title: 'Error',
        message: error.message,
      });
    }
  },

  /**
   * 保存用户偏好
   */
  savePreferences() {
    const preferences = {
      autolockTimeout: document.getElementById('autolock-timeout')?.value,
      currencyDisplay: document.getElementById('currency-display')?.value,
      dateFormat: document.getElementById('date-format')?.value,
      rememberPassphrase: document.getElementById('remember-passphrase-checkbox')?.checked,
      enableAnimations: document.getElementById('enable-animations-checkbox')?.checked,
    };

    localStorage.setItem('nrcs_preferences', JSON.stringify(preferences));
    
    console.log('Preferences saved:', preferences);
  },

  /**
   * 更新网络状态显示
   */
  updateNetworkStatus() {
    const statusBadge = document.getElementById('net-status-badge');
    const requestsCount = document.getElementById('requests-count-display');

    if (statusBadge) {
      statusBadge.className = `badge ${store.state.connectionStatus === 'connected' ? 'badge-success' : 'badge-warning'}`;
      statusBadge.textContent = store.state.connectionStatus.charAt(0).toUpperCase() + 
                                  store.state.connectionStatus.slice(1);
    }

    if (requestsCount && window.api) {
      const stats = window.api.getStats();
      requestsCount.textContent = stats.totalRequests.toString();
    }
  },

  /**
   * 更改密码短语
   */
  changePassphrase() {
    alert('Password change feature coming soon!');
  },

  /**
   * 清除缓存
   */
  clearCache() {
    if (confirm('Are you sure you want to clear all cached data?')) {
      localStorage.removeItem('nrcs_wallet_state');
      localStorage.removeItem('nrcs_preferences');
      
      if (window.api) {
        window.api.clearCache();
      }
      
      Toast.show({
        type: 'success',
        title: 'Cleared',
        message: 'All cache data has been cleared',
      });
    }
  },

  /**
   * 初始化终端控制台
   */
  initTerminal() {
    // 等待 DOM 更新后初始化终端
    setTimeout(() => {
      if (typeof TerminalConsole !== 'undefined') {
        TerminalConsole.init('terminal-container');
        
        // 注册自定义命令示例
        TerminalConsole.registerCommand('status', () => {
          TerminalConsole.printLine('\n📊 NRCS Wallet Status:', 'header');
          TerminalConsole.printLine(`  Connection: ${store.state.connectionStatus}`, 'info');
          TerminalConsole.printLine(`  Account: ${store.state.accountRS || 'Not logged in'}`, 'output');
          TerminalConsole.printLine(`  Block Height: ${store.state.blockchain?.height || 0}`, 'output');
          TerminalConsole.printLine(`  Sync Status: ${store.state.blockchain?.syncStatus || 'unknown'}`, 'output');
          TerminalConsole.printLine('', 'empty');
        });

        TerminalConsole.registerCommand('balance', async () => {
          try {
            TerminalConsole.printLine('Fetching balance...', 'info');
            const result = await api.getAccountBalance(store.state.account.id);
            
            if (result && !result.errorCode) {
              const balanceNQT = result.balanceNQT || 0;
              const unconfirmedBalanceNQT = result.unconfirmedBalanceNQT || 0;
              
              TerminalConsole.printLine('\n💰 Account Balance:', 'header');
              TerminalConsole.printLine(`  Confirmed:     ${(balanceNQT / 1e8).toFixed(8)} NRC`, 'success');
              TerminalConsole.printLine(`  Unconfirmed:   ${(unconfirmedBalanceNQT / 1e8).toFixed(8)} NRC`, 'success');
              TerminalConsole.printLine(`  Difference:    ${((unconfirmedBalanceNQT - balanceNQT) / 1e8).toFixed(8)} NRC`, 'output');
            } else {
              TerminalConsole.printLine('Failed to fetch balance', 'error');
            }
          } catch (error) {
            TerminalConsole.printLine(`Error: ${error.message}`, 'error');
          }
        });

        TerminalConsole.registerCommand('blockchain', async () => {
          try {
            const status = await api.getBlockchainStatus();
            
            if (status && !status.errorCode) {
              TerminalConsole.printLine('\n⛓️ Blockchain Status:', 'header');
              TerminalConsole.printLine(`  Height:       ${status.numberOfBlocks || 0}`, 'output');
              TerminalConsole.printLine(`  Last Block:    ${status.lastBlockHeight || status.numberOfBlocks || 0}`, 'output');
              TerminalConsole.printLine(`  Scanning:      ${status.isScanning ? 'Yes' : 'No'}`, 'info');
              TerminalConsole.printLine(`  Application:   ${status.version || 'Unknown'}`, 'output');
              TerminalConsole.printLine(`  Time:         ${new Date((status.time || 0) * 1000).toLocaleString()}`, 'output');
            } else {
              TerminalConsole.printLine('Failed to fetch blockchain status', 'error');
            }
          } catch (error) {
            TerminalConsole.printLine(`Error: ${error.message}`, 'error');
          }
        });

        console.log('Terminal initialized with custom commands');
      }
    }, 100);
  },
};

// 导出供全局使用
window.SettingsPage = SettingsPage;
