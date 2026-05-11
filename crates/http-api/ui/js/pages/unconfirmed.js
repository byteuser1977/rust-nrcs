/**
 * NRCS Wallet - Unconfirmed Transactions Page
 * 
 * 未确认交易队列页面
 * 展示: 待确认交易列表、广播状态、取消操作等
 */

const UnconfirmedPage = {
  currentPage: 0,
  pageSize: 20,

  /**
   * 初始化页面
   */
  async init() {
    console.log('Unconfirmed transactions page initialized');
    this.render();
    await this.loadData();

    // 设置自动刷新 (每15秒)
    this.refreshInterval = setInterval(() => this.loadData(), 15000);
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
      <!-- 页面标题 -->
      <div class="page-header">
        <h1 class="page-title">
          <span class="page-title-icon">⏳</span>
          Unconfirmed Transactions
        </h1>
        <p class="page-subtitle">Transactions waiting to be included in a block</p>
      </div>

      <!-- 统计信息 -->
      <div class="grid grid-cols-3 gap-6 mb-8">
        
        <div class="stat-card stagger-item delay-1">
          <div class="stat-card-header">
            <span class="stat-card-label">Pending Count</span>
            <div class="stat-card-icon warning">⏱️</div>
          </div>
          <div class="stat-card-value" id="pending-count">0</div>
          <div class="text-sm text-muted">Transactions in queue</div>
        </div>

        <div class="stat-card stagger-item delay-2">
          <div class="stat-card-header">
            <span class="stat-card-label">Total Value</span>
            <div class="stat-card-icon primary">💰</div>
          </div>
          <div class="stat-card-value" id="total-pending-value">0.00</div>
          <div class="text-sm text-muted">NRC pending</div>
        </div>

        <div class="stat-card stagger-item delay-3">
          <div class="stat-card-header">
            <span class="stat-card-label">Avg Wait Time</span>
            <div class="stat-card-icon secondary">📊</div>
          </div>
          <div class="stat-card-value" id="avg-wait-time">-</div>
          <div class="text-sm text-muted">Estimated</div>
        </div>
      </div>

      <!-- 工具栏 -->
      <div class="card mb-6">
        <div class="card-body">
          <div class="flex items-center justify-between gap-4 flex-wrap">
            
            <div class="flex items-center gap-3">
              <button class="btn btn-primary btn-sm" onclick="UnconfirmedPage.refresh()">
                🔄 Refresh Now
              </button>
              
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" id="auto-refresh-toggle" checked 
                       onchange="UnconfirmedPage.toggleAutoRefresh(this.checked)">
                <span class="text-sm text-secondary">Auto-refresh (15s)</span>
              </label>
            </div>

            <div class="text-sm text-muted">
              Last updated: <span id="last-updated-time">-</span>
            </div>

          </div>
        </div>
      </div>

      <!-- 未确认交易列表 -->
      <div class="card stagger-item delay-4">
        <div class="card-header">
          <h3 class="card-title">
            <span class="card-title-icon">📋</span>
            Transaction Queue
          </h3>
          
          <div class="flex items-center gap-2">
            <span class="text-sm text-muted" id="tx-count-label">0 transactions</span>
          </div>
        </div>

        <div class="table-container">
          <table class="data-table">
            <thead>
              <tr>
                <th>Type</th>
                <th>Transaction ID</th>
                <th>Recipient/Sender</th>
                <th>Amount (NRC)</th>
                <th>Fee (NRC)</th>
                <th>Age</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody id="unconfirmed-tx-table-body">
              <tr>
                <td colspan="7" style="text-align: center; padding: 50px;">
                  <div class="loading-spinner"></div>
                  <p class="text-muted mt-4">Loading unconfirmed transactions...</p>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- 空状态提示 -->
        <div id="empty-state" class="hidden p-12 text-center">
          <div style="font-size: 64px; opacity: 0.5;">✅</div>
          <h3 class="text-xl font-semibold mt-6 mb-2">All Clear!</h3>
          <p class="text-secondary mb-6">No unconfirmed transactions at the moment</p>
          <p class="text-sm text-muted">New transactions will appear here once submitted</p>
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
   * 加载数据
   */
  async loadData() {
    try {
      store.setState('ui.loading', true);

      const result = await api.getUnconfirmedTransactions(
        store.state.account.id || ''
      );

      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Failed to load unconfirmed transactions');
      }

      const transactions = result.unconfirmedTransactions || [];
      
      this.updateStats(transactions);
      this.updateTable(transactions);
      this.updateLastUpdatedTime();

    } catch (error) {
      console.error('Load unconfirmed error:', error);
      Toast.show({
        type: 'error',
        title: 'Load Failed',
        message: error.message,
      });
    } finally {
      store.setState('ui.loading', false);
    }
  },

  /**
   * 更新统计信息
   */
  updateStats(transactions) {
    const countEl = document.getElementById('pending-count');
    const valueEl = document.getElementById('total-pending-value');
    const timeEl = document.getElementById('avg-wait-time');

    // 更新数量
    if (countEl) {
      countEl.textContent = transactions.length.toString();
    }

    // 计算总价值
    let totalValue = 0;
    transactions.forEach(tx => {
      totalValue += Math.abs(tx.amountNQT || 0);
    });

    if (valueEl) {
      valueEl.textContent = Formatters.formatNrc(totalValue, 2);
    }

    // 计算平均等待时间 (简化实现)
    if (timeEl && transactions.length > 0) {
      timeEl.textContent = '< 2 min';
    } else if (timeEl) {
      timeEl.textContent = '-';
    }
  },

  /**
   * 更新表格
   */
  updateTable(transactions) {
    const tbody = document.getElementById('unconfirmed-tx-table-body');
    const emptyState = document.getElementById('empty-state');
    const countLabel = document.getElementById('tx-count-label');

    if (!tbody) return;

    // 更新计数标签
    if (countLabel) {
      countLabel.textContent = `${transactions.length} transaction${transactions.length !== 1 ? 's' : ''}`;
    }

    // 处理空状态
    if (!transactions || transactions.length === 0) {
      tbody.innerHTML = '';
      if (emptyState) emptyState.classList.remove('hidden');
      return;
    }

    if (emptyState) emptyState.classList.add('hidden');

    tbody.innerHTML = transactions.map((tx, index) => {
      const typeColor = Formatters.getTransactionColor(tx.type);
      const typeName = Formatters.formatTransactionType(tx.type);
      const isOutgoing = tx.senderRS === store.state.account.rsAddress;
      const amountValue = Math.abs(tx.amountNQT || 0);
      const feeValue = tx.feeNQT || 0;

      return `
        <tr class="transition-fast hover:bg-hover"
            style="animation: fadeIn 0.3s ease ${index * 0.03}s both;">
          
          <td>
            <span class="badge badge-${typeColor}">${typeName}</span>
          </td>
          
          <td>
            <code class="text-xs text-accent cursor-pointer hover-underline"
                  onclick="navigator.clipboard.writeText('${tx.transaction}')"
                  title="Click to copy ID">
              ${Formatters.truncateAddress(tx.transaction, 10, 6)}
            </code>
          </td>
          
          <td>
            <span class="text-sm truncate d-inline-block" 
                  style="max-width: 150px;"
                  title="${isOutgoing ? tx.recipientRS : tx.senderRS}">
              ${isOutgoing 
                ? `→ ${Formatters.truncateAddress(tx.recipientRS || '-', 10, 6)}`
                : `← ${Formatters.truncateAddress(tx.senderRS || '-', 10, 6)}`
              }
            </span>
          </td>
          
          <td>
            <span class="${isOutgoing ? 'text-danger' : 'text-success'} font-mono font-medium text-sm">
              ${isOutgoing ? '-' : '+'}${Formatters.formatNrc(amountValue, 4)}
            </span>
          </td>
          
          <td class="font-mono text-sm text-muted">
            ${Formatters.formatNrc(feeValue, 4)}
          </td>
          
          <td class="text-sm text-secondary">
            <span class="badge badge-warning" style="animation: pulse 2s infinite;">
              Pending
            </span>
          </td>
          
          <td>
            <button class="btn btn-sm btn-text text-danger"
                    onclick="UnconfirmedPage.cancelTransaction('${tx.transaction}')"
                    title="Cancel transaction">
              ✕ Cancel
            </button>
          </td>
        </tr>
      `;
    }).join('');
  },

  /**
   * 更新最后更新时间
   */
  updateLastUpdatedTime() {
    const el = document.getElementById('last-updated-time');
    
    if (el) {
      el.textContent = new Date().toLocaleTimeString();
    }
  },

  /**
   * 刷新数据
   */
  async refresh() {
    await this.loadData();
  },

  /**
   * 切换自动刷新
   */
  toggleAutoRefresh(enabled) {
    if (enabled) {
      if (!this.refreshInterval) {
        this.refreshInterval = setInterval(() => this.loadData(), 15000);
      }
      Toast.show({
        type: 'info',
        title: 'Auto-refresh Enabled',
        message: 'Queue will refresh every 15 seconds',
      });
    } else {
      if (this.refreshInterval) {
        clearInterval(this.refreshInterval);
        this.refreshInterval = null;
      }
      Toast.show({
        type: 'info',
        title: 'Auto-refresh Disabled',
        message: 'Manual refresh only',
      });
    }
  },

  /**
   * 取消交易 (占位实现)
   */
  async cancelTransaction(transactionId) {
    const confirmed = confirm(`Are you sure you want to cancel this transaction?\n\nID: ${transactionId}`);

    if (!confirmed) return;

    try {
      // TODO: 实现真正的取消逻辑 (需要调用 removeUnconfirmedTransaction API)
      Toast.show({
        type: 'warning',
        title: 'Feature Coming Soon',
        message: 'Transaction cancellation will be available soon',
      });
    } catch (error) {
      Toast.show({
        type: 'error',
        title: 'Cancel Failed',
        message: error.message,
      });
    }
  },
};

// 导出供全局使用
window.UnconfirmedPage = UnconfirmedPage;
