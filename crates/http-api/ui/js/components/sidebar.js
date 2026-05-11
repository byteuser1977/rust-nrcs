/**
 * NRCS Wallet - Sidebar Component
 * 
 * 侧边栏组件
 * 处理: 导航菜单、用户信息展示、搜索功能等
 */

const Sidebar = {
  /**
   * 初始化侧边栏
   */
  init() {
    this.sidebar = document.getElementById('sidebar');
    this.overlay = document.getElementById('sidebar-overlay');
    
    if (!this.sidebar) return;
    
    this.bindEvents();
    this.setupSearch();
    console.log('Sidebar initialized');
  },

  /**
   * 绑定事件监听器
   */
  bindEvents() {
    // 遮罩点击关闭
    if (this.overlay) {
      this.overlay.addEventListener('click', () => this.close());
    }
    
    // 导航菜单项点击
    const navItems = this.sidebar.querySelectorAll('.nav-item');
    navItems.forEach(item => {
      item.addEventListener('click', (e) => this.handleNavClick(e, item));
    });
  },

  /**
   * 处理导航项点击
   */
  handleNavClick(event, item) {
    event.preventDefault();
    
    // 更新活动状态
    this.setActiveItem(item);
    
    // 移动端自动关闭
    if (window.innerWidth <= 1024) {
      setTimeout(() => this.close(), 150);
    }
  },

  /**
   * 设置当前活动的导航项
   * @param {HTMLElement|string} itemOrPage - 元素或页面名称
   */
  setActiveItem(itemOrPage) {
    const navItems = this.sidebar.querySelectorAll('.nav-item');
    
    navItems.forEach(item => {
      item.classList.remove('active');
      
      if (typeof itemOrPage === 'string') {
        const page = item.getAttribute('data-page');
        if (page === itemOrPage) {
          item.classList.add('active');
        }
      } else if (item === itemOrPage) {
        item.classList.add('active');
      }
    });
  },

  /**
   * 打开侧边栏 (移动端)
   */
  open() {
    if (!this.sidebar) return;
    
    this.sidebar.classList.add('open');
    if (this.overlay) {
      this.overlay.classList.add('visible');
    }
    
    // 禁止背景滚动
    document.body.style.overflow = 'hidden';
    
    store.setState('ui.sidebarOpen', true);
  },

  /**
   * 关闭侧边栏 (移动端)
   */
  close() {
    if (!this.sidebar) return;
    
    this.sidebar.classList.remove('open');
    if (this.overlay) {
      this.overlay.classList.remove('visible');
    }
    
    // 恢复滚动
    document.body.style.overflow = '';
    
    store.setState('ui.sidebarOpen', false);
  },

  /**
   * 切换侧边栏状态
   */
  toggle() {
    if (this.sidebar?.classList.contains('open')) {
      this.close();
    } else {
      this.open();
    }
  },

  /**
   * 设置全局搜索功能
   */
  setupSearch() {
    const searchInput = document.getElementById('global-search');
    
    if (!searchInput) return;
    
    let debounceTimer;
    
    searchInput.addEventListener('input', (e) => {
      clearTimeout(debounceTimer);
      
      const query = e.target.value.trim();
      
      if (!query || query.length < 2) return;
      
      debounceTimer = setTimeout(() => {
        this.handleSearch(query);
      }, 300);
    });
    
    searchInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        this.handleSearch(e.target.value.trim());
      }
      
      // ESC 清空搜索
      if (e.key === 'Escape') {
        searchInput.value = '';
        searchInput.blur();
      }
    });
  },

  /**
   * 处理搜索请求
   * @param {string} query - 搜索关键词
   */
  async handleSearch(query) {
    try {
      store.setState('ui.loading', true);
      
      const results = await api.search(query);
      
      if (results.accounts && results.accounts.length > 0) {
        // 跳转到第一个匹配的账户
        router.navigate(`/account/${results.accounts[0].account}`);
      } else {
        Toast.show({
          type: 'warning',
          title: 'No results',
          message: `No accounts found for "${query}"`,
        });
      }
    } catch (error) {
      console.error('Search error:', error);
      Toast.show({
        type: 'error',
        title: 'Search failed',
        message: error.message,
      });
    } finally {
      store.setState('ui.loading', false);
    }
  },

  /**
   * 更新用户信息卡片
   * @param {Object} accountData - 账户数据
   */
  updateUserInfo(accountData) {
    const accountIdEl = document.getElementById('sidebar-account-id');
    const balanceEl = document.getElementById('sidebar-balance');
    
    if (accountIdEl && accountData?.rsAddress) {
      accountIdEl.textContent = Formatters.truncateAddress(accountData.rsAddress, 12, 6);
    }
    
    if (balanceEl && accountData?.balanceNQT !== undefined) {
      balanceEl.textContent = Formatters.formatNrc(accountData.balanceNQT, 2);
    }
  },

  /**
   * 更新连接状态指示器
   * @param {string} status - 连接状态
   */
  updateConnectionStatus(status) {
    const dot = document.getElementById('connection-dot');
    const label = document.getElementById('connection-label');
    
    if (dot && label) {
      dot.className = `status-dot ${status}`;
      
      switch (status) {
        case 'connected':
          label.textContent = 'Connected';
          break;
        case 'connecting':
          label.textContent = 'Connecting...';
          dot.style.background = 'var(--accent-warning)';
          break;
        case 'disconnected':
        default:
          label.textContent = 'Disconnected';
          break;
      }
    }
  },

  /**
   * 更新区块高度显示
   * @param {number} height - 区块高度
   */
  updateHeight(height) {
    const heightLabel = document.getElementById('height-label');
    
    if (heightLabel) {
      heightLabel.textContent = `Height: ${Formatters.formatNumber(height)}`;
    }
  },

  /**
   * 更新区块链同步状态显示
   * 
   * 根据同步状态显示不同的UI:
   * - scanning: 正在扫描区块链 (黄色闪烁)
   * - downloading: 正在下载区块 (蓝色进度条)
   * - synced: 已同步完成 (绿色)
   * - error: 同步错误 (红色)
   * 
   * @param {Object} syncData - 同步数据
   * @param {string} syncData.status - 同步状态
   * @param {number} syncData.progress - 进度百分比 (0-100)
   * @param {number} syncData.currentHeight - 当前高度
   * @param {number} syncData.targetHeight - 目标高度
   */
  updateSyncStatus(syncData) {
    const forgingDot = document.getElementById('forging-dot');
    const heightLabel = document.getElementById('height-label');

    if (!syncData || !forgingDot || !heightLabel) return;

    const { status, progress = 0, currentHeight = 0, targetHeight = 0 } = syncData;

    switch (status) {
      case 'scanning':
        // 扫描中状态
        forgingDot.className = 'status-dot scanning';
        forgingDot.style.background = '#f59e0b';
        forgingDot.style.boxShadow = '0 0 8px rgba(245, 158, 11, 0.6)';
        forgingDot.style.animation = 'pulse 1.5s infinite';
        heightLabel.innerHTML = `
          <span class="sync-status-text scanning">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align: middle; margin-right: 4px;">
              <circle cx="11" cy="11" r="8"/>
              <path d="m21 21-4.35-4.35"/>
            </svg>
            Scanning...
          </span>
        `;
        break;

      case 'downloading':
        // 下载中状态
        forgingDot.className = 'status-dot downloading';
        forgingDot.style.background = '#3b82f6';
        forgingDot.style.boxShadow = '0 0 8px rgba(59, 130, 246, 0.6)';
        forgingDot.style.animation = 'blink 1s infinite';
        
        const progressBar = Math.min(Math.round(progress), 100);
        heightLabel.innerHTML = `
          <span class="sync-status-text downloading">
            Syncing ${progressBar}%
            ${currentHeight > 0 && targetHeight > 0 ? `(${Formatters.formatNumber(currentHeight)}/${Formatters.formatNumber(targetHeight)})` : ''}
          </span>
        `;
        break;

      case 'synced':
        // 已同步完成
        forgingDot.className = 'status-dot synced';
        forgingDot.style.background = '#10b981';
        forgingDot.style.boxShadow = '0 0 8px rgba(16, 185, 129, 0.4)';
        forgingDot.style.animation = 'none';
        heightLabel.textContent = `Height: ${Formatters.formatNumber(currentHeight || targetHeight)}`;
        break;

      case 'error':
        // 同步错误
        forgingDot.className = 'status-dot error';
        forgingDot.style.background = '#ef4444';
        forgingDot.style.boxShadow = '0 0 8px rgba(239, 68, 68, 0.4)';
        forgingDot.style.animation = 'none';
        heightLabel.innerHTML = `
          <span class="sync-status-text error">Sync Error</span>
        `;
        break;

      default:
        // 未知状态
        forgingDot.style.background = 'var(--text-muted)';
        forgingDot.style.boxShadow = 'none';
        forgingDot.style.animation = 'none';
        if (targetHeight > 0) {
          heightLabel.textContent = `Height: ${Formatters.formatNumber(targetHeight)}`;
        }
    }

    // 添加动态样式（如果不存在）
    this.injectSyncStyles();
  },

  /**
   * 注入同步状态相关的CSS样式
   */
  injectSyncStyles() {
    if (document.getElementById('sidebar-sync-styles')) return;

    const style = document.createElement('style');
    style.id = 'sidebar-sync-styles';
    style.textContent = `
      .sync-status-text {
        font-size: 11px;
        display: inline-flex;
        align-items: center;
        gap: 2px;
      }
      
      .sync-status-text.scanning {
        color: #f59e0b;
        font-weight: 500;
      }
      
      .sync-status-text.downloading {
        color: #3b82f6;
        font-weight: 500;
      }
      
      .sync-status-text.error {
        color: #ef4444;
        font-weight: 600;
      }
      
      @keyframes pulse {
        0%, 100% { opacity: 1; transform: scale(1); }
        50% { opacity: 0.5; transform: scale(1.1); }
      }
      
      @keyframes blink {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.3; }
      }
      
      /* 移动端优化 */
      @media (max-width: 768px) {
        .sync-status-text {
          font-size: 10px;
        }
      }
    `;
    
    document.head.appendChild(style);
  },

  // ========== P2-UI-2: 子菜单展开/折叠交互 (Treeview) ==========
  
  /**
   * 初始化 Treeview 菜单
   * 
   * 参考 NRCS Bootstrap treeview 组件:
   * - 支持多级嵌套菜单
   * - 展开/折叠动画
   * - 键盘导航支持
   * - 状态持久化 (localStorage)
   */
  initTreeview() {
    const menuItems = this.sidebar?.querySelectorAll('.nav-item.has-submenu');
    
    if (!menuItems || menuItems.length === 0) {
      console.debug('[Sidebar] No submenu items found');
      return;
    }
    
    // 为每个有子菜单的项添加事件监听
    menuItems.forEach(item => {
      const toggle = item.querySelector('.submenu-toggle, .nav-link');
      const submenu = item.querySelector('.submenu, .sub-nav');
      
      if (!toggle || !submenu) return;
      
      // 检查是否应该默认展开 (从 localStorage 恢复)
      const itemId = item.dataset.menuId || item.id;
      const wasExpanded = localStorage.getItem(`sidebar_menu_${itemId}`) === 'expanded';
      if (wasExpanded) {
        item.classList.add('expanded');
        submenu.style.display = 'block';
      }
      
      // 点击切换子菜单
      toggle.addEventListener('click', (e) => {
        e.preventDefault();
        this.toggleSubmenu(item);
      });
      
      // 键盘支持: Enter/Space 切换
      toggle.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          this.toggleSubmenu(item);
        }
        
        // 键盘导航: 右箭头展开，左箭头折叠
        if (e.key === 'ArrowRight' && !item.classList.contains('expanded')) {
          e.preventDefault();
          this.toggleSubmenu(item);
        }
        
        if (e.key === 'ArrowLeft' && item.classList.contains('expanded')) {
          e.preventDefault();
          this.toggleSubmenu(item);
        }
      });
      
      // 设置 ARIA 属性
      toggle.setAttribute('aria-expanded', item.classList.contains('expanded'));
      toggle.setAttribute('aria-controls', submenu.id || `submenu-${itemId}`);
      submenu.setAttribute('role', 'menu');
    });
    
    console.log(`[Sidebar] Treeview initialized with ${menuItems.length} expandable items`);
  },

  /**
   * 切换子菜单显示/隐藏
   * 
   * @param {HTMLElement} menuItem - 父级菜单项元素
   */
  toggleSubmenu(menuItem) {
    const isExpanded = menuItem.classList.contains('expanded');
    const submenu = menuItem.querySelector('.submenu, .sub-nav');
    const toggle = menuItem.querySelector('.submenu-toggle, .nav-link');
    
    if (!submenu) return;
    
    if (isExpanded) {
      // 折叠
      menuItem.classList.remove('expanded');
      submenu.style.maxHeight = `${submenu.scrollHeight}px`;
      
      // 触发重排以启用过渡动画
      requestAnimationFrame(() => {
        submenu.style.maxHeight = '0';
        submenu.style.opacity = '0';
      });
      
      setTimeout(() => {
        submenu.style.display = 'none';
        submenu.removeAttribute('style');
      }, 300);  // 与 CSS 过渡时间匹配
      
      if (toggle) toggle.setAttribute('aria-expanded', 'false');
      
    } else {
      // 展开
      menuItem.classList.add('expanded');
      submenu.style.display = 'block';
      submenu.style.maxHeight = '0';
      submenu.style.opacity = '0';
      
      // 触发重排以启用过渡动画
      requestAnimationFrame(() => {
        submenu.style.maxHeight = `${submenu.scrollHeight}px`;
        submenu.style.opacity = '1';
      });
      
      // 动画结束后清除内联样式
      setTimeout(() => {
        submenu.removeAttribute('style');
      }, 300);
      
      if (toggle) toggle.setAttribute('aria-expanded', 'true');
    }
    
    // 保存状态到 localStorage
    const itemId = menuItem.dataset.menuId || menuItem.id;
    if (itemId) {
      localStorage.setItem(
        `sidebar_menu_${itemId}`, 
        isExpanded ? 'collapsed' : 'expanded'
      );
    }
    
    // 可选: 关闭同级其他展开的菜单 (手风琴模式)
    // this.closeSiblingMenus(menuItem);
  },

  /**
   * 关闭同级的其他展开菜单 (手风琴模式)
   * 
   * @param {HTMLElement} currentMenu - 当前操作的菜单项
   */
  closeSiblingMenus(currentMenu) {
    const parent = currentMenu.parentElement;
    const siblings = parent.querySelectorAll(':scope > .nav-item.has-submenu.expanded');
    
    siblings.forEach(sibling => {
      if (sibling !== currentMenu && sibling !== parent) {
        this.toggleSubmenu(sibling);
      }
    });
  },

  /**
   * 展开所有子菜单
   */
  expandAllMenus() {
    const items = this.sidebar?.querySelectorAll('.nav-item.has-submenu:not(.expanded)');
    items?.forEach(item => this.toggleSubmenu(item));
  },

  /**
   * 折叠所有子菜单
   */
  collapseAllMenus() {
    const items = this.sidebar?.querySelectorAll('.nav-item.has-submenu.expanded');
    items?.forEach(item => this.toggleSubmenu(item));
  },
};

// 导出供全局使用
window.Sidebar = Sidebar;
