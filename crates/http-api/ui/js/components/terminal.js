/**
 * NRCS Wallet - Terminal Console Component
 * 
 * 终端控制台组件，提供命令行交互功能
 * 
 * 支持的内置命令:
 * - clear: 清除终端内容
 * - save: 保存终端内容到文件
 * - history: 显示命令历史记录
 * - help: 显示帮助信息
 * - [其他]: 传递给自定义处理器
 */

const TerminalConsole = {
  // 终端容器元素
  container: null,
  
  // 输出区域
  outputArea: null,
  
  // 输入区域
  inputArea: null,
  
  // 命令历史
  commandHistory: [],
  
  // 历史记录索引（用于上下键导航）
  historyIndex: -1,
  
  // 当前输入的命令（用于历史导航）
  currentInput: '',
  
  // 最大历史记录数
  maxHistorySize: 100,
  
  // 是否已初始化
  isInitialized: false,
  
  // 自定义命令处理器
  customHandlers: new Map(),

  /**
   * 初始化终端控制台
   * @param {string} containerId - 容器元素的 ID
   */
  init(containerId = 'terminal-container') {
    const container = document.getElementById(containerId);
    
    if (!container) {
      console.error('Terminal container not found:', containerId);
      return;
    }
    
    this.container = container;
    this.renderTerminal();
    this.bindEvents();
    this.isInitialized = true;
    
    console.log('Terminal console initialized');
  },

  /**
   * 渲染终端界面
   */
  renderTerminal() {
    this.container.innerHTML = `
      <div class="terminal-wrapper">
        <!-- 终端头部 -->
        <div class="terminal-header">
          <div class="terminal-title">
            <span class="terminal-icon">⌨️</span>
            <span>NRCS Console</span>
          </div>
          <div class="terminal-actions">
            <button class="terminal-btn" id="terminal-clear-btn" title="Clear (Ctrl+L)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
            <button class="terminal-btn" id="terminal-save-btn" title="Save (Ctrl+S)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
                <polyline points="17 21 17 13 7 13 7 21"/>
                <polyline points="7 3 7 8 15 8"/>
              </svg>
            </button>
            <button class="terminal-btn" id="terminal-history-btn" title="History (Ctrl+H)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
            </button>
          </div>
        </div>
        
        <!-- 输出区域 -->
        <div class="terminal-output" id="terminal-output">
          <div class="terminal-line welcome">
            <span class="output-text">Welcome to NRCS Wallet Console v1.0.0</span>
          </div>
          <div class="terminal-line help-text">
            <span class="output-text">Type 'help' for available commands</span>
          </div>
        </div>
        
        <!-- 输入区域 -->
        <div class="terminal-input-line">
          <span class="terminal-prompt">❯</span>
          <input 
            type="text" 
            class="terminal-input" 
            id="terminal-input"
            placeholder="Enter command..."
            autocomplete="off"
            spellcheck="false"
          >
        </div>
      </div>
    `;
    
    this.outputArea = document.getElementById('terminal-output');
    this.inputArea = document.getElementById('terminal-input');
  },

  /**
   * 绑定事件监听器
   */
  bindEvents() {
    // 输入框事件
    if (this.inputArea) {
      // 回车键执行命令
      this.inputArea.addEventListener('keydown', (e) => this.handleKeyDown(e));
      
      // 自动聚焦
      this.container.addEventListener('click', () => {
        this.inputArea.focus();
      });
    }
    
    // 头部按钮事件
    const clearBtn = document.getElementById('terminal-clear-btn');
    const saveBtn = document.getElementById('terminal-save-btn');
    const historyBtn = document.getElementById('terminal-history-btn');
    
    if (clearBtn) clearBtn.addEventListener('click', () => this.executeCommand('clear'));
    if (saveBtn) saveBtn.addEventListener('click', () => this.executeCommand('save'));
    if (historyBtn) historyBtn.addEventListener('click', () => this.executeCommand('history'));
  },

  /**
   * 处理键盘事件
   */
  handleKeyDown(event) {
    const key = event.key;
    
    switch (key) {
      case 'Enter':
        event.preventDefault();
        this.executeCommand(this.inputArea.value.trim());
        break;
        
      case 'ArrowUp':
        event.preventDefault();
        this.navigateHistory(-1);
        break;
        
      case 'ArrowDown':
        event.preventDefault();
        this.navigateHistory(1);
        break;
        
      case 'Tab':
        event.preventDefault();
        this.autoComplete();
        break;
        
      case 'l':
        if (event.ctrlKey || event.metaKey) {
          event.preventDefault();
          this.executeCommand('clear');
        }
        break;
        
      case 's':
        if (event.ctrlKey || event.metaKey) {
          event.preventDefault();
          this.executeCommand('save');
        }
        break;
        
      case 'h':
        if (event.ctrlKey || event.metaKey) {
          event.preventDefault();
          this.executeCommand('history');
        }
        break;
    }
  },

  /**
   * 执行命令
   * @param {string} command - 命令字符串
   */
  executeCommand(command) {
    if (!command) return;
    
    // 显示用户输入
    this.printLine(`❯ ${command}`, 'input');
    
    // 解析命令和参数
    const parts = command.split(/\s+/);
    const cmd = parts[0].toLowerCase();
    const args = parts.slice(1);
    
    // 添加到历史记录
    this.addToHistory(command);
    
    // 处理内置命令
    if (this.builtInCommands[cmd]) {
      this.builtInCommands[cmd](args);
    } else if (this.customHandlers.has(cmd)) {
      // 自定义命令处理器
      this.customHandlers.get(cmd)(args, command);
    } else {
      this.printLine(`Command not found: ${cmd}. Type 'help' for available commands.`, 'error');
    }
    
    // 清空输入框并重置历史索引
    this.inputArea.value = '';
    this.historyIndex = -1;
    this.currentInput = '';
    
    // 滚动到底部
    this.scrollToBottom();
  },

  /**
   * 内置命令集合
   */
  builtInCommands: {
    /**
     * 清除终端内容
     */
    clear: () => {
      if (this.outputArea) {
        this.outputArea.innerHTML = `
          <div class="terminal-line system">
            <span class="output-text">Terminal cleared [${new Date().toLocaleTimeString()}]</span>
          </div>
        `;
      }
      console.log('Terminal cleared');
    },
    
    /**
     * 保存终端内容到文件
     */
    save: async () => {
      try {
        const content = this.getOutputText();
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const filename = `nrcs-terminal-${timestamp}.txt`;
        
        // 创建 Blob 并下载
        const blob = new Blob([content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        this.printLine(`✅ Terminal output saved to: ${filename}`, 'success');
        this.printLine(`📄 Size: ${(content.length / 1024).toFixed(2)} KB | Lines: ${content.split('\n').length}`, 'info');
        
      } catch (error) {
        this.printLine(`❌ Save failed: ${error.message}`, 'error');
      }
    },
    
    /**
     * 显示命令历史
     */
    history: (args) => {
      const limit = args[0] ? parseInt(args[0]) : this.commandHistory.length;
      
      if (this.commandHistory.length === 0) {
        this.printLine('No commands in history.', 'info');
        return;
      }
      
      this.printLine(`\n📜 Command History (${Math.min(limit, this.commandHistory.length)} most recent):\n`, 'header');
      
      const startIdx = Math.max(0, this.commandHistory.length - limit);
      const historyToShow = this.commandHistory.slice(startIdx);
      
      historyToShow.forEach((cmd, idx) => {
        const num = (startIdx + idx + 1).toString().padStart(4, ' ');
        this.printLine(`  ${num}  ${cmd}`, 'history-item');
      });
      
      this.printLine('', 'empty');
    },
    
    /**
     * 显示帮助信息
     */
    help: () => {
      const helpText = `
╔══════════════════════════════════════════════════╗
║           NRCS Wallet Console Help               ║
╠══════════════════════════════════════════════════╣
║  Built-in Commands:                              ║
║  ─────────────────                              ║
║  clear                 Clear terminal output     ║
║  save                  Save output to file       ║
║  history [n]           Show last n commands      ║
║  help                  Show this help message    ║
║                                                   ║
║  Keyboard Shortcuts:                              ║
║  ────────────────────                            ║
║  Enter                 Execute command           ║
║  ↑ / ↓                Navigate history          ║
║  Tab                   Auto-complete             ║
║  Ctrl+L / Cmd+L        Clear terminal            ║
║  Ctrl+S / Cmd+S        Save to file              ║
║  Ctrl+H / Cmd+H        Show history              ║
╚══════════════════════════════════════════════════╝`;
      
      this.printLine(helpText, 'help');
    },
  },

  /**
   * 在终端中打印一行文本
   * @param {string} text - 文本内容
   * @param {string} type - 类型 (input/output/error/success/info/header)
   */
  printLine(text, type = 'output') {
    if (!this.outputArea) return;
    
    const line = document.createElement('div');
    line.className = `terminal-line ${type}`;
    
    const span = document.createElement('span');
    span.className = 'output-text';
    span.textContent = text;
    
    line.appendChild(span);
    this.outputArea.appendChild(line);
  },

  /**
   * 获取所有输出文本
   * @returns {string}
   */
  getOutputText() {
    if (!this.outputArea) return '';
    
    const lines = this.outputArea.querySelectorAll('.output-text');
    return Array.from(lines)
      .map(span => span.textContent)
      .join('\n');
  },

  /**
   * 添加命令到历史记录
   * @param {string} command - 命令文本
   */
  addToHistory(command) {
    // 避免重复添加相同的连续命令
    if (this.commandHistory[this.commandHistory.length - 1] !== command) {
      this.commandHistory.push(command);
      
      // 限制历史记录大小
      if (this.commandHistory.length > this.maxHistorySize) {
        this.commandHistory.shift();
      }
    }
  },

  /**
   * 导航命令历史
   * @param {number} direction - 方向 (-1: 向上/更旧, 1: 向下/更新)
   */
  navigateHistory(direction) {
    if (this.commandHistory.length === 0) return;
    
    // 第一次按上键时保存当前输入
    if (this.historyIndex === -1 && direction === -1) {
      this.currentInput = this.inputArea.value;
    }
    
    // 更新索引
    this.historyIndex += direction;
    
    // 边界检查
    if (this.historyIndex < 0) {
      this.historyIndex = -1;
      this.inputArea.value = this.currentInput;
      return;
    }
    
    if (this.historyIndex >= this.commandHistory.length) {
      this.historyIndex = this.commandHistory.length - 1;
    }
    
    // 显示历史命令
    this.inputArea.value = this.commandHistory[this.historyIndex];
  },

  /**
   * 自动补全（简单实现）
   */
  autoComplete() {
    const input = this.inputArea.value.toLowerCase();
    if (!input) return;
    
    // 收集所有可用命令
    const allCommands = [
      ...Object.keys(this.builtInCommands),
      ...Array.from(this.customHandlers.keys()),
    ];
    
    // 查找匹配的命令
    const matches = allCommands.filter(cmd => cmd.startsWith(input));
    
    if (matches.length === 1) {
      this.inputArea.value = matches[0];
    } else if (matches.length > 1) {
      this.printLine(`\nPossible completions: ${matches.join(', ')}`, 'info');
    }
  },

  /**
   * 注册自定义命令处理器
   * @param {string} command - 命令名称
   * @param {Function} handler - 处理函数 (args, fullCommand) => void
   */
  registerCommand(command, handler) {
    this.customHandlers.set(command.toLowerCase(), handler);
  },

  /**
   * 注销自定义命令
   * @param {string} command - 命令名称
   */
  unregisterCommand(command) {
    this.customHandlers.delete(command.toLowerCase());
  },

  /**
   * 滚动到底部
   */
  scrollToBottom() {
    if (this.outputArea) {
      this.outputArea.scrollTop = this.outputArea.scrollHeight;
    }
  },

  /**
   * 聚焦到输入框
   */
  focus() {
    if (this.inputArea) {
      this.inputArea.focus();
    }
  },

  /**
   * 销毁终端实例
   */
  destroy() {
    if (this.container) {
      this.container.innerHTML = '';
    }
    this.isInitialized = false;
    this.commandHistory = [];
    this.customHandlers.clear();
  },
};

// 导出到全局作用域
window.TerminalConsole = TerminalConsole;