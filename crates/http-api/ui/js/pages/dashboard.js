/**
 * NRCS Wallet - Dashboard Page
 * 
 * 仪表盘页面模块
 * 展示: 账户余额、最新区块、交易统计、区块链状态等核心信息
 */

const DashboardPage = {
  /**
   * 初始化仪表盘页面
   */
  async init() {
    console.log('Dashboard page initialized');
    this.render();
    await this.loadData();
    
    // 设置自动刷新 (每30秒)
    this.refreshInterval = setInterval(() => this.loadData(), 30000);
  },

  /**
   * 销毁页面 (清理定时器等)
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
          <span class="page-title-icon">📊</span>
          Dashboard
        </h1>
        <p class="page-subtitle">Overview of your blockchain account and network status</p>
      </div>

      <!-- 统计卡片网格 -->
      <div class="grid grid-cols-4 gap-6 mb-8" id="stats-grid">
        
        <!-- 账户余额卡片 -->
        <div class="stat-card stagger-item delay-1">
          <div class="stat-card-header">
            <span class="stat-card-label">Account Balance</span>
            <div class="stat-card-icon primary">💼</div>
          </div>
          <div class="stat-card-value" id="balance-value">0.00</div>
          <div style="display: flex; align-items: center; gap: var(--space-2);">
            <span class="text-sm text-muted">NRC</span>
            <span class="stat-card-change positive" id="balance-change" style="display: none;">
              +0.00%
            </span>
          </div>
        </div>

        <!-- 最新区块卡片 -->
        <div class="stat-card stagger-item delay-2">
          <div class="stat-card-header">
            <span class="stat-card-label">Latest Block</span>
            <div class="stat-card-icon secondary">🔷</div>
          </div>
          <div class="stat-card-value" id="block-height">-</div>
          <div class="text-sm text-muted" id="block-time">Loading...</div>
        </div>

        <!-- 连接状态卡片 -->
        <div class="stat-card stagger-item delay-3">
          <div class="stat-card-header">
            <span class="stat-card-label">Connected Peers</span>
            <div class="stat-card-icon success">☁️</div>
          </div>
          <div class="stat-card-value" id="peers-count">0</div>
          <div class="text-sm text-muted" id="connection-status-text">Connecting...</div>
        </div>

        <!-- 待确认交易卡片 -->
        <div class="stat-card stagger-item delay-4">
          <div class="stat-card-header">
            <span class="stat-card-label">Unconfirmed</span>
            <div class="stat-card-icon warning">⏳</div>
          </div>
          <div class="stat-card-value" id="unconfirmed-count">0</div>
          <div class="text-sm text-muted">Transactions pending</div>
        </div>
      </div>

      <!-- 主要内容区域 -->
      <div class="grid grid-cols-3 gap-6">
        
        <!-- 最近交易列表 -->
        <div class="card col-span-2 stagger-item delay-5">
          <div class="card-header">
            <h3 class="card-title">
              <span class="card-title-icon">💳</span>
              Recent Transactions
            </h3>
            <a href="#/transactions" class="btn btn-sm btn-ghost">
              View All →
            </a>
          </div>
          <div class="card-body card-no-padding">
            <div class="table-container">
              <table class="data-table" id="recent-transactions-table">
                <thead>
                  <tr>
                    <th>Type</th>
                    <th>Amount</th>
                    <th>Date/Time</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody id="recent-transactions-body">
                  <!-- 动态加载 -->
                  <tr>
                    <td colspan="4" style="text-align: center; padding: 40px;">
                      <div class="loading-spinner"></div>
                      <p class="text-muted mt-4">Loading transactions...</p>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- 区块链状态面板 -->
        <div class="card stagger-item delay-6">
          <div class="card-header">
            <h3 class="card-title">
              <span class="card-title-icon">⛓️</span>
              Blockchain Status
            </h3>
          </div>
          <div class="card-body">
            
            <!-- 同步进度 -->
            <div class="mb-6">
              <div class="flex justify-between mb-2">
                <span class="text-sm font-medium">Sync Progress</span>
                <span class="text-sm text-accent" id="sync-percentage">100%</span>
              </div>
              <div class="progress progress-lg">
                <div class="progress-bar success" id="sync-progress" style="width: 100%;"></div>
              </div>
            </div>

            <!-- 状态信息列表 -->
            <div class="space-y-4" id="blockchain-info-list">
              
              <div class="flex items-center justify-between p-3 rounded-md bg-hover">
                <div>
                  <div class="text-xs text-muted mb-1">Cumulative Difficulty</div>
                  <div class="font-mono text-sm" id="cumulative-difficulty">-</div>
                </div>
              </div>

              <div class="flex items-center justify-between p-3 rounded-md bg-hover">
                <div>
                  <div class="text-xs text-muted mb-1">Total Blocks</div>
                  <div class="font-semibold" id="total-blocks">-</div>
                </div>
              </div>

              <div class="flex items-center justify-between p-3 rounded-md bg-hover">
                <div>
                  <div class="text-xs text-muted mb-1">Version</div>
                  <div class="font-mono text-sm" id="nrcs-version">-</div>
                </div>
              </div>

              <div class="flex items-center justify-between p-3 rounded-md bg-hover">
                <div>
                  <div class="text-xs text-muted mb-1">Last Block Generator</div>
                  <div class="font-mono text-xs truncate" id="last-generator" style="max-width: 180px;">
                    -
                  </div>
                </div>
              </div>

              <div class="flex items-center justify-between p-3 rounded-md bg-hover">
                <div>
                  <div class="text-xs text-muted mb-1">Time</div>
                  <div class="text-sm" id="network-time">-</div>
                </div>
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
   * 加载仪表盘数据
   */
  async loadData() {
    try {
      store.setState('ui.loading', true);
      
      // 并行请求所有数据
      const [
        blockchainStatusResult,
        latestBlockResult,
        peersResult,
        transactionsResult,
      ] = await api.batchRequests([
        { requestType: 'getBlockchainStatus' },
        { requestType: 'getBlock', params: { height: -1 } },
        { requestType: 'getPeers', params: { activeOnly: true, state: 'CONNECTED' } },
        { requestType: 'getBlockchainTransactions', params: { 
          firstIndex: 0, 
          lastIndex: 9,
        }},
      ]);

      // 处理区块链状态
      if (blockchainStatusResult.status === 'fulfilled') {
        const status = blockchainStatusResult.value;
        this.updateBlockchainStatus(status);
        store.setState('blockchain', {
          height: status.numberOfBlocks || 0,
          cumulativeDifficulty: status.cumulativeDifficulty,
          numberOfBlocks: status.numberOfBlocks || 0,
          time: status.time || 0,
          version: status.version || '',
          isScanning: status.isScanning || false,
          isDownloading: status.isDownloading || false,
        });
      }

      // 处理最新区块
      if (latestBlockResult.status === 'fulfilled') {
        const block = latestBlockResult.value;
        this.updateLatestBlock(block);
        store.setState('latestBlock', block);
      }

      // 处理节点信息
      if (peersResult.status === 'fulfilled') {
        const peersData = peersResult.value;
        this.updatePeersInfo(peersData);
        store.setState('peers', {
          connected: peersData.peers?.length || 0,
          total: peersData.peers?.length || 0,
          list: peersData.peers || [],
        });
        Sidebar.updateConnectionStatus('connected');
      }

      // 处理交易数据
      if (transactionsResult.status === 'fulfilled') {
        const txData = transactionsResult.value;
        this.updateTransactionsTable(txData.transactions || []);
        store.setState('transactions', txData.transactions || []);
      }

      // 更新侧边栏信息
      Sidebar.updateHeight(store.state.blockchain.height);

    } catch (error) {
      console.error('Dashboard load error:', error);
      Toast.show({
        type: 'error',
        title: 'Load Failed',
        message: 'Failed to load dashboard data. Please refresh the page.',
      });
      Sidebar.updateConnectionStatus('disconnected');
    } finally {
      store.setState('ui.loading', false);
    }
  },

  /**
   * 更新区块链状态面板
   */
  updateBlockchainStatus(status) {
    // 更新累计难度
    const diffEl = document.getElementById('cumulative-difficulty');
    if (diffEl && status.cumulativeDifficulty) {
      diffEl.textContent = Formatters.formatDifficulty(status.cumulativeDifficulty);
    }

    // 更新总区块数
    const totalBlocksEl = document.getElementById('total-blocks');
    if (totalBlocksEl) {
      totalBlocksEl.textContent = Formatters.formatNumber(status.numberOfBlocks || 0);
    }

    // 更新版本号
    const versionEl = document.getElementById('nrcs-version');
    if (versionEl && status.version) {
      versionEl.textContent = `v${status.version}`;
    }

    // 更新网络时间
    const timeEl = document.getElementById('network-time');
    if (timeEl && status.time) {
      timeEl.textContent = Formatters.formatDate(status.time);
    }

    // 更新同步进度
    const isScanning = status.isScanning || status.isDownloading;
    const syncProgressEl = document.getElementById('sync-progress');
    const syncPercentageEl = document.getElementById('sync-percentage');
    
    if (syncProgressEl && syncPercentageEl) {
      if (isScanning) {
        syncProgressEl.style.width = '75%';
        syncProgressEl.classList.remove('success');
        syncProgressEl.classList.add('warning');
        syncPercentageEl.textContent = 'Syncing...';
      } else {
        syncProgressEl.style.width = '100%';
        syncProgressEl.classList.remove('warning');
        syncProgressEl.classList.add('success');
        syncPercentageEl.textContent = '100%';
      }
    }
  },

  /**
   * 更新最新区块显示
   */
  updateLatestBlock(block) {
    const heightEl = document.getElementById('block-height');
    const timeEl = document.getElementById('block-time');
    const generatorEl = document.getElementById('last-generator');

    if (heightEl && block?.height !== undefined) {
      heightEl.textContent = Formatters.formatNumber(block.height);
    }

    if (timeEl && block?.timestamp) {
      timeEl.textContent = Formatters.formatRelativeTime(block.timestamp);
    }

    if (generatorEl && block?.generatorRS) {
      generatorEl.textContent = Formatters.truncateAddress(generatorBlock.generatorRS, 10, 6);
    }
  },

  /**
   * 更新节点连接信息
   */
  updatePeersInfo(peersData) {
    const countEl = document.getElementById('peers-count');
    const statusTextEl = document.getElementById('connection-status-text');

    const peerCount = peersData.peers?.length || 0;

    if (countEl) {
      countEl.textContent = peerCount.toString();
    }

    if (statusTextEl) {
      if (peerCount > 20) {
        statusTextEl.textContent = `${peerCount} peers online`;
        statusTextEl.className = 'text-sm text-success';
      } else if (peerCount > 5) {
        statusTextEl.textContent = `${peerCount} peers connected`;
        statusTextEl.className = 'text-sm text-accent';
      } else if (peerCount > 0) {
        statusTextEl.textContent = `${peerCount} peers (low)`;
        statusTextEl.className = 'text-sm text-warning';
      } else {
        statusTextEl.textContent = 'No connections';
        statusTextEl.className = 'text-sm text-danger';
      }
    }
  },

  /**
   * 更新最近交易表格
   */
  updateTransactionsTable(transactions) {
    const tbody = document.getElementById('recent-transactions-body');

    if (!tbody) return;

    if (!transactions || transactions.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="4" style="text-align: center; padding: 40px;">
            <div class="empty-state-icon" style="font-size: 48px;">📭</div>
            <p class="text-secondary mt-4">No recent transactions</p>
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = transactions.map(tx => {
      const typeColor = Formatters.getTransactionColor(tx.type);
      const typeName = Formatters.formatTransactionType(tx.type);
      
      // 判断是收入还是支出 (简化判断)
      const isOutgoing = tx.senderRS === store.state.account.rsAddress;
      const amountValue = Math.abs(tx.amountNQT || 0);
      const amountFormatted = Formatters.formatNrc(amountValue, 4);
      
      return `
        <tr class="hover:bg-hover transition-fast cursor-pointer" onclick="router.navigate('/transactions/${tx.transaction}')">
          <td>
            <span class="badge badge-${typeColor}">
              ${typeName}
            </span>
          </td>
          <td>
            <span class="${isOutgoing ? 'text-danger' : 'text-success'} font-medium">
              ${isOutgoing ? '-' : '+'}${amountFormatted}
            </span>
          </td>
          <td class="text-muted text-sm">
            ${Formatters.formatRelativeTime(tx.timestamp)}
          </td>
          <td>
            ${tx.confirmations > 0 
              ? '<span class="badge badge-success">Confirmed</span>'
              : '<span class="badge badge-warning">Pending</span>'
            }
          </td>
        </tr>
      `;
    }).join('');
  },
};

// 导出供全局使用
window.DashboardPage = DashboardPage;
