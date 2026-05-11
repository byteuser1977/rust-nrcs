/**
 * NRCS Wallet - Transactions Page
 * 
 * 交易列表页面模块
 * 展示: 交易历史、类型筛选、搜索、分页等
 */

const TransactionsPage = {
  currentPage: 0,
  pageSize: 20,
  filterType: 'all',
  
  /**
   * 初始化交易页面
   */
  async init(params = {}) {
    console.log('Transactions page initialized', params);
    
    // 从参数中获取初始状态
    if (params.type) this.filterType = params.type;
    if (params.page) this.currentPage = parseInt(params.page);
    
    this.render();
    await this.loadData();
    
    // 设置自动刷新 (每60秒)
    this.refreshInterval = setInterval(() => this.loadData(), 60000);
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
   * 刷新数据 (供外部调用)
   */
  async refresh() {
    await this.loadData();
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
          <span class="page-title-icon">💳</span>
          Transactions
        </h1>
        <p class="page-subtitle">View and manage your transaction history</p>
      </div>

      <!-- 工具栏 -->
      <div class="card mb-6">
        <div class="card-body">
          <div class="flex items-center justify-between gap-4 flex-wrap">
            
            <!-- 类型筛选标签 -->
            <div class="flex gap-2" id="filter-tabs">
              <button class="btn btn-sm ${this.filterType === 'all' ? 'btn-primary' : 'btn-ghost'}" 
                      data-filter="all" onclick="TransactionsPage.setFilter('all')">
                All
              </button>
              <button class="btn btn-sm ${this.filterType === 'payment' ? 'btn-primary' : 'btn-ghost'}" 
                      data-filter="payment" onclick="TransactionsPage.setFilter('payment')">
                Payments
              </button>
              <button class="btn btn-sm ${this.filterType === 'messaging' ? 'btn-primary' : 'btn-ghost'}" 
                      data-filter="messaging" onclick="TransactionsPage.setFilter('messaging')">
                Messages
              </button>
              <button class="btn btn-sm ${this.filterType === 'asset' ? 'btn-primary' : 'btn-ghost'}" 
                      data-filter="asset" onclick="TransactionsPage.setFilter('asset')">
                Assets
              </button>
              <button class="btn btn-sm ${this.filterType === 'other' ? 'btn-primary' : 'btn-ghost'}" 
                      data-filter="other" onclick="TransactionsPage.setFilter('other')">
                Other
              </button>
            </div>

            <!-- 搜索框 -->
            <div class="search-box" style="width: 280px;">
              <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="m21 21-4.35-4.35"/>
              </svg>
              <input 
                type="text" 
                class="form-input form-input-sm" 
                placeholder="Search transactions..."
                id="tx-search-input"
                onkeyup="if(event.key==='Enter') TransactionsPage.search(this.value)"
              >
            </div>

          </div>
        </div>
      </div>

      <!-- 交易列表表格 -->
      <div class="card">
        <div class="table-container">
          <table class="data-table">
            <thead>
              <tr>
                <th>Type</th>
                <th>Transaction ID</th>
                <th>Amount</th>
                <th>Fee</th>
                <th>Date/Time</th>
                <th>Status</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody id="transactions-table-body">
              <!-- 动态加载 -->
              <tr>
                <td colspan="7" style="text-align: center; padding: 50px;">
                  <div class="loading-spinner"></div>
                  <p class="text-muted mt-4">Loading transactions...</p>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- 分页控件 -->
        <div class="card-footer" id="pagination-controls">
          <!-- 动态生成 -->
        </div>
      </div>
    `;
  },

  /**
   * 加载交易数据
   */
  async loadData() {
    try {
      store.setState('ui.loading', true);

      const firstIndex = this.currentPage * this.pageSize;
      const lastIndex = firstIndex + this.pageSize - 1;

      // 获取交易数据
      const result = await api.getBlockchainTransactions(
        store.state.account.id || '',
        firstIndex,
        lastIndex
      );

      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Failed to load transactions');
      }

      const transactions = result.transactions || [];
      
      // 应用筛选
      let filteredTxs = transactions;
      if (this.filterType !== 'all') {
        filteredTxs = transactions.filter(tx => {
          switch (this.filterType) {
            case 'payment': return tx.type === 0;
            case 'messaging': return tx.type === 1;
            case 'asset': return [2, 3].includes(tx.type);
            case 'other': return ![0, 1, 2, 3].includes(tx.type);
            default: return true;
          }
        });
      }

      this.updateTable(filteredTxs);
      this.updatePagination(result.transactions?.length || 0);

    } catch (error) {
      console.error('Transactions load error:', error);
      
      const tbody = document.getElementById('transactions-table-body');
      if (tbody) {
        tbody.innerHTML = `
          <tr>
            <td colspan="7" style="text-align: center; padding: 40px;">
              <div class="empty-state-icon">⚠️</div>
              <p class="text-danger font-medium mt-4">${error.message}</p>
              <button class="btn btn-primary mt-4" onclick="TransactionsPage.loadData()">
                Retry
              </button>
            </td>
          </tr>
        `;
      }
    } finally {
      store.setState('ui.loading', false);
    }
  },

  /**
   * 更新交易表格
   */
  updateTable(transactions) {
    const tbody = document.getElementById('transactions-table-body');

    if (!tbody) return;

    if (!transactions || transactions.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="7" style="text-align: center; padding: 50px;">
            <div class="empty-state-icon" style="font-size: 48px;">📭</div>
            <p class="text-secondary mt-4 mb-2">No transactions found</p>
            <p class="text-muted text-sm">Your transaction history will appear here</p>
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = transactions.map((tx, index) => {
      const typeColor = Formatters.getTransactionColor(tx.type);
      const typeName = Formatters.formatTransactionType(tx.type);
      const isOutgoing = tx.senderRS === store.state.account.rsAddress;
      const amountValue = Math.abs(tx.amountNQT || 0);
      const feeValue = tx.feeNQT || 0;

      return `
        <tr class="transition-fast hover:bg-hover" style="animation: fadeIn 0.3s ease ${index * 0.05}s both;">
          
          <!-- 交易类型 -->
          <td>
            <span class="badge badge-${typeColor}">${typeName}</span>
          </td>
          
          <!-- 交易 ID -->
          <td>
            <code class="text-xs text-accent cursor-pointer hover-underline"
                  onclick="navigator.clipboard.writeText('${tx.transaction}')"
                  title="Click to copy">
              ${Formatters.truncateAddress(tx.transaction, 8, 6)}
            </code>
          </td>
          
          <!-- 金额 -->
          <td>
            <span class="${isOutgoing ? 'text-danger' : 'text-success'} font-semibold font-mono">
              ${isOutgoing ? '-' : '+'}${Formatters.formatNrc(amountValue, 4)}
            </span>
          </td>
          
          <!-- 手续费 -->
          <td class="text-muted text-sm font-mono">
            ${Formatters.formatNrc(feeValue, 4)}
          </td>
          
          <!-- 时间 -->
          <td class="text-sm text-secondary">
            <div>${Formatters.formatDate(tx.timestamp)}</div>
            <div class="text-xs text-muted">${Formatters.formatRelativeTime(tx.timestamp)}</div>
          </td>
          
          <!-- 状态 -->
          <td>
            ${tx.confirmations > 1440 
              ? '<span class="tag"><span class="tag-icon">✓</span>Matured</span>'
              : tx.confirmations > 0 
                ? `<span class="badge badge-success">${tx.confirmations} conf.</span>`
                : '<span class="badge badge-warning">Unconfirmed</span>'
            }
          </td>
          
          <!-- 操作按钮 -->
          <td>
            <button class="btn btn-sm btn-text text-primary" onclick="TransactionsPage.showDetails('${tx.transaction}')">
              Details →
            </button>
          </td>
        </tr>
      `;
    }).join('');
  },

  /**
   * 更新分页控件
   */
  updatePagination(totalItems) {
    const paginationEl = document.getElementById('pagination-controls');
    
    if (!paginationEl) return;

    const totalPages = Math.ceil(totalItems / this.pageSize);
    
    if (totalPages <= 1) {
      paginationEl.innerHTML = '';
      return;
    }

    let html = '<div class="flex items-center justify-between w-full">';
    
    // 左侧信息
    html += `<span class="text-sm text-muted">Showing page ${this.currentPage + 1} of ${totalPages}</span>`;
    
    // 右侧分页按钮
    html += '<div class="flex gap-2">';
    
    // 上一页
    html += `
      <button class="btn btn-sm btn-ghost" 
              onclick="TransactionsPage.goToPage(${this.currentPage - 1})"
              ${this.currentPage === 0 ? 'disabled' : ''}>
        ← Prev
      </button>
    `;
    
    // 页码
    for (let i = 0; i < Math.min(totalPages, 5); i++) {
      const isActive = i === this.currentPage;
      html += `
        <button class="btn btn-sm ${isActive ? 'btn-primary' : 'btn-ghost'}"
                onclick="TransactionsPage.goToPage(${i})">
          ${i + 1}
        </button>
      `;
    }
    
    // 下一页
    html += `
      <button class="btn btn-sm btn-ghost"
              onclick="TransactionsPage.goToPage(${this.currentPage + 1})"
              ${this.currentPage >= totalPages - 1 ? 'disabled' : ''}>
        Next →
      </button>
    `;
    
    html += '</div></div>';
    
    paginationEl.innerHTML = html;
  },

  /**
   * 设置筛选类型
   */
  setFilter(type) {
    this.filterType = type;
    this.currentPage = 0;
    
    // 更新按钮状态
    document.querySelectorAll('#filter-tabs button').forEach(btn => {
      const isActive = btn.dataset.filter === type;
      btn.className = `btn btn-sm ${isActive ? 'btn-primary' : 'btn-ghost'}`;
    });
    
    this.loadData();
  },

  /**
   * 跳转到指定页
   */
  goToPage(page) {
    if (page < 0) return;
    this.currentPage = page;
    this.loadData();
  },

  /**
   * 搜索交易
   */
  search(query) {
    if (!query.trim()) {
      Toast.show({ type: 'warning', title: 'Warning', message: 'Please enter a search term' });
      return;
    }
    
    // TODO: 实现服务端搜索
    Toast.show({ type: 'info', title: 'Search', message: `Searching for: ${query}` });
  },

  /**
   * 显示交易详情 (简化版)
   */
  showDetails(transactionId) {
    // TODO: 实现完整的交易详情模态框
    navigator.clipboard.writeText(transactionId).then(() => {
      Toast.show({
        type: 'success',
        title: 'Copied',
        message: `Transaction ID copied to clipboard`,
      });
    }).catch(() => {
      Toast.show({
        type: 'info',
        title: 'Transaction ID',
        message: transactionId,
        duration: 5000,
      });
    });
  },
};

// 导出供全局使用
window.TransactionsPage = TransactionsPage;
