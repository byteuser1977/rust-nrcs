/**
 * NRCS Wallet - Forging Control Panel
 * 
 * 铸造控制面板
 * 展示: 铸造状态、启动/停止控制、统计信息等
 */

const ForgingPage = {
  isForging: false,
  forgingStats: null,

  /**
   * 初始化页面
   */
  async init() {
    console.log('Forging control panel initialized');
    this.render();
    await this.loadForgingStatus();

    // 设置自动刷新 (每10秒)
    this.refreshInterval = setInterval(() => this.loadForgingStatus(), 10000);
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
          <span class="page-title-icon">⛏️</span>
          Forging Control
        </h1>
        <p class="page-subtitle">Manage block forging and monitor your node's performance</p>
      </div>

      <!-- 主要内容区域 -->
      <div class="grid grid-cols-2 gap-8">
        
        <!-- 左侧: 铸造状态和控制 -->
        <div class="space-y-6 stagger-item delay-1">
          
          <!-- 铸造状态卡片 -->
          <div class="card" id="forging-status-card" style="border-left: 4px solid var(--text-muted);">
            <div class="card-body text-center py-8">
              
              <!-- 状态图标 -->
              <div class="forging-status-icon mb-6" id="forging-icon"
                   style="font-size: 80px; opacity: 0.3;">
                ⏸️
              </div>

              <!-- 状态文本 -->
              <h2 class="text-2xl font-bold mb-2" id="forging-status-text">
                Not Forging
              </h2>
              <p class="text-secondary mb-6" id="forging-description">
                Your account is not currently forging blocks
              </p>

              <!-- 控制按钮 -->
              <div class="flex justify-center gap-4" id="forging-controls">
                <button class="btn btn-success btn-lg" 
                        onclick="ForgingPage.startForging()"
                        id="start-forging-btn">
                  ▶️ Start Forging
                </button>

                <button class="btn btn-danger btn-lg" 
                        onclick="ForgingPage.stopForging()"
                        id="stop-forging-btn"
                        disabled style="opacity: 0.5;">
                  ⏹️ Stop Forging
                </button>
              </div>

            </div>
          </div>

          <!-- 账户余额要求 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">💰</span>
                Forging Requirements
              </h3>
            </div>
            <div class="card-body space-y-4">
              
              <div class="flex items-center justify-between p-4 rounded-lg bg-hover transition-fast">
                <div>
                  <div class="font-medium">Minimum Balance</div>
                  <div class="text-sm text-muted mt-1">Required to start forging</div>
                </div>
                <code class="font-mono text-accent font-semibold">0 NRC</code>
              </div>

              <div class="flex items-center justify-between p-4 rounded-lg bg-hover transition-fast">
                <div>
                  <div class="font-medium">Your Balance</div>
                  <div class="text-sm text-muted mt-1">Current available balance</div>
                </div>
                <code class="font-mono text-success font-semibold" id="your-balance-display">
                  Loading...
                </code>
              </div>

              <div class="alert alert-info mt-4">
                <div class="alert-icon">ℹ️</div>
                <div class="alert-content">
                  <p class="alert-message text-sm">
                    Forging requires your account to have a sufficient balance and be unlocked.
                    The probability of forging a block is proportional to your effective balance.
                  </p>
                </div>
              </div>

            </div>
          </div>

        </div>

        <!-- 右侧: 统计信息和历史 -->
        <div class="space-y-6 stagger-item delay-2">
          
          <!-- 铸造统计 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">📊</span>
                Forging Statistics
              </h3>
            </div>
            <div class="card-body">
              <div class="grid grid-cols-2 gap-4" id="forging-stats-grid">
                
                <div class="p-4 rounded-lg bg-hover text-center">
                  <div class="text-xs text-muted uppercase tracking-wider mb-2">
                    Blocks Forged
                  </div>
                  <div class="text-3xl font-bold text-primary" id="blocks-forged-count">
                    -
                  </div>
                </div>

                <div class="p-4 rounded-lg bg-hover text-center">
                  <div class="text-xs text-muted uppercase tracking-wider mb-2">
                    Total Earned
                  </div>
                  <div class="text-3xl font-bold text-success" id="total-earned-value">
                    -
                  </div>
                </div>

                <div class="p-4 rounded-lg bg-hover text-center">
                  <div class="text-xs text-muted uppercase tracking-wider mb-2">
                    Uptime
                  </div>
                  <div class="text-xl font-bold" id="forging-uptime">
                    -
                  </div>
                </div>

                <div class="p-4 rounded-lg bg-hover text-center">
                  <div class="text-xs text-muted uppercase tracking-wider mb-2">
                    Hit Rate
                  </div>
                  <div class="text-xl font-bold text-secondary" id="hit-rate">
                    -
                  </div>
                </div>

              </div>
            </div>
          </div>

          <!-- 最近铸造的区块 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">🔷</span>
                Recently Forged Blocks
              </h3>
              <button class="btn btn-sm btn-ghost" onclick="router.navigate('/blocks')">
                View All →
              </button>
            </div>
            <div class="card-body card-no-padding">
              <div class="table-container">
                <table class="data-table">
                  <thead>
                    <tr>
                      <th>Height</th>
                      <th>ID</th>
                      <th>Time</th>
                      <th>Reward</th>
                    </tr>
                  </thead>
                  <tbody id="forged-blocks-table-body">
                    <tr>
                      <td colspan="4" style="text-align: center; padding: 30px;">
                        <span class="text-sm text-muted">No blocks forged yet</span>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>

          <!-- 日志输出 -->
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <span class="card-title-icon">📝</span>
                Activity Log
              </h3>
              <button class="btn btn-sm btn-text text-primary" onclick="ForgingPage.clearLog()">
                Clear
              </button>
            </div>
            <div class="card-body">
              <div class="bg-elevated rounded-lg p-4 font-mono text-xs overflow-auto max-h-48" 
                   id="forging-log-container"
                   style="line-height: 1.6;">
                <div class="text-muted" id="forging-log-content">
                  Waiting for activity...
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

    // 加载账户余额
    this.loadAccountBalance();
  },

  /**
   * 加载铸造状态
   */
  async loadForgingStatus() {
    try {
      const result = await api.startStopForging('', false); // 查询状态

      if (!result.errorCode && result !== undefined) {
        this.updateForgingStatus(result);
      }

      // 加载统计信息
      this.loadForgingStats();

    } catch (error) {
      console.error('Load forging status error:', error);
      this.addLogEntry(`Error loading status: ${error.message}`, 'error');
    }
  },

  /**
   * 加载账户余额
   */
  async loadAccountBalance() {
    try {
      if (store.state.account.id) {
        const result = await api.getAccountBalance(store.state.account.id);
        
        if (!result.errorCode) {
          const balanceEl = document.getElementById('your-balance-display');
          if (balanceEl) {
            balanceEl.textContent = Formatters.formatNrc(result.balanceNQT || 0, 2) + ' NRC';
          }
        }
      }
    } catch (error) {
      console.error('Load balance error:', error);
    }
  },

  /**
   * 更新铸造状态显示
   */
  updateForgingStatus(statusData) {
    const statusCard = document.getElementById('forging-status-card');
    const iconEl = document.getElementById('forging-icon');
    const statusTextEl = document.getElementById('forging-status-text');
    const descEl = document.getElementById('forging-description');
    const startBtn = document.getElementById('start-forging-btn');
    const stopBtn = document.getElementById('stop-forging-btn');

    // 判断是否正在锻造 (这里需要根据实际 API 返回值判断)
    const isCurrentlyForging = statusData?.status === 'forging' || this.isForging;
    
    if (isCurrentlyForging) {
      this.isForging = true;
      
      if (statusCard) statusCard.style.borderLeftColor = 'var(--accent-success)';
      if (iconEl) {
        iconEl.textContent = '⚡';
        iconEl.style.opacity = '1';
        iconEl.style.animation = 'float 2s ease-in-out infinite';
      }
      if (statusTextEl) statusTextEl.textContent = 'Currently Forging';
      if (descEl) descEl.textContent = 'Your account is actively forging blocks';
      
      if (startBtn) {
        startBtn.disabled = true;
        startBtn.style.opacity = '0.5';
      }
      if (stopBtn) {
        stopBtn.disabled = false;
        stopBtn.style.opacity = '1';
      }
      
      this.addLogEntry('Forging is active', 'success');
    } else {
      this.isForging = false;
      
      if (statusCard) statusCard.style.borderLeftColor = 'var(--text-muted)';
      if (iconEl) {
        iconEl.textContent = '⏸️';
        iconEl.style.opacity = '0.3';
        iconEl.style.animation = '';
      }
      if (statusTextEl) statusTextEl.textContent = 'Not Forging';
      if (descEl) descEl.textContent = 'Your account is not currently forging blocks';
      
      if (startBtn) {
        startBtn.disabled = false;
        startBtn.style.opacity = '1';
      }
      if (stopBtn) {
        stopBtn.disabled = true;
        stopBtn.style.opacity = '0.5';
      }
    }
  },

  /**
   * 加载铸造统计数据
   */
  async loadForgingStats() {
    // TODO: 实现真正的统计 API 调用
    // 这里使用模拟数据作为占位
    
    const stats = {
      blocksForged: Math.floor(Math.random() * 100),
      totalEarned: Math.random() * 50,
      uptime: '0h 0m',
      hitRate: '-',
    };

    // 更新显示
    const blocksEl = document.getElementById('blocks-forged-count');
    const earnedEl = document.getElementById('total-earned-value');
    const uptimeEl = document.getElementById('forging-uptime');
    const rateEl = document.getElementById('hit-rate');

    if (blocksEl) blocksEl.textContent = stats.blocksForged.toString();
    if (earnedEl) earnedEl.textContent = `${stats.totalEarned.toFixed(2)} NRC`;
    if (uptimeEl) uptimeEl.textContent = stats.uptime;
    if (rateEl) rateEl.textContent = stats.hitRate;
  },

  /**
   * 开始锻造
   */
  async startForging() {
    if (!store.state.secretPhrase) {
      Toast.show({
        type: 'warning',
        title: 'Authentication Required',
        message: 'Please enter your passphrase to start forging',
      });
      return;
    }

    const confirmed = confirm('Are you sure you want to start forging?\n\nThis will use your account\'s balance to attempt to forge new blocks.');

    if (!confirmed) return;

    try {
      this.addLogEntry('Attempting to start forging...', 'info');

      const result = await api.startStopForging(store.state.secretPhrase, true);

      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Failed to start forging');
      }

      this.isForging = true;
      this.updateForgingStatus({ status: 'forging' });

      Toast.show({
        type: 'success',
        title: 'Forging Started',
        message: 'Your account is now attempting to forge blocks',
      });

      this.addLogEntry('Forging started successfully', 'success');

    } catch (error) {
      console.error('Start forging error:', error);
      Toast.show({
        type: 'error',
        title: 'Start Failed',
        message: error.message,
      });
      this.addLogEntry(`Failed to start: ${error.message}`, 'error');
    }
  },

  /**
   * 停止锻造
   */
  async stopForging() {
    const confirmed = confirm('Are you sure you want to stop forging?\n\nYou will no longer forge new blocks.');

    if (!confirmed) return;

    try {
      this.addLogEntry('Stopping forging...', 'info');

      const result = await api.startStopForging(store.state.secretPhrase, false);

      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Failed to stop forging');
      }

      this.isForging = false;
      this.updateForgingStatus({ status: 'not_forging' });

      Toast.show({
        type: 'info',
        title: 'Forging Stopped',
        message: 'Your account has stopped forging',
      });

      this.addLogEntry('Forging stopped successfully', 'warning');

    } catch (error) {
      console.error('Stop forging error:', error);
      Toast.show({
        type: 'error',
        title: 'Stop Failed',
        message: error.message,
      });
      this.addLogEntry(`Failed to stop: ${error.message}`, 'error');
    }
  },

  /**
   * 添加日志条目
   */
  addLogEntry(message, type = 'info') {
    const logContainer = document.getElementById('forging-log-content');
    
    if (!logContainer) return;

    const timestamp = new Date().toLocaleTimeString();
    const colors = {
      info: 'text-secondary',
      success: 'text-success',
      warning: 'text-warning',
      error: 'text-danger',
    };

    const entry = document.createElement('div');
    entry.className = colors[type] || 'text-secondary';
    entry.innerHTML = `<span class="text-muted">[${timestamp}]</span> ${message}`;

    logContainer.appendChild(entry);
    logContainer.scrollTop = logContainer.scrollHeight;

    // 保持最多100条日志
    while (logContainer.children.length > 100) {
      logContainer.removeChild(logContainer.firstChild);
    }
  },

  /**
   * 清空日志
   */
  clearLog() {
    const logContainer = document.getElementById('forging-log-content');
    
    if (logContainer) {
      logContainer.innerHTML = '<div class="text-muted">Log cleared</div>';
    }
  },
};

// 导出供全局使用
window.ForgingPage = ForgingPage;
