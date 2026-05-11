/**
 * NRCS Wallet - Peers Page
 * 
 * 网络节点页面模块 (占位符实现)
 */

const PeersPage = {
  /**
   * 初始化节点页面
   */
  async init() {
    console.log('Peers page initialized');
    this.render();
    await this.loadData();
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
          <span class="page-title-icon">☁️</span>
          Network Peers
        </h1>
        <p class="page-subtitle">Monitor and manage connected network nodes</p>
      </div>

      <!-- 统计卡片 -->
      <div class="grid grid-cols-3 gap-6 mb-8">
        
        <!-- 已连接节点 -->
        <div class="stat-card stagger-item delay-1">
          <div class="stat-card-header">
            <span class="stat-card-label">Connected</span>
            <div class="stat-card-icon success">🟢</div>
          </div>
          <div class="stat-card-value" id="connected-peers-count">0</div>
          <div class="text-sm text-muted">Active connections</div>
        </div>

        <!-- 总节点数 -->
        <div class="stat-card stagger-item delay-2">
          <div class="stat-card-header">
            <span class="stat-card-label">Total Known</span>
            <div class="stat-card-icon primary">🌐</div>
          </div>
          <div class="stat-card-value" id="total-peers-count">0</div>
          <div class="text-sm text-muted">In peer database</div>
        </div>

        <!-- 平均延迟 -->
        <div class="stat-card stagger-item delay-3">
          <div class="stat-card-header">
            <span class="stat-card-label">Avg Latency</span>
            <div class="stat-card-icon warning">⚡</div>
          </div>
          <div class="stat-card-value" id="avg-latency">-</div>
          <div class="text-sm text-muted">Response time</div>
        </div>
      </div>

      <!-- 节点列表 -->
      <div class="card stagger-item delay-4">
        <div class="card-header">
          <h3 class="card-title">
            <span class="card-title-icon">📡</span>
            Peer List
          </h3>
          
          <button class="btn btn-primary btn-sm" onclick="PeersPage.refresh()">
            🔄 Refresh
          </button>
        </div>
        
        <div class="table-container">
          <table class="data-table">
            <thead>
              <tr>
                <th>Status</th>
                <th>Address</th>
                <th>Version</th>
                <th>Latency</th>
                <th>Last Connect</th>
              </tr>
            </thead>
            <tbody id="peers-table-body">
              <tr>
                <td colspan="5" style="text-align: center; padding: 50px;">
                  <div class="loading-spinner"></div>
                  <p class="text-muted mt-4">Loading peers...</p>
                </td>
              </tr>
            </tbody>
          </table>
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
   * 加载节点数据
   */
  async loadData() {
    try {
      store.setState('ui.loading', true);

      const result = await api.getPeers(true);
      
      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Failed to load peers');
      }

      const peers = result.peers || [];
      
      this.updateStats(peers);
      this.updateTable(peers);

    } catch (error) {
      console.error('Peers load error:', error);
      
      const tbody = document.getElementById('peers-table-body');
      if (tbody) {
        tbody.innerHTML = `
          <tr>
            <td colspan="5" style="text-align: center; padding: 40px;">
              <div class="empty-state-icon">⚠️</div>
              <p class="text-danger font-medium mt-4">${error.message}</p>
              <button class="btn btn-primary mt-4" onclick="PeersPage.loadData()">Retry</button>
            </td>
          </tr>
        `;
      }
    } finally {
      store.setState('ui.loading', false);
    }
  },

  /**
   * 更新统计信息
   */
  updateStats(peers) {
    const connectedEl = document.getElementById('connected-peers-count');
    const totalEl = document.getElementById('total-peers-count');
    const latencyEl = document.getElementById('avg-latency');

    if (connectedEl) {
      connectedEl.textContent = peers.length.toString();
    }

    if (totalEl) {
      totalEl.textContent = peers.length.toString();
    }

    if (latencyEl && peers.length > 0) {
      const avgLatency = peers.reduce((sum, p) => sum + (p.downloadVolume || 0), 0) / peers.length;
      latencyEl.textContent = Formatters.formatLatency(avgLatency);
    } else if (latencyEl) {
      latencyEl.textContent = '-';
    }
  },

  /**
   * 更新节点表格
   */
  updateTable(peers) {
    const tbody = document.getElementById('peers-table-body');

    if (!tbody) return;

    if (!peers || peers.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="5" style="text-align: center; padding: 50px;">
            <div class="empty-state-icon" style="font-size: 48px;">☁️</div>
            <p class="text-secondary mt-4 mb-2">No peers found</p>
            <p class="text-muted text-sm">Connected peers will appear here</p>
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = peers.map((peer, index) => `
      <tr style="animation: fadeIn 0.3s ease ${index * 0.03}s both;">
        <td>
          <span class="status-dot connected"></span>
        </td>
        <td>
          <code class="text-xs">${peer.address || peer.announcedAddress || '-'}</code>
        </td>
        <td>
          <span class="badge badge-primary text-xs">${peer.version || 'Unknown'}</span>
        </td>
        <td class="text-sm font-mono text-muted">
          ${Formatters.formatLatency(peer.downloadVolume || 0)}
        </td>
        <td class="text-sm text-secondary">
          ${peer.lastConnect ? Formatters.formatRelativeTime(peer.lastConnect) : '-'}
        </td>
      </tr>
    `).join('');
  },

  /**
   * 刷新数据
   */
  async refresh() {
    await this.loadData();
  },
};

// 导出供全局使用
window.PeersPage = PeersPage;
