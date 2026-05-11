/**
 * NRCS Wallet - Blocks Page
 * 
 * 区块浏览器页面模块
 * 展示: 区块列表、区块详情、生成者信息等
 */

const BlocksPage = {
  currentPage: 0,
  pageSize: 20,
  
  /**
   * 初始化区块页面
   */
  async init(params = {}) {
    console.log('Blocks page initialized', params);
    
    if (params.page) this.currentPage = parseInt(params.page);
    
    this.render();
    await this.loadData();
    
    // 设置自动刷新 (每30秒)
    this.refreshInterval = setInterval(() => this.loadData(), 30000);
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
   * 刷新数据
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
          <span class="page-title-icon">🔷</span>
          Block Explorer
        </h1>
        <p class="page-subtitle">Browse and explore the blockchain blocks</p>
      </div>

      <!-- 统计概览 -->
      <div class="grid grid-cols-3 gap-6 mb-8">
        
        <!-- 最新区块高度 -->
        <div class="stat-card stagger-item delay-1">
          <div class="stat-card-header">
            <span class="stat-card-label">Latest Height</span>
            <div class="stat-card-icon primary">📊</div>
          </div>
          <div class="stat-card-value" id="latest-height">-</div>
          <div class="text-sm text-muted" id="latest-block-time">Loading...</div>
        </div>

        <!-- 平均区块时间 -->
        <div class="stat-card stagger-item delay-2">
          <div class="stat-card-header">
            <span class="stat-card-label">Avg Block Time</span>
            <div class="stat-card-icon secondary">⏱️</div>
          </div>
          <div class="stat-card-value" id="avg-block-time">~60s</div>
          <div class="text-sm text-muted">Target interval</div>
        </div>

        <!-- 总交易数 -->
        <div class="stat-card stagger-item delay-3">
          <div class="stat-card-header">
            <span class="stat-card-label">Total Transactions</span>
            <div class="stat-card-icon success">💳</div>
          </div>
          <div class="stat-card-value" id="total-tx-count">-</div>
          <div class="text-sm text-muted" id="tx-per-block">- tx/block avg</div>
        </div>
      </div>

      <!-- 区块列表 -->
      <div class="card stagger-item delay-4">
        <div class="card-header">
          <h3 class="card-title">
            <span class="card-title-icon">⛓️</span>
            Block History
          </h3>
          
          <div class="flex gap-2">
            <!-- 跳转到指定高度 -->
            <div class="search-box" style="width: 200px;">
              <input type="number" 
                     class="form-input form-input-sm" 
                     placeholder="Go to height..."
                     id="goto-height-input"
                     onkeyup="if(event.key==='Enter') BlocksPage.goToHeight(this.value)">
            </div>
            
            <!-- 刷新按钮 -->
            <button class="btn btn-sm btn-primary" onclick="BlocksPage.refresh()">
              🔄 Refresh
            </button>
          </div>
        </div>
        
        <div class="table-container">
          <table class="data-table">
            <thead>
              <tr>
                <th>Height</th>
                <th>Block ID</th>
                <th>Generator</th>
                <th># Txns</th>
                <th>Total Amount</th>
                <th>Total Fee</td>
                <th>Timestamp</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody id="blocks-table-body">
              <!-- 动态加载 -->
              <tr>
                <td colspan="8" style="text-align: center; padding: 50px;">
                  <div class="loading-spinner"></div>
                  <p class="text-muted mt-4">Loading blocks...</p>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- 分页控件 -->
        <div class="card-footer" id="blocks-pagination">
          <!-- 动态生成 -->
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
   * 加载区块数据
   */
  async loadData() {
    try {
      store.setState('ui.loading', true);

      const firstIndex = this.currentPage * this.pageSize;
      const lastIndex = firstIndex + this.pageSize - 1;

      // 并行请求最新区块和区块列表
      const [latestBlockResult, blocksResult] = await api.batchRequests([
        { requestType: 'getBlock', params: { height: -1 } },
        { requestType: 'getBlocks', params: { firstIndex, lastIndex } },
      ]);

      // 处理最新区块信息
      if (latestBlockResult.status === 'fulfilled' && latestBlockResult.value) {
        const latestBlock = latestBlockResult.value;
        this.updateLatestBlockInfo(latestBlock);
      }

      // 处理区块列表
      if (blocksResult.status === 'fulfilled' && blocksResult.value) {
        const blocks = blocksResult.value.blocks || [];
        this.updateBlocksTable(blocks);
        this.updatePagination(blocks.length);
      }

    } catch (error) {
      console.error('Blocks load error:', error);
      
      const tbody = document.getElementById('blocks-table-body');
      if (tbody) {
        tbody.innerHTML = `
          <tr>
            <td colspan="8" style="text-align: center; padding: 40px;">
              <div class="empty-state-icon">⚠️</div>
              <p class="text-danger font-medium mt-4">${error.message}</p>
              <button class="btn btn-primary mt-4" onclick="BlocksPage.loadData()">Retry</button>
            </td>
          </tr>
        `;
      }
    } finally {
      store.setState('ui.loading', false);
    }
  },

  /**
   * 更新最新区块统计信息
   */
  updateLatestBlockInfo(block) {
    const heightEl = document.getElementById('latest-height');
    const timeEl = document.getElementById('latest-block-time');

    if (heightEl && block?.height !== undefined) {
      heightEl.textContent = Formatters.formatNumber(block.height);
    }

    if (timeEl && block?.timestamp) {
      timeEl.textContent = Formatters.formatRelativeTime(block.timestamp);
    }

    Sidebar.updateHeight(block.height);
  },

  /**
   * 更新区块表格
   */
  updateBlocksTable(blocks) {
    const tbody = document.getElementById('blocks-table-body');

    if (!tbody) return;

    if (!blocks || blocks.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="8" style="text-align: center; padding: 50px;">
            <div class="empty-state-icon" style="font-size: 48px;">🔍</div>
            <p class="text-secondary mt-4 mb-2">No blocks found</p>
            <p class="text-muted text-sm">Blocks will appear here as they are mined</p>
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = blocks.map((block, index) => {
      const totalAmountNQT = block.totalAmountNQT || block.totalAmount || 0;
      const totalFeeNQT = block.totalFeeNQT || block.totalFee || 0;
      const txCount = block.numberOfTransactions || 0;

      return `
        <tr class="transition-fast hover:bg-hover cursor-pointer"
            onclick="BlocksPage.showBlockDetails('${block.block}')"
            style="animation: fadeIn 0.3s ease ${index * 0.03}s both;">
          
          <!-- 区块高度 -->
          <td>
            <span class="font-semibold text-accent font-mono">#${Formatters.formatNumber(block.height)}</span>
          </td>
          
          <!-- 区块 ID -->
          <td>
            <code class="text-xs text-secondary cursor-pointer hover-underline"
                  onclick="event.stopPropagation(); navigator.clipboard.writeText('${block.block}')"
                  title="Click to copy ID">
              ${Formatters.truncateAddress(block.block, 10, 6)}
            </code>
          </td>
          
          <!-- 生成者 -->
          <td>
            <span class="text-sm" title="${block.generatorRS || block.generatorId}">
              ${block.generatorRS 
                ? Formatters.truncateAddress(block.generatorRS, 12, 6)
                : Formatters.truncateAddress(block.generatorId?.toString() || '-', 12, 6)}
            </span>
          </td>
          
          <!-- 交易数量 -->
          <td>
            ${txCount > 0 
              ? `<span class="badge badge-primary">${txCount}</span>`
              : '<span class="text-muted">0</span>'
            }
          </td>
          
          <!-- 总金额 -->
          <td class="font-mono text-sm text-success">
            +${Formatters.formatNrc(totalAmountNQT, 2)}
          </td>
          
          <!-- 总手续费 -->
          <td class="font-mono text-sm text-muted">
            ${Formatters.formatNrc(totalFeeNQT, 2)}
          </td>
          
          <!-- 时间戳 -->
          <td class="text-sm text-secondary">
            <div>${Formatters.formatDate(block.timestamp)}</div>
            <div class="text-xs text-muted">${Formatters.formatRelativeTime(block.timestamp)}</div>
          </td>
          
          <!-- 操作按钮 -->
          <td>
            <button class="btn btn-sm btn-text text-primary">
              View →
            </button>
          </td>
        </tr>
      `;
    }).join('');

    // 更新统计数据
    this.updateStats(blocks);
  },

  /**
   * 更新统计面板
   */
  updateStats(blocks) {
    if (!blocks || blocks.length === 0) return;

    let totalTxCount = 0;

    blocks.forEach(block => {
      totalTxCount += block.numberOfTransactions || 0;
    });

    const totalTxEl = document.getElementById('total-tx-count');
    const txPerBlockEl = document.getElementById('tx-per-block');

    if (totalTxEl) {
      totalTxEl.textContent = Formatters.formatNumber(totalTxCount);
    }

    if (txPerBlockEl) {
      const avg = (totalTxCount / blocks.length).toFixed(1);
      txPerBlockEl.textContent = `${avg} tx/block avg`;
    }
  },

  /**
   * 更新分页控件
   */
  updatePagination(totalItems) {
    const paginationEl = document.getElementById('blocks-pagination');
    
    if (!paginationEl) return;

    const totalPages = Math.ceil(totalItems / this.pageSize);
    
    if (totalPages <= 1) {
      paginationEl.innerHTML = '';
      return;
    }

    let html = '<div class="flex items-center justify-between w-full">';
    
    html += `<span class="text-sm text-muted">Page ${this.currentPage + 1} of ${totalPages}</span>`;
    
    html += '<div class="flex gap-2">';
    
    html += `
      <button class="btn btn-sm btn-ghost" 
              onclick="BlocksPage.goToPage(${this.currentPage - 1})"
              ${this.currentPage === 0 ? 'disabled' : ''}>
        ← Prev
      </button>
    `;
    
    for (let i = 0; i < Math.min(totalPages, 5); i++) {
      const isActive = i === this.currentPage;
      html += `
        <button class="btn btn-sm ${isActive ? 'btn-primary' : 'btn-ghost'}"
                onclick="BlocksPage.goToPage(${i})">
          ${i + 1}
        </button>
      `;
    }
    
    html += `
      <button class="btn btn-sm btn-ghost"
              onclick="BlocksPage.goToPage(${this.currentPage + 1})"
              ${this.currentPage >= totalPages - 1 ? 'disabled' : ''}>
        Next →
      </button>
    `;
    
    html += '</div></div>';
    
    paginationEl.innerHTML = html;
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
   * 跳转到指定高度的区块
   */
  goToHeight(height) {
    if (!height || parseInt(height) < 0) {
      Toast.show({ type: 'warning', title: 'Invalid Input', message: 'Please enter a valid block height' });
      return;
    }
    
    // TODO: 实现跳转功能或显示详情
    Toast.show({
      type: 'info',
      title: 'Navigate',
      message: `Navigating to block #${height}`,
    });
  },

  /**
   * 显示区块详情 (简化版)
   */
  showBlockDetails(blockId) {
    navigator.clipboard.writeText(blockId).then(() => {
      Toast.show({
        type: 'success',
        title: 'Copied',
        message: `Block ID copied to clipboard`,
      });
    }).catch(() => {
      Toast.show({
        type: 'info',
        title: 'Block ID',
        message: blockId,
        duration: 5000,
      });
    });
  },
};

// 导出供全局使用
window.BlocksPage = BlocksPage;
