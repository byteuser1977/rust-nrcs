/**
 * NRCS Wallet - Account Detail Page
 * 
 * 账户详情页面模块
 * 展示: 账户完整信息、余额详情、交易历史、资产持有等
 */

const AccountPage = {
  currentAccountId: null,
  
  /**
   * 初始化账户详情页
   */
  async init(params = {}) {
    console.log('Account detail page initialized', params);
    
    // 从参数获取账户 ID
    this.currentAccountId = params.id || store.state.account.id;
    
    if (!this.currentAccountId) {
      Toast.show({ type: 'error', title: 'Error', message: 'No account ID provided' });
      router.navigate('/dashboard');
      return;
    }
    
    this.render();
    await this.loadAccountData();
    
    // 设置自动刷新 (每60秒)
    this.refreshInterval = setInterval(() => this.loadAccountData(), 60000);
  },

  /**
   * 销毁页面
   */
  destroy() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
      this.refreshInterval = null;
    }
  },

  /**
   * 渲染页面结构
   */
  render() {
    const container = document.getElementById('page-content');
    
    container.innerHTML = `
      <!-- 账户头部信息卡片 -->
      <div class="account-header-card stagger-item delay-1">
        <div class="flex items-start justify-between gap-6 flex-wrap">
          
          <!-- 左侧: 基本信息 -->
          <div>
            <div class="account-avatar" id="account-avatar">N</div>
            <h1 class="account-title" id="account-title">Loading...</h1>
            <div class="account-address" id="account-address">-</div>
            
            <!-- 操作按钮组 -->
            <div class="account-actions mt-4">
              <button class="btn btn-primary btn-sm" onclick="Navbar.openSendModal()">
                💰 Send NRC
              </button>
              <button class="btn btn-secondary btn-sm" onclick="AccountPage.copyAddress()">
                📋 Copy Address
              </button>
              <button class="btn btn-ghost btn-sm" onclick="router.navigate('/transactions')">
                📜 View Transactions
              </button>
            </div>
          </div>
          
          <!-- 右侧: 余额显示 -->
          <div class="text-right">
            <div class="balance-label">Available Balance</div>
            <div class="balance-display">
              <span class="balance-value" id="account-balance">0.00</span>
              <span class="balance-currency">NRC</span>
            </div>
            
            <!-- 未确认余额 -->
            <div class="mt-3 text-sm text-muted">
              Unconfirmed: 
              <span id="unconfirmed-balance" class="font-mono">0.00 NRC</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 主要内容区域 -->
      <div class="grid grid-cols-2 gap-6 mt-8">
        
        <!-- 左侧: 账户详细信息 -->
        <div class="space-y-6 stagger-item delay-2">
          
          <!-- 公钥信息 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">🔐</span>
                Cryptographic Keys
              </h3>
            </div>
            <div class="card-body space-y-4">
              
              <div>
                <label class="text-xs text-muted uppercase tracking-wider font-semibold mb-1 block">
                  Public Key
                </label>
                <code class="text-xs break-all p-3 rounded-lg bg-elevated block font-mono"
                      id="public-key-display">
                  Loading...
                </code>
              </div>
              
              <div class="border-t border-color-light pt-4">
                <label class="text-xs text-muted uppercase tracking-wider font-semibold mb-1 block">
                  Account ID (Numeric)
                </label>
                <code class="text-sm font-mono text-accent cursor-pointer hover-underline"
                      id="numeric-account-id"
                      onclick="navigator.clipboard.writeText(this.textContent)">
                  -
                </code>
              </div>
              
              <div class="alert alert-info mt-4">
                <div class="alert-icon">ℹ️</div>
                <div class="alert-content">
                  <p class="alert-message text-xs">
                    Your public key is safe to share. Others can use it to send you payments or verify your signatures.
                  </p>
                </div>
              </div>
            </div>
          </div>

          <!-- 统计信息 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">📊</span>
                Statistics
              </h3>
            </div>
            <div class="card-body">
              <div class="grid grid-cols-2 gap-4" id="account-stats-grid">
                
                <div class="p-4 rounded-lg bg-hover transition-fast">
                  <div class="text-xs text-muted uppercase tracking-wider mb-1">
                    Total Sent
                  </div>
                  <div class="font-semibold text-danger font-mono" id="total-sent">-</div>
                </div>

                <div class="p-4 rounded-lg bg-hover transition-fast">
                  <div class="text-xs text-muted uppercase tracking-wider mb-1">
                    Total Received
                  </div>
                  <div class="font-semibold text-success font-mono" id="total-received">-</div>
                </div>

                <div class="p-4 rounded-lg bg-hover transition-fast">
                  <div class="text-xs text-muted uppercase tracking-wider mb-1">
                    Transactions
                  </div>
                  <div class="font-semibold" id="transaction-count">-</div>
                </div>

                <div class="p-4 rounded-lg bg-hover transition-fast">
                  <div class="text-xs text-muted uppercase tracking-wider mb-1">
                    First Seen
                  </div>
                  <div class="text-sm" id="first-seen-time">-</div>
                </div>

              </div>
            </div>
          </div>

        </div>

        <!-- 右侧: 最近交易和资产 -->
        <div class="space-y-6 stagger-item delay-3">
          
          <!-- 最近交易列表 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">💳</span>
                Recent Activity
              </h3>
              <a href="#/transactions" class="btn btn-sm btn-ghost">View All →</a>
            </div>
            <div class="card-body card-no-padding">
              <div class="table-container">
                <table class="data-table">
                  <thead>
                    <tr>
                      <th>Type</th>
                      <th>Amount</th>
                      <th>Time</th>
                    </tr>
                  </thead>
                  <tbody id="recent-tx-table-body">
                    <tr>
                      <td colspan="3" style="text-align: center; padding: 30px;">
                        <div class="loading-spinner"></div>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>

          <!-- 快速操作面板 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">⚡</span>
                Quick Actions
              </h3>
            </div>
            <div class="card-body">
              <div class="quick-actions">
                
                <button class="quick-action-btn" onclick="Navbar.openSendModal()">
                  <div class="icon">💸</div>
                  <div class="label">Send Money</div>
                </button>

                <button class="quick-action-btn" onclick="router.navigate('/blocks')">
                  <div class="icon">🔷</div>
                  <div class="label">View Blocks</div>
                </button>

                <button class="quick-action-btn" onclick="AccountPage.exportAccountInfo()">
                  <div class="icon">📥</div>
                  <div class="label">Export Info</div>
                </button>

                <button class="quick-action-btn" onclick="AccountPage.showQRCode()">
                  <div class="icon">📱</div>
                  <div class="label">QR Code</div>
                </button>

              </div>
            </div>
          </div>

        </div>
      </div>
    `;

    // 触发入场动画
    setTimeout(() => {
      document.querySelectorAll('.stagger-item').forEach(item => {
        item.classList.add('animate');
      });
    }, 100);
  },

  /**
   * 加载账户数据
   */
  async loadAccountData() {
    try {
      store.setState('ui.loading', true);

      // 并行请求账户信息和交易历史
      const [accountResult, transactionsResult] = await api.batchRequests([
        { requestType: 'getAccount', params: { account: this.currentAccountId } },
        { requestType: 'getBlockchainTransactions', params: { 
          account: this.currentAccountId, 
          firstIndex: 0, 
          lastIndex: 9,
        }},
      ]);

      // 处理账户信息
      if (accountResult.status === 'fulfilled' && !accountResult.value.errorCode) {
        const account = accountResult.value;
        this.updateAccountInfo(account);
      }

      // 处理交易数据
      if (transactionsResult.status === 'fulfilled' && !transactionsResult.value.errorCode) {
        const txs = transactionsResult.value.transactions || [];
        this.updateRecentTransactions(txs);
      }

    } catch (error) {
      console.error('Load account data error:', error);
      Toast.show({
        type: 'error',
        title: 'Load Failed',
        message: error.message || 'Failed to load account data',
      });
    } finally {
      store.setState('ui.loading', false);
    }
  },

  /**
   * 更新账户基本信息显示
   */
  updateAccountInfo(account) {
    // 更新标题
    const titleEl = document.getElementById('account-title');
    if (titleEl) {
      titleEl.textContent = account.name || `Account #${Formatters.truncateAddress(this.currentAccountId, 8, 6)}`;
    }

    // 更新地址
    const addressEl = document.getElementById('account-address');
    if (addressEl && account.accountRS) {
      addressEl.textContent = account.accountRS;
    }

    // 更新余额
    const balanceEl = document.getElementById('account-balance');
    if (balanceEl && account.balanceNQT !== undefined) {
      balanceEl.textContent = Formatters.formatNrc(account.balanceNQT, 4);
    }

    // 更新未确认余额
    const unconfBalanceEl = document.getElementById('unconfirmed-balance');
    if (unconfBalanceEl && account.unconfirmedBalanceNQT !== undefined) {
      unconfBalanceEl.textContent = `${Formatters.formatNrc(account.unconfirmedBalanceNQT, 4)} NRC`;
    }

    // 更新公钥
    const pkEl = document.getElementById('public-key-display');
    if (pkEl && account.publicKey) {
      pkEl.textContent = account.publicKey;
    }

    // 更新数字账户ID
    const numericIdEl = document.getElementById('numeric-account-id');
    if (numericIdEl && account.account) {
      numericIdEl.textContent = account.account.toString();
    }

    // 更新头像字母
    const avatarEl = document.getElementById('account-avatar');
    if (avatarEl && account.name) {
      avatarEl.textContent = account.name.charAt(0).toUpperCase();
    }
  },

  /**
   * 更新最近交易表格
   */
  updateRecentTransactions(transactions) {
    const tbody = document.getElementById('recent-tx-table-body');

    if (!tbody) return;

    if (!transactions || transactions.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="3" style="text-align: center; padding: 20px;">
            <span class="text-sm text-muted">No recent activity</span>
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = transactions.map(tx => {
      const isOutgoing = tx.senderRS === this.currentAccountId;
      const amountValue = Math.abs(tx.amountNQT || 0);

      return `
        <tr class="transition-fast hover:bg-hover cursor-pointer"
            onclick="TransactionsPage.showDetails('${tx.transaction}')">
          <td>
            <span class="badge badge-${Formatters.getTransactionColor(tx.type)}">
              ${Formatters.formatTransactionType(tx.type)}
            </span>
          </td>
          <td>
            <span class="${isOutgoing ? 'text-danger' : 'text-success'} font-mono font-medium text-sm">
              ${isOutgoing ? '-' : '+'}${Formatters.formatNrc(amountValue, 4)}
            </span>
          </td>
          <td class="text-sm text-muted">
            ${Formatters.formatRelativeTime(tx.timestamp)}
          </td>
        </tr>
      `;
    }).join('');
  },

  /**
   * 复制账户地址到剪贴板
   */
  async copyAddress() {
    try {
      const addressEl = document.getElementById('account-address');
      if (addressEl?.textContent) {
        await navigator.clipboard.writeText(addressEl.textContent);
        Toast.show({
          type: 'success',
          title: 'Copied',
          message: 'Account address copied to clipboard',
        });
      }
    } catch (error) {
      Toast.show({
        type: 'error',
        title: 'Copy Failed',
        message: error.message,
      });
    }
  },

  /**
   * 导出账户信息
   */
  exportAccountInfo() {
    const info = {
      accountId: this.currentAccountId,
      rsAddress: document.getElementById('account-address')?.textContent,
      publicKey: document.getElementById('public-key-display')?.textContent,
      balance: document.getElementById('account-balance')?.textContent,
      exportTime: new Date().toISOString(),
    };

    const blob = new Blob([JSON.stringify(info, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    a.download = `nrcs-account-${this.currentAccountId}.json`;
    a.click();
    
    URL.revokeObjectURL(url);

    Toast.show({
      type: 'success',
      title: 'Exported',
      message: 'Account information exported successfully',
    });
  },

  /**
   * 显示二维码 (占位实现)
   */
  showQRCode() {
    // TODO: 集成 QR Code 库生成收款二维码
    const address = document.getElementById('account-address')?.textContent;
    
    Toast.show({
      type: 'info',
      title: 'QR Code',
      message: `QR code for: ${address}`,
      duration: 5000,
    });

    alert(`QR Code feature coming soon!\n\nAddress:\n${address}`);
  },
};

// 导出供全局使用
window.AccountPage = AccountPage;
