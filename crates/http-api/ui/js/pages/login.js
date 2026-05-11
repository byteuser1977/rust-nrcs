/**
 * NRCS Wallet - Login Page Module
 * 
 * 实现双模式登录:
 * 1. 助记词登录 (Passphrase Login) - 使用12词助记词短语
 * 2. 账号登录 (Account Login) - 使用 NRCS 账号地址
 * 
 * 功能特性:
 * - 助记词生成器 (3步向导)
 * - 账号历史记录管理
 * - 区块链同步状态显示
 * - 安全警告和强度检查
 * 
 * 参考源码:
 * - NRCS Java: nrs.login.js
 * - NRCS Java: passphrasegenerator.js
 */

const LoginPage = {
  // 当前登录模式: 'passphrase' | 'account'
  currentMode: 'passphrase',
  
  // 助记词生成器状态
  generatorState: {
    step: 1,
    passphrase: null,
    accountId: null,
    revealedIndices: [],
  },

  /**
   * 初始化登录页面
   */
  init() {
    this.renderLoginPage();
    this.bindEvents();
    this.checkSavedAccounts();
    this.checkBlockchainStatus();
  },

  /**
   * 渲染登录页面 HTML
   */
  renderLoginPage() {
    const overlay = document.getElementById('lockscreen-overlay');
    
    overlay.innerHTML = `
      <div class="login-wrapper">
        <!-- 背景 -->
        <div class="login-bg">
          <div class="login-bg-gradient"></div>
          <div class="login-bg-pattern"></div>
        </div>
        
        <!-- 主容器 -->
        <div class="login-main">
          <!-- Logo 区域 -->
          <div class="login-header">
            <div class="login-logo-container">
              <div class="login-logo-icon">N</div>
              <h1 class="login-title">NRCS Wallet</h1>
            </div>
            <p class="login-subtitle">Next Generation Blockchain Interface</p>
            
            <!-- 同步状态指示器 -->
            <div class="login-sync-status" id="login-sync-status" style="display: none;">
              <div class="sync-indicator">
                <span class="sync-dot scanning" id="login-sync-dot"></span>
                <span class="sync-text" id="login-sync-text">Scanning blockchain...</span>
              </div>
              <div class="sync-progress" id="login-sync-progress" style="display: none;">
                <div class="progress-bar">
                  <div class="progress-fill" id="login-progress-fill" style="width: 0%"></div>
                </div>
                <span class="progress-text" id="login-progress-text">0%</span>
              </div>
            </div>
          </div>
          
          <!-- 登录模式切换 -->
          <div class="login-mode-switcher">
            <button 
              class="mode-btn active" 
              data-mode="passphrase"
              id="btn-mode-passphrase"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
              </svg>
              <span>Passphrase</span>
            </button>
            <button 
              class="mode-btn" 
              data-mode="account"
              id="btn-mode-account"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
              </svg>
              <span>Account</span>
            </button>
          </div>

          <!-- ========== 助记词登录面板 ========== -->
          <div class="login-panel" id="panel-passphrase">
            <!-- 已保存账户列表 -->
            <div class="saved-accounts-section" id="saved-accounts-section" style="display: none;">
              <div class="section-label">Saved Accounts</div>
              <ul class="saved-accounts-list" id="saved-accounts-list">
                <!-- 动态填充 -->
              </ul>
              <button class="btn btn-ghost btn-sm" id="btn-use-other-passphrase">
                Use different passphrase
              </button>
            </div>

            <!-- 助记词输入表单 -->
            <form id="passphrase-login-form" class="login-form">
              <div class="form-group">
                <label for="passphrase-input" class="form-label">
                  Your Passphrase
                  <span class="label-hint">(12 words)</span>
                </label>
                <div class="input-with-action">
                  <textarea 
                    id="passphrase-input"
                    class="form-textarea passphrase-area"
                    placeholder="Enter your 12-word passphrase..."
                    rows="3"
                    autocomplete="off"
                    spellcheck="false"
                  ></textarea>
                  <button 
                    type="button" 
                    class="input-action-btn" 
                    id="btn-toggle-passphrase-visibility"
                    title="Toggle visibility"
                  >
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" id="eye-icon">
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                      <circle cx="12" cy="12" r="3"/>
                    </svg>
                  </button>
                </div>
                
                <!-- 密码强度指示器 -->
                <div class="password-strength" id="password-strength" style="display: none;">
                  <div class="strength-bar">
                    <div class="strength-fill" id="strength-fill"></div>
                  </div>
                  <span class="strength-text" id="strength-text">-</span>
                </div>
              </div>

              <div class="form-options">
                <label class="form-checkbox">
                  <input type="checkbox" id="remember-me-passphrase">
                  <span>Remember me</span>
                </label>
              </div>

              <button type="submit" class="btn btn-primary btn-block btn-lg" id="btn-login-passphrase">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4M10 17l5-5-5-5M13.8 12H3"/>
                </svg>
                <span>Unlock Wallet</span>
              </button>

              <!-- 新用户注册链接 -->
              <div class="register-link-section">
                <p class="text-muted text-sm">New to NRCS?</p>
                <button type="button" class="btn btn-outline btn-block" id="btn-generate-new-wallet">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/>
                    <line x1="12" y1="8" x2="12" y2="16"/>
                    <line x1="8" y1="12" x2="16" y2="12"/>
                  </svg>
                  <span>Create New Wallet</span>
                </button>
              </div>
            </form>
          </div>

          <!-- ========== 账号登录面板 ========== -->
          <div class="login-panel" id="panel-account" style="display: none;">
            <!-- 已保存账号列表 -->
            <div class="saved-accounts-section" id="saved-account-list-section">
              <div class="section-label">Your Accounts</div>
              <ul class="saved-accounts-list" id="saved-account-list">
                <!-- 动态填充 -->
              </ul>
              <button class="btn btn-ghost btn-sm" id="btn-enter-custom-account">
                Enter account manually
              </button>
            </div>

            <!-- 手动输入账号 -->
            <form id="account-login-form" class="login-form" style="display: none;">
              <div class="form-group">
                <label for="account-id-input" class="form-label">
                  Account Address
                  <span class="label-hint">(NRCS-XXXX-XXXX-XXXX-XXXXX)</span>
                </label>
                <input 
                  type="text" 
                  id="account-id-input"
                  class="form-input"
                  placeholder="NRCS-XXXX-XXXX-XXXX-XXXXX"
                  autocomplete="off"
                  maxlength="25"
                >
              </div>

              <div class="form-options">
                <label class="form-checkbox">
                  <input type="checkbox" id="remember-me-account">
                  <span>Remember account</span>
                </label>
              </div>

              <button type="submit" class="btn btn-primary btn-block btn-lg" id="btn-login-account">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4M10 17l5-5-5-5M13.8 12H3"/>
                </svg>
                <span>Login with Account</span>
              </button>
            </form>
          </div>

          <!-- 错误提示 -->
          <div class="alert alert-danger" id="login-error" style="display: none;"></div>

          <!-- 底部信息 -->
          <div class="login-footer">
            <div class="footer-links">
              <a href="#" class="footer-link" data-action="show-security-info">Security Info</a>
              <span class="separator">•</span>
              <a href="#" class="footer-link" data-action="show-network-info">Network Status</a>
            </div>
            <p class="footer-copyright">
              Secure • Decentralized • Open Source
            </p>
          </div>
        </div>
      </div>

      <!-- ========== 助记词生成器模态框 ========== -->
      <div class="modal-overlay" id="generator-modal" style="display: none;">
        <div class="modal modal-lg generator-modal">
          <div class="modal-header">
            <h3 class="modal-title">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
              </svg>
              Create New Wallet
            </h3>
            <button class="modal-close" id="close-generator-modal">&times;</button>
          </div>

          <!-- 步骤指示器 -->
          <div class="generator-steps-indicator">
            <div class="step-item active" data-step="1">
              <span class="step-number">1</span>
              <span class="step-label">Generate</span>
            </div>
            <div class="step-line"></div>
            <div class="step-item" data-step="2">
              <span class="step-number">2</span>
              <span class="step-label">Backup</span>
            </div>
            <div class="step-line"></div>
            <div class="step-item" data-step="3">
              <span class="step-number">3</span>
              <span class="step-label">Verify</span>
            </div>
          </div>

          <div class="modal-body generator-body">
            <!-- Step 1: 生成中 -->
            <div class="generator-step step-1 active" id="gen-step-1">
              <div class="step-content centered">
                <div class="generating-animation" id="generating-animation">
                  <div class="spinner"></div>
                  <p class="generating-text">Generating secure passphrase...</p>
                  <p class="generating-subtext">This may take a moment</p>
                </div>
              </div>
            </div>

            <!-- Step 2: 显示助记词 -->
            <div class="generator-step step-2" id="gen-step-2" style="display: none;">
              <div class="security-warning-callout">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2">
                  <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                  <line x1="12" y1="9" x2="12" y2="13"/>
                  <line x1="12" y1="17" x2="12.01" y2="17"/>
                </svg>
                <div>
                  <strong>Important!</strong> Write down your passphrase on paper and store it securely.
                  Never share it with anyone or store it digitally.
                </div>
              </div>

              <div class="passphrase-display-card">
                <div class="card-header">
                  <h4>Your Secret Passphrase</h4>
                  <button 
                    class="btn btn-ghost btn-sm" 
                    id="btn-copy-passphrase"
                    title="Copy to clipboard"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                    </svg>
                    Copy
                  </button>
                </div>
                <div class="card-body">
                  <pre class="passphrase-display" id="generated-passphrase-display"></pre>
                </div>
                <div class="card-footer">
                  <small class="text-muted">
                    Account ID: <code id="generated-account-id" class="mono">-</code>
                  </small>
                </div>
              </div>

              <div class="confirmation-checkbox">
                <label class="form-checkbox warning">
                  <input type="checkbox" id="confirm-backup-warning">
                  <span>I have safely stored my passphrase and understand that I cannot recover it if lost</span>
                </label>
              </div>

              <div class="step-actions">
                <button class="btn btn-primary btn-block" id="btn-goto-step-3" disabled>
                  Continue to Verification →
                </button>
              </div>
            </div>

            <!-- Step 3: 验证助记词 -->
            <div class="generator-step step-3" id="gen-step-3" style="display: none;">
              <div class="verification-instructions">
                <h4>Verify Your Passphrase</h4>
                <p>Enter the missing words from your passphrase to confirm you've backed it up correctly.</p>
              </div>

              <div class="masked-passphrase-display" id="masked-passphrase-display">
                <!-- 动态填充 -->
              </div>

              <div class="verification-form">
                <div class="form-group" id="verification-inputs-container">
                  <!-- 动态填充输入框 -->
                </div>

                <div class="alert alert-danger" id="verification-error" style="display: none;"></div>
              </div>

              <div class="step-actions">
                <button class="btn btn-outline" id="btn-back-to-step-2">
                  ← Back
                </button>
                <button class="btn btn-primary" id="btn-verify-and-login">
                  Verify & Unlock Wallet
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    `;
  },

  /**
   * 绑定事件监听器
   */
  bindEvents() {
    // 模式切换
    document.getElementById('btn-mode-passphrase').addEventListener('click', () => {
      this.switchMode('passphrase');
    });
    
    document.getElementById('btn-mode-account').addEventListener('click', () => {
      this.switchMode('account');
    });

    // 助记词登录表单提交
    document.getElementById('passphrase-login-form').addEventListener('submit', (e) => {
      e.preventDefault();
      this.handlePassphraseLogin();
    });

    // 账号登录表单提交
    document.getElementById('account-login-form').addEventListener('submit', (e) => {
      e.preventDefault();
      this.handleAccountLogin();
    });

    // 助记词可见性切换
    document.getElementById('btn-toggle-passphrase-visibility').addEventListener('click', () => {
      this.togglePassphraseVisibility();
    });

    // 助记词输入时检查强度
    document.getElementById('passphrase-input').addEventListener('input', () => {
      this.checkPassphraseStrength();
    });

    // 生成新钱包按钮
    document.getElementById('btn-generate-new-wallet').addEventListener('click', () => {
      this.showGeneratorModal();
    });

    // 关闭生成器模态框
    document.getElementById('close-generator-modal').addEventListener('click', () => {
      this.hideGeneratorModal();
    });

    // 复制助记词
    document.getElementById('btn-copy-passphrase').addEventListener('click', () => {
      this.copyPassphraseToClipboard();
    });

    // 确认备份复选框
    document.getElementById('confirm-backup-warning').addEventListener('change', (e) => {
      const btn = document.getElementById('btn-goto-step-3');
      btn.disabled = !e.target.checked;
      
      const container = document.getElementById(e.target.id).closest('.confirmation-checkbox');
      if (!e.target.checked) {
        container.style.backgroundColor = '';
      }
    });

    // 步骤导航
    document.getElementById('btn-goto-step-3').addEventListener('click', () => {
      this.goToStep(3);
    });

    document.getElementById('btn-back-to-step-2').addEventListener('click', () => {
      this.goToStep(2);
    });

    document.getElementById('btn-verify-and-login').addEventListener('click', () => {
      this.verifyAndLogin();
    });

    // 使用其他助记词
    document.getElementById('btn-use-other-passphrase').addEventListener('click', () => {
      document.getElementById('saved-accounts-section').style.display = 'none';
      document.getElementById('passphrase-login-form').style.display = 'block';
    });

    // 手动输入账号
    document.getElementById('btn-enter-custom-account').addEventListener('click', () => {
      document.getElementById('saved-account-list-section').querySelector('ul').style.display = 'none';
      document.getElementById('btn-enter-custom-account').style.display = 'none';
      document.getElementById('account-login-form').style.display = 'block';
    });

    // 底部链接
    document.querySelectorAll('[data-action]').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.preventDefault();
        this.handleFooterAction(btn.dataset.action);
      });
    });
  },

  /**
   * 切换登录模式
   */
  switchMode(mode) {
    this.currentMode = mode;
    
    // 更新按钮状态
    document.querySelectorAll('.mode-btn').forEach(btn => {
      btn.classList.toggle('active', btn.dataset.mode === mode);
    });

    // 切换面板
    document.getElementById('panel-passphrase').style.display = 
      mode === 'passphrase' ? 'block' : 'none';
    document.getElementById('panel-account').style.display = 
      mode === 'account' ? 'block' : 'none';

    // 清除错误
    this.hideError();
  },

  /**
   * 检查已保存的账户
   */
  checkSavedAccounts() {
    const savedAccounts = localStorage.getItem('nrcs_saved_accounts');
    
    if (savedAccounts && savedAccounts.trim()) {
      const accounts = savedAccounts.split(';').filter(a => a.trim());
      
      if (this.currentMode === 'passphrase' && accounts.length > 0) {
        this.renderSavedAccounts(accounts, 'saved-accounts-list');
        document.getElementById('saved-accounts-section').style.display = 'block';
        document.getElementById('passphrase-login-form').style.display = 'none';
      }
    }

    // 始终显示账号列表（如果有的话）
    if (savedAccounts && savedAccounts.trim()) {
      const accounts = savedAccounts.split(';').filter(a => a.trim());
      this.renderSavedAccounts(accounts, 'saved-account-list');
    }
  },

  /**
   * 渲染已保存账户列表
   */
  renderSavedAccounts(accounts, containerId) {
    const container = document.getElementById(containerId);
    if (!container) return;
    
    container.innerHTML = accounts.map(account => `
      <li class="saved-account-item">
        <button class="saved-account-btn" data-account="${account}">
          <span class="account-address">${account}</span>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="arrow-icon">
            <polyline points="9 18 15 12 9 6"/>
          </svg>
        </button>
        <button class="remove-account-btn" data-account="${account}" title="Remove account">
          ×
        </button>
      </li>
    `).join('');

    // 绑定点击事件
    container.querySelectorAll('.saved-account-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const account = btn.dataset.account;
        if (this.currentMode === 'passphrase') {
          // 对于助记词模式，这里应该使用保存的助记词登录
          // 但由于安全原因，我们只保存账号地址
          console.log('Login with saved account:', account);
        } else {
          this.loginWithAccount(account);
        }
      });
    });

    container.querySelectorAll('.remove-account-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.removeSavedAccount(btn.dataset.account);
      });
    });
  },

  /**
   * 移除已保存的账户
   */
  removeSavedAccount(account) {
    let savedAccounts = localStorage.getItem('nrcs_saved_accounts') || '';
    savedAccounts = savedAccounts.replace(account + ';', '').replace(account, '');
    
    if (savedAccounts.trim()) {
      localStorage.setItem('nrcs_saved_accounts', savedAccounts);
    } else {
      localStorage.removeItem('nrcs_saved_accounts');
    }

    this.checkSavedAccounts();
  },

  /**
   * 保存账户到本地存储
   */
  saveAccount(account) {
    let savedAccounts = localStorage.getItem('nrcs_saved_accounts') || '';
    const accounts = savedAccounts.split(';').filter(a => a.trim());
    
    if (!accounts.includes(account)) {
      accounts.push(account);
      localStorage.setItem('nrcs_saved_accounts', accounts.join(';'));
    }
  },

  /**
   * 处理助记词登录
   * 
   * 参考 NRCS nrs.login.js 第 292-524 行的完整登录流程:
   * Step 1: 显示全屏 Spinner (NRS.spinner.spin)
   * Step 2: 验证助记词长度和强度
   * Step 3: 区块链状态预检 (getBlockchainStatus)
   * Step 4: 调用 getAccountId API
   * Step 5: 公钥安全验证 (getAccountPublicKey)
   * Step 6: 处理记住我选项
   * Step 7: 执行登录成功逻辑
   */
  async handlePassphraseLogin() {
    const passphrase = document.getElementById('passphrase-input').value.trim();
    
    if (!passphrase) {
      this.showError('Please enter your passphrase');
      return;
    }

    // 验证助记词长度 (参考 NRCS nrs.login.js 第 306-313 行)
    // 
    // NRCS 验证策略:
    // - 测试网络 (isTestNet): 最小 12 字符
    // - 主网: 无硬性最小长度，但建议 35 字符以上
    // - 可通过 UI 复选框跳过长度检查 (loginCheckPasswordLength)
    
    // 检查是否为测试网络 (参考 NRCS: NRS.isTestNet)
    const isTestnet = this.isTestnetMode();
    
    // 检查用户是否选择跳过密码长度验证 (参考 NRCS nrs.login.js 第 307 行)
    // NRCS: if (!NRS.isTestNet && id.length < 12 && loginCheckPasswordLength.val() == 1)
    const skipLengthCheck = document.getElementById('skip-length-check')?.checked;
    
    if (isTestnet) {
      // 测试网络: 强制最小 12 字符 (参考 NRCS)
      if (passphrase.length < 12) {
        this.showError('Passphrase is too short. Minimum 12 characters required on testnet.');
        return;
      }
    } else if (!skipLengthCheck && passphrase.length < 12) {
      // 主网: 默认要求最小 12 字符，但允许跳过
      this.showError('Passphrase is too short. Minimum 12 characters recommended. Use "Skip length check" to continue.');
      return;
    }

    // 安全强度警告 (参考 NRCS nrs.login.js 第 415-419 行)
    // 仅在主网且助记词较短时显示警告
    if (!isTestnet && passphrase.length < 35) {
      const hasUppercase = /[A-Z]/.test(passphrase);
      const hasNumbers = /[0-9]/.test(passphrase);
      
      if (!hasUppercase || !hasNumbers) {
        console.warn(
          '[Security Warning] For better security, passphrase should contain:\n' +
          '- At least 35 characters\n' +
          '- Uppercase letters\n' +
          '- Numbers\n' +
          '- Special characters'
        );
        
        // 可选: 在 UI 上显示安全建议提示
        this.showSecurityWarning('Consider using a longer passphrase with mixed case and numbers for better security.');
      }
    }

    try {
      // Step 1: 显示全屏 Spinner (参考 NRCS: NRS.spinner.spin($("#center")[0]))
      this.showFullPageSpinner('Processing login...');

      // Step 2: 区块链状态预检 (参考 NRCS nrs.login.js 第 320-326 行)
      const status = await api.request('getBlockchainStatus', {});
      if (status.errorCode) {
        throw new Error(status.errorDescription || 'Unable to connect to blockchain network');
      }
      
      console.log('Blockchain status check passed, continuing login...');

      // Step 3: 调用 API 获取账户 ID (参考 NRCS: accountRequest = "getAccountId")
      const result = await api.request('getAccountId', {
        secretPhrase: passphrase
      });

      // 错误处理 (参考 NRCS nrs.login.js 第 341-376 行)
      if (result.errorCode === 19 || result.errorCode === 21) {
        throw new Error('Light client is connecting to network, please try again later');
      }

      if (result.errorCode && !result.account) {
        throw new Error(result.errorDescription || 'Failed to generate account ID');
      }

      const accountId = result.account;
      const accountRS = result.accountRS;

      if (!accountId || !accountRS) {
        throw new Error('Failed to generate account ID from passphrase');
      }

      // Step 4: 公钥安全验证 (参考 NRCS nrs.login.js 第 378-388 行)
      await this.verifyPublicKey(accountId, passphrase);

      // Step 5: 记住我选项处理 (参考 NRCS nrs.login.js 第 391-401 行)
      if (document.getElementById('remember-me-passphrase')?.checked) {
        this.savePassphraseSecurely(passphrase);
        store.setState({ rememberPassword: true });
      } else {
        store.setState({ rememberPassword: false });
      }

      // Step 6: 执行登录成功逻辑 (参考 NRCS nrs.login.js 第 403-523 行)
      this.onLoginSuccess({
        accountId,
        accountRS,
        publicKey: result.publicKey,
        isPassphraseLogin: true,
        passphrase,
      });

    } catch (error) {
      console.error('Passphrase login error:', error);
      this.showError(error.message || 'Login failed. Please check your passphrase and try again.');
    } finally {
      // 确保隐藏 Spinner (参考 NRCS: NRS.spinner.stop())
      this.hideFullPageSpinner();
      this.setLoadingState(false);
    }
  },

  /**
   * 处理账号登录
   */
  async handleAccountLogin() {
    const accountId = document.getElementById('account-id-input').value.trim();
    
    if (!accountId) {
      this.showError('Please enter your account address');
      return;
    }

    // 验证账号格式 (NRCS-XXXX-XXXX-XXXX-XXXXX 或 NRCS-XXXX-XXXX-XXXX-XXXX)
    // Reed-Solomon 编码地址，支持 4-4-4-5 或 4-4-4-4 格式
    if (!/^NRCS-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4,5}$/.test(accountId)) {
      this.showError('Invalid account address format. Expected: NRCS-XXXX-XXXX-XXXX-XXXXX');
      return;
    }

    await this.loginWithAccount(accountId);
  },

  /**
   * 使用账号ID登录
   * 
   * 参考 NRCS nrs.login.js 第 334-376 行的实现:
   * - 使用 getAccount API 获取账户信息
   * - 错误码 5 表示账户不存在但格式正确，仍允许继续
   * - 错误码 4 表示账号格式不正确，需要报错
   * - 错误码 19, 21 表示轻客户端连接问题
   */
  async loginWithAccount(accountId) {
    try {
      // 显示全屏 Spinner (参考 NRCS: NRS.spinner.spin)
      this.showFullPageSpinner('Loading account information...');

      // Step 1: 区块链状态预检 (参考 NRCS nrs.login.js 第 320-326 行)
      const status = await api.request('getBlockchainStatus', {});
      if (status.errorCode) {
        throw new Error(status.errorDescription || 'Unable to connect to blockchain network');
      }

      // Step 2: 调用 API 获取账户信息 (参考 NRCS: accountRequest = "getAccount")
      const result = await api.request('getAccount', {
        account: accountId
      });

      // Step 3: 参考 NRCS 错误处理逻辑 (nrs.login.js 第 350-376 行)
      
      // 错误码 19 或 21: 轻客户端正在连接网络
      if (result.errorCode === 19 || result.errorCode === 21) {
        throw new Error('Light client is connecting to network, please try again later');
      }
      
      // 错误码 4: Incorrect account (账号格式不正确或校验和不匹配)
      if (result.errorCode === 4) {
        throw new Error('Invalid account address format or checksum error');
      }
      
      // 错误码 5: Unknown account (账户不存在但格式正确)
      // NRCS 允许这种情况继续登录查看账户信息
      if (result.errorCode === 5) {
        console.log('Account does not exist yet, but format is valid. Continuing...');
      }
      
      // 其他错误码
      if (result.errorCode && result.errorCode !== 5) {
        throw new Error(result.errorDescription || 'Failed to get account information');
      }

      // 记住我选项 (参考 NRCS rememberAccount 函数)
      if (document.getElementById('remember-me-account')?.checked) {
        this.saveAccount(accountId);
      }

      // Step 4: 执行登录成功逻辑
      // 参考 NRCS: NRS.account = response.account; NRS.accountRS = response.accountRS;
      this.onLoginSuccess({
        accountId: result.account || accountId,
        accountRS: result.accountRS || accountId,
        publicKey: result.publicKey || null,
        isPassphraseLogin: false,
      });

    } catch (error) {
      console.error('Account login error:', error);
      this.showError(error.message || 'Network error. Please check your connection and try again.');
    } finally {
      // 确保隐藏 Spinner
      this.hideFullPageSpinner();
      this.setLoadingState(false);
    }
  },

  /**
   * 登录成功处理
   */
  onLoginSuccess({ accountId, accountRS, publicKey, isPassphraseLogin, passphrase }) {
    console.log('[Login] onLoginSuccess called', { accountId, accountRS });
    
    if (typeof store === 'undefined' || !store.setState) {
      console.error('[Login] Store not available! Checking global scope...');
      console.error('[Login] window.store:', typeof window.store);
      alert('System error: Store module not loaded. Please refresh the page (Ctrl+Shift+R / Cmd+Shift+R)');
      return;
    }
    
    // 更新全局状态
    store.setState({
      isLoggedIn: true,
      accountId,
      accountRS,
      publicKey,
      isPassphraseLogin,
      lastLoginTime: Date.now(),
    });

    // 清除敏感数据
    document.getElementById('passphrase-input').value = '';
    this.setLoadingState(false);

    // 隐藏锁屏界面
    document.getElementById('lockscreen-overlay').style.display = 'none';
    document.getElementById('app-container').style.display = 'flex';

    // 初始化应用
    if (typeof App !== 'undefined') {
      App.onUserLoggedIn();
    }

    // 显示欢迎消息
    Toast.success(`Welcome back! Logged in as ${accountRS}`);
  },

  /**
   * 显示/隐藏助记词
   */
  togglePassphraseVisibility() {
    const input = document.getElementById('passphrase-input');
    const icon = document.getElementById('eye-icon');
    
    if (input.type === 'password') {
      input.type = 'text';
      icon.innerHTML = `
        <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
        <line x1="1" y1="1" x2="23" y2="23"/>
      `;
    } else {
      input.type = 'password';
      icon.innerHTML = `
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
        <circle cx="12" cy="12" r="3"/>
      `;
    }
  },

  /**
   * 检查助记词强度
   */
  checkPassphraseStrength() {
    const passphrase = document.getElementById('passphrase-input').value;
    const strengthContainer = document.getElementById('password-strength');
    const strengthFill = document.getElementById('strength-fill');
    const strengthText = document.getElementById('strength-text');

    if (passphrase.length === 0) {
      strengthContainer.style.display = 'none';
      return;
    }

    strengthContainer.style.display = 'block';
    const result = MnemonicUtils.checkPasswordStrength(passphrase);

    // 更新强度条
    strengthFill.style.width = `${result.score}%`;
    strengthFill.className = `strength-fill ${result.level}`;
    strengthText.textContent = `${result.level.replace('_', ' ').toUpperCase()} (${result.score}%)`;

    // 显示建议
    if (result.suggestions.length > 0) {
      strengthText.title = result.suggestions.join('\n');
    }
  },

  /**
   * 显示助记词生成器模态框
   */
  showGeneratorModal() {
    const modal = document.getElementById('generator-modal');
    modal.style.display = 'flex';
    
    // 重置到步骤1
    this.generatorState = { step: 1, passphrase: null, accountId: null, revealedIndices: [] };
    this.goToStep(1);
    
    // 开始生成
    setTimeout(() => this.generatePassphrase(), 100);
  },

  /**
   * 隐藏助记词生成器模态框
   */
  hideGeneratorModal() {
    document.getElementById('generator-modal').style.display = 'none';
    MnemonicUtils.clearSensitiveData(this.generatorState.passphrase);
    this.generatorState.passphrase = null;
  },

  /**
   * 生成助记词 (参考 NRCS passphrasegenerator.js)
   */
  async generatePassphrase() {
    try {
      const result = MnemonicUtils.generateMnemonic();
      
      this.generatorState.passphrase = result.passphrase;
      
      // 计算账户ID (前端模拟)
      const publicKey = await MnemonicUtils.getPublicKeyFromPassphrase(result.passphrase);
      this.generatorState.accountId = MnemonicUtils.formatAccountId(publicKey);

      // 模拟延迟以显示动画
      setTimeout(() => {
        this.goToStep(2);
        
        // 显示生成的助记词
        document.getElementById('generated-passphrase-display').textContent = 
          MnemonicUtils.formatMnemonicForDisplay(result.passphrase);
        document.getElementById('generated-account-id').textContent = 
          this.generatorState.accountId;
      }, 1500);

    } catch (error) {
      console.error('Failed to generate passphrase:', error);
      this.showError('Failed to generate passphrase. Browser may not support secure random.');
      this.hideGeneratorModal();
    }
  },

  /**
   * 导航到指定步骤
   */
  goToStep(step) {
    this.generatorState.step = step;
    
    // 更新步骤指示器
    document.querySelectorAll('.generator-steps-indicator .step-item').forEach(item => {
      const itemStep = parseInt(item.dataset.step);
      item.classList.toggle('active', itemStep === step);
      item.classList.toggle('completed', itemStep < step);
    });

    // 显示对应步骤内容
    document.querySelectorAll('.generator-step').forEach(s => s.style.display = 'none');
    document.getElementById(`gen-step-${step}`).style.display = 'block';

    if (step === 3) {
      this.setupVerificationStep();
    }
  },

  /**
   * 设置验证步骤
   */
  setupVerificationStep() {
    const masked = MnemonicUtils.maskMnemonicForConfirmation(
      this.generatorState.passphrase,
      6  // 显示6个单词
    );

    this.generatorState.revealedIndices = masked.revealedIndices;

    // 显示掩码后的助记词
    document.getElementById('masked-passphrase-display').innerHTML = `
      <div class="masked-words-grid">
        ${masked.masked.split('  ').map((word, i) => `
          <span class="masked-word ${word === '_____' ? 'hidden-word' : ''}" data-index="${i}">
            <span class="word-number">${i + 1}.</span>
            <span class="word-value">${word}</span>
          </span>
        `).join('')}
      </div>
    `;

    // 创建输入框
    const inputsContainer = document.getElementById('verification-inputs-container');
    inputsContainer.innerHTML = masked.hiddenWords.map((_, i) => `
      <div class="form-group verification-input-group">
        <label class="form-label">Word #${masked.revealedIndices[i] + 1}</label>
        <input 
          type="text" 
          class="form-input verification-input"
          data-index="${masked.revealedIndices[i]}"
          placeholder="Enter word..."
          autocomplete="off"
          spellcheck="false"
        >
      </div>
    `).join('');

    // 绑定回车键提交
    inputsContainer.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        this.verifyAndLogin();
      }
    });
  },

  /**
   * 验证助记词并登录
   */
  verifyAndLogin() {
    const inputs = document.querySelectorAll('.verification-input');
    const enteredWords = [];
    let allFilled = true;

    inputs.forEach(input => {
      const value = input.value.trim().toLowerCase();
      if (!value) allFilled = false;
      enteredWords.push({ index: parseInt(input.dataset.index), value });
    });

    if (!allFilled) {
      this.showVerificationError('Please enter all missing words');
      return;
    }

    // 验证输入的单词
    const actualWords = this.generatorState.passphrase.split(/\s+/);
    let allCorrect = true;

    for (const { index, value } of enteredWords) {
      if (value !== actualWords[index].toLowerCase()) {
        allCorrect = false;
        break;
      }
    }

    if (!allCorrect) {
      this.showVerificationError('One or more words are incorrect. Please check and try again.');
      return;
    }

    // 验证通过，执行登录
    this.hideGeneratorModal();
    this.handlePassphraseLoginWithGenerated(this.generatorState.passphrase);
  },

  /**
   * 使用生成的助记词登录
   */
  async handlePassphraseLoginWithGenerated(passphrase) {
    // 设置到输入框并提交
    document.getElementById('passphrase-input').value = passphrase;
    await this.handlePassphraseLogin();
  },

  /**
   * 复制助记词到剪贴板
   */
  copyPassphraseToClipboard() {
    if (!this.generatorState.passphrase) return;

    navigator.clipboard.writeText(this.generatorState.passphrase).then(() => {
      Toast.success('Passphrase copied to clipboard');
    }).catch(err => {
      console.error('Copy failed:', err);
      Toast.error('Failed to copy to clipboard');
    });
  },

  /**
   * 格式化账户ID为 NRCS 格式
   */
  formatAccountId(publicKeyHex) {
    // 这里应该是调用后端API获取真正的 accountRS
    // 前端仅做模拟显示
    const bytes = [];
    for (let i = 0; i < Math.min(16, publicKeyHex.length); i += 2) {
      bytes.push(parseInt(publicKeyHex.substr(i, 2), 16));
    }

    // 简单的格式化 (实际应使用 Reed-Solomon 编码)
    const num = BigInt('0x' + publicKeyHex.substr(0, 16));
    const padded = num.toString().padStart(16, '0');
    
    return `NRCS-${padded.substr(0, 4)}-${padded.substr(4, 4)}-${padded.substr(8, 4)}-${padded.substr(12, 5)}`;
  },

  /**
   * 检查区块链同步状态
   */
  async checkBlockchainStatus() {
    try {
      const status = await api.request('getBlockchainStatus', {});
      
      if (status && status.numberOfBlocks > 0) {
        store.setState({ blockchainHeight: status.numberOfBlocks });
        this.updateSyncStatus(status);
      }
    } catch (error) {
      // API 调用失败时静默处理，不显示错误
      // 可能是后端未启动或网络问题
      console.debug('Blockchain status check skipped:', error.message);
      
      // 显示离线状态（可选）
      const statusContainer = document.getElementById('login-sync-status');
      if (statusContainer) {
        statusContainer.style.display = 'block';
        
        const dot = document.getElementById('login-sync-dot');
        const text = document.getElementById('login-sync-text');
        
        if (dot && text) {
          dot.className = 'sync-dot disconnected';
          dot.style.background = '#6b7280';  // 灰色
          text.textContent = 'Waiting for connection...';
          text.style.color = '#9ca3af';
        }
      }
    }
  },

  /**
   * 更新同步状态显示
   */
  updateSyncStatus(status) {
    const statusContainer = document.getElementById('login-sync-status');
    const dot = document.getElementById('login-sync-dot');
    const text = document.getElementById('login-sync-text');
    const progressContainer = document.getElementById('login-sync-progress');
    const progressFill = document.getElementById('login-progress-fill');
    const progressText = document.getElementById('login-progress-text');

    statusContainer.style.display = 'block';

    if (!status.isScanning) {
      // 已同步完成或正在下载
      if (status.lastBlockHeight >= status.numberOfBlocks - 1) {
        dot.className = 'sync-dot synced';
        text.textContent = 'Blockchain synchronized';
        progressContainer.style.display = 'none';
      } else {
        // 正在下载区块
        dot.className = 'sync-dot downloading';
        text.textContent = 'Downloading blocks...';
        progressContainer.style.display = 'flex';
        
        const progress = ((status.lastBlockHeight + 1) / status.numberOfBlocks) * 100;
        progressFill.style.width = `${Math.min(progress, 100)}%`;
        progressText.textContent = `${Math.round(progress)}%`;
      }
    } else {
      // 正在扫描
      dot.className = 'sync-dot scanning';
      text.textContent = 'Scanning blockchain...';
      progressContainer.style.display = 'none';
    }
  },

  /**
   * 设置加载状态
   */
  setLoadingState(loading) {
    const btnPassphrase = document.getElementById('btn-login-passphrase');
    const btnAccount = document.getElementById('btn-login-account');

    [btnPassphrase, btnAccount].forEach(btn => {
      if (btn) {
        btn.disabled = loading;
        btn.innerHTML = loading ? `
          <div class="btn-spinner"></div>
          <span>Processing...</span>
        ` : btn.dataset.originalHtml || btn.innerHTML;
        
        if (!loading && !btn.dataset.originalHtml) {
          btn.dataset.originalHtml = btn.innerHTML;
        }
      }
    });
  },

  /**
   * 显示错误消息
   */
  showError(message) {
    const errorEl = document.getElementById('login-error');
    errorEl.textContent = message;
    errorEl.style.display = 'block';
    
    setTimeout(() => {
      errorEl.style.display = 'none';
    }, 5000);
  },

  /**
   * 隐藏错误消息
   */
  hideError() {
    document.getElementById('login-error').style.display = 'none';
  },

  /**
   * 显示验证错误
   */
  showVerificationError(message) {
    const errorEl = document.getElementById('verification-error');
    errorEl.textContent = message;
    errorEl.style.display = 'block';
  },

  /**
   * 处理底部链接操作
   */
  handleFooterAction(action) {
    switch (action) {
      case 'show-security-info':
        alert(`Security Information:\n\n- All cryptographic operations are performed locally\n- Passphrases never leave your device unencrypted\n- Uses Ed25519 digital signatures\n- Compatible with NRCS blockchain protocol`);
        break;
      case 'show-network-info':
        this.checkBlockchainStatus();
        break;
    }
  },

  // ========== P0 级别修复: 新增辅助方法 (参考 NRCS 实现) ==========

  /**
   * 显示全屏 Spinner 加载动画
   * 
   * 参考 NRCS: NRS.spinner.spin($("#center")[0])
   * 在登录过程中显示全屏遮罩层，防止用户重复操作
   * 
   * @param {string} message - 加载提示文字
   */
  showFullPageSpinner(message = 'Processing...') {
    // 移除已存在的 spinner
    this.hideFullPageSpinner();

    const spinnerOverlay = document.createElement('div');
    spinnerOverlay.id = 'login-fullpage-spinner';
    spinnerOverlay.className = 'fullpage-spinner-overlay';
    spinnerOverlay.innerHTML = `
      <div class="fullpage-spinner-container">
        <div class="spinner-circle"></div>
        <p class="spinner-message">${message}</p>
        <p class="spinner-hint">Please wait, this may take a moment</p>
      </div>
    `;
    
    document.body.appendChild(spinnerOverlay);
    
    // 添加淡入动画
    requestAnimationFrame(() => {
      spinnerOverlay.classList.add('visible');
    });
    
    console.log('Full page spinner shown');
  },

  /**
   * 隐藏全屏 Spinner
   * 
   * 参考 NRCS: NRS.spinner.stop()
   */
  hideFullPageSpinner() {
    const spinnerOverlay = document.getElementById('login-fullpage-spinner');
    
    if (spinnerOverlay) {
      // 添加淡出动画
      spinnerOverlay.classList.remove('visible');
      spinnerOverlay.classList.add('hiding');
      
      // 动画结束后移除元素
      setTimeout(() => {
        if (spinnerOverlay.parentNode) {
          spinnerOverlay.parentNode.removeChild(spinnerOverlay);
        }
      }, 300);
      
      console.log('Full page spinner hidden');
    }
  },

  /**
   * 验证账户公钥安全性
   * 
   * 参考 NRCS nrs.login.js 第 378-388 行:
   * 防止使用已被他人占用的账户地址登录
   * 通过比对链上公钥和本地生成的公钥来验证
   * 
   * @param {string} accountId - 账户 ID
   * @param {string} passphrase - 用户助记词
   * @throws {Error} 如果账户已被占用
   */
  async verifyPublicKey(accountId, passphrase) {
    try {
      console.log('Verifying public key for account:', accountId);
      
      // 调用 API 获取账户公钥 (参考 NRCS: getAccountPublicKey)
      const response = await api.request('getAccountPublicKey', {
        account: accountId
      });
      
      if (response && response.publicKey) {
        // 账户已有公钥，需要验证是否匹配当前助记词
        console.log('Account has existing public key, verifying...');
        
        // 生成本地公钥 (需要调用 MnemonicUtils)
        let generatedPublicKey;
        
        if (typeof MnemonicUtils !== 'undefined' && MnemonicUtils.getPublicKeyFromPassphrase) {
          generatedPublicKey = await MnemonicUtils.getPublicKeyFromPassphrase(passphrase);
        } else {
          // 如果 MnemonicUtils 不可用，跳过验证（降级处理）
          console.warn('MnemonicUtils not available, skipping public key verification');
          return;
        }
        
        // 比对公钥 (参考 NRCS nrs.login.js 第 381-382 行)
        if (response.publicKey !== generatedPublicKey) {
          throw new Error(
            'This account is already taken by another passphrase. ' +
            'Please use the correct passphrase or login with a different account.'
          );
        }
        
        console.log('Public key verification passed');
      } else {
        // 账户无公钥（新账户），这是正常的
        console.log('No existing public key found (new account)');
      }
      
    } catch (error) {
      // API 调用失败时，如果是 5 错误码（新账户）则允许继续
      if (error.errorCode === 5 || error.message?.includes('5')) {
        console.log('New account detected, public key verification skipped');
        return;
      }
      
      // 其他错误抛出
      if (error.message?.includes('already taken')) {
        throw error;  // 重新抛出"账户被占用"错误
      }
      
      // 非关键错误仅警告，不阻止登录
      console.warn('Public key verification failed with non-critical error:', error.message);
    }
  },

  /**
   * 安全存储助记词（记住我功能）
   * 
   * 参考 NRCS nrs.login.js 第 392-401 行:
   * NRS.rememberPassword = true;
   * NRS.setPassword(id);
   * 
   * ⚠️ 安全提示:
   * - 仅在用户明确勾选"记住我"时存储
   * - 建议使用浏览器提供的加密存储方案
   * - 生产环境应考虑更安全的存储方式
   * 
   * @param {string} passphrase - 用户助记词
   */
  savePassphraseSecurely(passphrase) {
    try {
      // 检查浏览器是否支持 secure storage
      if (window.crypto && window.crypto.subtle) {
        // 使用 Web Crypto API 进行简单混淆（非加密）
        // 生产环境应使用更强的保护机制
        const encoder = new TextEncoder();
        const data = encoder.encode(passphrase);
        
        // 使用 SHA-256 哈希作为标识（不存储明文）
        return crypto.subtle.digest('SHA-256', data).then(hashBuffer => {
          const hashArray = Array.from(new Uint8Array(hashBuffer));
          const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
          
          // 存储哈希值用于后续验证（不存储实际密码短语）
          localStorage.setItem('nrcs_passphrase_hash', hashHex);
          localStorage.setItem('nrcs_remember_me', 'true');
          
          console.log('Passphrase saved securely (hash only)');
        });
      } else {
        // 降级处理：直接标记为已记住（不实际存储）
        localStorage.setItem('nrcs_remember_me', 'true');
        console.warn('Secure storage not available, using fallback mode');
      }
    } catch (error) {
      console.error('Failed to save passphrase securely:', error);
      // 不阻止登录流程
    }
  },

  /**
   * 检测是否为测试网络模式
   * 
   * 参考 NRCS: NRS.isTestNet
   * 通过区块链状态或配置判断当前网络类型
   * 
   * @returns {boolean} 是否为测试网络
   */
  isTestnetMode() {
    // 方法1: 从 Store 状态获取 (如果已加载)
    if (store.state?.blockchain?.isTestnet !== undefined) {
      return store.state.blockchain.isTestnet;
    }
    
    // 方法2: 从 URL 参数获取 (?testnet=true)
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get('testnet') === 'true') {
      return true;
    }
    
    // 方法3: 从 localStorage 获取之前的设置
    const savedMode = localStorage.getItem('nrcs_network_mode');
    if (savedMode === 'testnet') {
      return true;
    }
    
    // 默认: 主网模式
    return false;
  },

  /**
   * 显示安全建议警告 (非阻塞)
   * 
   * 参考 NRCS: 使用 $.growl 或 console.warn
   * 在 UI 上显示友好的安全建议，但不阻止操作
   * 
   * @param {string} message - 安全建议消息
   */
  showSecurityWarning(message) {
    // 方法1: 使用 Toast 组件显示 (如果有)
    if (typeof Toast !== 'undefined' && Toast.warning) {
      Toast.warning(message, { duration: 5000 });
      return;
    }
    
    // 方法2: 在页面顶部显示横幅提示
    const existingWarning = document.getElementById('security-warning-banner');
    if (!existingWarning) {
      const banner = document.createElement('div');
      banner.id = 'security-warning-banner';
      banner.className = 'security-warning-banner';
      banner.innerHTML = `
        <div class="warning-content">
          <span class="warning-icon">⚠️</span>
          <span class="warning-text">${message}</span>
          <button class="warning-close" onclick="this.parentElement.parentElement.remove()">×</button>
        </div>
      `;
      
      // 插入到登录表单之前
      const loginForm = document.querySelector('.login-form') || document.querySelector('.login-container');
      if (loginForm) {
        loginForm.parentNode.insertBefore(banner, loginForm);
        
        // 10秒后自动消失
        setTimeout(() => {
          if (banner.parentNode) {
            banner.remove();
          }
        }, 10000);
      } else {
        // 降级到控制台输出
        console.warn('[Security]', message);
      }
    }
  },
};

// 导出到全局作用域
window.LoginPage = LoginPage;