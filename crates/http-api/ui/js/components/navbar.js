/**
 * NRCS Wallet - Navbar Component
 * 
 * 顶部导航栏组件
 * 处理: 面包屑导航、操作按钮、用户菜单等
 */

const Navbar = {
  /**
   * 初始化导航栏
   */
  init() {
    this.navbar = document.querySelector('.navbar');
    this.menuToggleBtn = document.getElementById('menu-toggle-btn');
    
    if (!this.navbar) return;
    
    this.bindEvents();
    console.log('Navbar initialized');
  },

  /**
   * 绑定事件监听器
   */
  bindEvents() {
    // 移动端菜单切换按钮
    if (this.menuToggleBtn) {
      this.menuToggleBtn.addEventListener('click', () => Sidebar.toggle());
    }
    
    // 发送 NRC 按钮
    const sendMoneyBtn = document.getElementById('btn-send-money');
    if (sendMoneyBtn) {
      sendMoneyBtn.addEventListener('click', () => this.openSendModal());
    }
    
    // 通知按钮
    const notifBtn = document.getElementById('btn-notifications');
    if (notifBtn) {
      notifBtn.addEventListener('click', () => this.showNotifications());
    }
    
    // 用户菜单按钮
    const userMenuBtn = document.getElementById('user-menu-btn');
    if (userMenuBtn) {
      userMenuBtn.addEventListener('click', (e) => this.toggleUserMenu(e));
    }
    
    // 模态框关闭按钮
    document.querySelectorAll('[data-close-modal]').forEach(btn => {
      btn.addEventListener('click', () => this.closeModal());
    });
    
    // 点击模态框外部关闭
    const modalOverlay = document.getElementById('send-modal');
    if (modalOverlay) {
      modalOverlay.addEventListener('click', (e) => {
        if (e.target === modalOverlay) {
          this.closeModal();
        }
      });
    }
    
    // 发送表单提交
    const sendForm = document.getElementById('send-form');
    if (sendForm) {
      sendForm.addEventListener('submit', (e) => this.handleSendSubmit(e));
    }
  },

  /**
   * 更新面包屑导航
   * @param {Array<{label, link?}>} items - 面包屑项目数组
   */
  updateBreadcrumb(items) {
    const breadcrumb = document.getElementById('breadcrumb');
    const currentPageTitle = document.getElementById('current-page-title');
    
    if (!breadcrumb) return;
    
    let html = '';
    
    items.forEach((item, index) => {
      if (index > 0) {
        html += '<span class="breadcrumb-separator">/</span>';
      }
      
      const isLast = index === items.length - 1;
      
      if (isLast || !item.link) {
        html += `<span class="breadcrumb-item current">${item.label}</span>`;
        
        // 更新当前页面标题
        if (currentPageTitle) {
          currentPageTitle.textContent = item.label;
        }
      } else {
        html += `<a href="${item.link}" class="breadcrumb-item">${item.label}</a>`;
      }
    });
    
    breadcrumb.innerHTML = html;
  },

  /**
   * 打开发送交易模态框
   */
  openSendModal() {
    const modal = document.getElementById('send-modal');
    
    if (modal) {
      modal.classList.add('active');
      document.body.style.overflow = 'hidden';
      
      // 聚焦到第一个输入框
      setTimeout(() => {
        const firstInput = modal.querySelector('.form-input');
        if (firstInput) firstInput.focus();
      }, 100);
    }
  },

  /**
   * 关闭模态框
   */
  closeModal() {
    const modals = document.querySelectorAll('.modal-overlay.active');
    
    modals.forEach(modal => {
      modal.classList.remove('active');
    });
    
    document.body.style.overflow = '';
  },

  /**
   * 处理发送交易表单提交
   */
  async handleSendSubmit(event) {
    event.preventDefault();
    
    const recipient = document.getElementById('send-recipient').value.trim();
    const amount = parseFloat(document.getElementById('send-amount').value);
    const fee = parseFloat(document.getElementById('send-fee').value) || 0.1;
    const message = document.getElementById('send-message').value.trim();
    
    // 验证输入
    if (!recipient) {
      Toast.show({ type: 'error', title: 'Error', message: 'Please enter a recipient address' });
      return;
    }
    
    if (!amount || amount <= 0) {
      Toast.show({ type: 'error', title: 'Error', message: 'Please enter a valid amount' });
      return;
    }
    
    const submitBtn = document.getElementById('send-submit-btn');
    const originalText = submitBtn.innerHTML;
    
    try {
      // 显示加载状态
      submitBtn.disabled = true;
      submitBtn.innerHTML = `
        <svg class="spinner-circle" style="width:16px;height:16px;border-width:2px;"></svg>
        <span>Sending...</span>
      `;
      
      // 转换为 NQT (8位小数)
      const amountNQT = Math.round(amount * 1e8);
      const feeNQT = Math.round(fee * 1e8);
      
      // 调用 API 发送交易
      const result = await api.sendMoney({
        recipient,
        amountNQT,
        feeNQT,
        secretPhrase: store.state.secretPhrase,
        ...(message ? { message, messageIsText: true } : {}),
      });
      
      if (result.errorCode) {
        throw new Error(result.errorDescription || 'Transaction failed');
      }
      
      // 成功提示
      Toast.show({
        type: 'success',
        title: 'Transaction Sent',
        message: `Successfully sent ${amount} NRC to ${recipient}`,
      });
      
      // 关闭模态框并重置表单
      this.closeModal();
      document.getElementById('send-form').reset();
      
      // 刷新交易列表
      if (typeof TransactionsPage !== 'undefined') {
        await TransactionsPage.refresh();
      }
      
    } catch (error) {
      console.error('Send transaction error:', error);
      Toast.show({
        type: 'error',
        title: 'Transaction Failed',
        message: error.message,
      });
    } finally {
      // 恢复按钮状态
      submitBtn.disabled = false;
      submitBtn.innerHTML = originalText;
    }
  },

  /**
   * 显示通知面板 (简化版)
   */
  showNotifications() {
    // TODO: 实现完整的通知系统
    Toast.show({
      type: 'info',
      title: 'Notifications',
      message: 'Notification panel coming soon!',
    });
  },

  /**
   * 切换用户菜单下拉
   */
  toggleUserMenu(event) {
    event.stopPropagation();
    
    // 简化实现：直接登出确认
    const shouldLogout = confirm('Do you want to logout and lock the wallet?');
    
    if (shouldLogout) {
      App.logout();
    }
  },
};

// 导出供全局使用
window.Navbar = Navbar;

/**
 * Toast 通知工具类
 */
const Toast = {
  container: null,

  /**
   * 显示 Toast 通知
   * @param {Object} options - 配置选项
   * @param {string} options.type - 类型 (success/error/warning/info)
   * @param {string} options.title - 标题
   * @param {string} options.message - 消息内容
   * @param {number} options.duration - 显示时长 (ms), 默认 4000
   */
  show(options = {}) {
    this.initContainer();
    
    const {
      type = 'info',
      title = '',
      message = '',
      duration = 4000,
    } = options;
    
    const icons = {
      success: '✓',
      error: '✕',
      warning: '⚠',
      info: 'ℹ',
    };
    
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.innerHTML = `
      <div style="color: var(--accent-${type === 'info' ? 'primary' : type}); font-size: 1.25rem;">
        ${icons[type]}
      </div>
      <div class="toast-content">
        <div class="toast-title">${title}</div>
        ${message ? `<div class="toast-message">${message}</div>` : ''}
      </div>
      <button class="toast-close" onclick="this.parentElement.remove()">×</button>
    `;
    
    this.container.appendChild(toast);
    
    // 自动移除
    if (duration > 0) {
      setTimeout(() => {
        toast.style.animation = 'toastSlideOut 0.3s ease-in forwards';
        setTimeout(() => toast.remove(), 300);
      }, duration);
    }
  },

  /**
   * 初始化容器
   */
  initContainer() {
    if (!this.container) {
      this.container = document.getElementById('toast-container');
    }
  },
};

// 导出供全局使用
window.Toast = Toast;
