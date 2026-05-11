/**
 * NRCS Wallet - Formatters
 * 
 * 数据格式化工具函数
 * 包含: 数字格式化、时间格式化、地址截断等
 */

const Formatters = {
  /**
   * 格式化 NRC 数量 (8位小数)
   * @param {number|string} value - 原始值 (NQT)
   * @param {number} decimals - 显示的小数位数
   * @returns {string} 格式化后的字符串
   */
  formatNrc(value, decimals = 2) {
    const num = typeof value === 'string' ? parseFloat(value) : value;
    if (!Number.isFinite(num)) return '0.00';
    
    // NRC 使用 8 位小数
    const nrcValue = num / 1e8;
    return nrcValue.toFixed(decimals);
  },

  /**
   * 格式化数字为千分位
   * @param {number} num - 数字
   * @returns {string}
   */
  formatNumber(num) {
    if (!Number.isFinite(num)) return '0';
    
    if (Math.abs(num) >= 1e9) {
      return (num / 1e9).toFixed(2) + 'B';
    } else if (Math.abs(num) >= 1e6) {
      return (num / 1e6).toFixed(2) + 'M';
    } else if (Math.abs(num) >= 1e3) {
      return num.toLocaleString('en-US', { maximumFractionDigits: 2 });
    }
    
    return num.toFixed(2);
  },

  /**
   * 截断账户地址
   * @param {string} address - 完整地址
   * @param {number} startLength - 开头保留长度
   * @param {number} endLength - 结尾保留长度
   * @returns {string} 截断后的地址
   */
  truncateAddress(address, startLength = 10, endLength = 4) {
    if (!address || address.length <= startLength + endLength + 3) {
      return address || '-';
    }
    return `${address.slice(0, startLength)}...${address.slice(-endLength)}`;
  },

  /**
   * 格式化 Unix 时间戳为可读日期
   * @param {number} timestamp - Unix 时间戳 (秒)
   * @param {boolean} includeTime - 是否包含时间
   * @returns {string} 格式化后的日期字符串
   */
  formatDate(timestamp, includeTime = true) {
    if (!timestamp || !Number.isFinite(timestamp)) return '-';

    const date = new Date(parseInt(timestamp) + 145000000000); // NRCS epoch offset
    
    const options = {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    };

    if (includeTime) {
      options.hour = '2-digit';
      options.minute = '2-digit';
      options.second = '2-digit';
    }

    try {
      return date.toLocaleDateString('en-US', options);
    } catch (e) {
      return '-';
    }
  },

  /**
   * 格式化为相对时间 (X minutes ago)
   * @param {number} timestamp - Unix 时间戳
   * @returns {string}
   */
  formatRelativeTime(timestamp) {
    if (!timestamp || !Number.isFinite(timestamp)) return '';

    const now = Date.now();
    const date = new Date(parseInt(timestamp) + 145000000000); // NRCS epoch offset
    const diffMs = now - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffDays > 7) {
      return this.formatDate(timestamp, false);
    } else if (diffDays > 0) {
      return `${diffDays}d ago`;
    } else if (diffHours > 0) {
      return `${diffHours}h ago`;
    } else if (diffMins > 0) {
      return `${diffMins}m ago`;
    } else if (diffSecs > 0) {
      return `${diffSecs}s ago`;
    } else {
      return 'just now';
    }
  },

  /**
   * 格式化交易类型 ID 为可读名称
   * @param {number} typeId - 交易类型 ID
   * @returns {string}
   */
  formatTransactionType(typeId) {
    const types = {
      0: 'Payment',
      1: 'Messaging',
      2: 'Colored Coins',
      3: 'Digital Goods',
      4: 'Account Info',
      5: 'Alias Assignment',
      6: 'Leasing',
      7: 'Phasing',
    };
    return types[typeId] || `Type ${typeId}`;
  },

  /**
   * 获取交易类型对应的图标类名
   * @param {number} typeId - 交易类型 ID
   * @returns {string} 图标名称
   */
  getTransactionIcon(typeId) {
    const icons = {
      0: 'payment',
      1: 'message',
      2: 'asset',
      3: 'marketplace',
      4: 'info',
      5: 'alias',
      6: 'lease',
      7: 'phasing',
    };
    return icons[typeId] || 'unknown';
  },

  /**
   * 获取交易类型对应的颜色类
   * @param {number} typeId - 交易类型 ID
   * @returns {string} 颜色类名
   */
  getTransactionColor(typeId) {
    const colors = {
      0: 'primary',     // Payment - 青蓝
      1: 'secondary',   // Messaging - 紫罗兰
      2: 'success',     // Assets - 绿色
      3: 'warning',     // Marketplace - 橙色
      4: 'info',        // Account Info - 蓝色
      5: 'secondary',   // Alias - 紫罗兰
      6: 'success',     // Leasing - 绿色
      7: 'warning',     // Phasing - 橙色
    };
    return colors[typeId] || 'muted';
  },

  /**
   * 格式化字节大小
   * @param {number} bytes - 字节数
   * @returns {string}
   */
  formatBytes(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const size = bytes / Math.pow(1024, i);
    
    return `${size.toFixed(2)} ${units[i]}`;
  },

  /**
   * 格式化延迟时间
   * @param {number} ms - 毫秒数
   * @returns {string}
   */
  formatLatency(ms) {
    if (!ms || ms < 0) return '-';
    
    if (ms < 1000) {
      return `${Math.round(ms)}ms`;
    } else {
      return `${(ms / 1000).toFixed(2)}s`;
    }
  },

  /**
   * 格式化难度值
   * @param {string|Array} difficulty - 难度值 (可能是十六进制字符串或字节数组)
   * @returns {string}
   */
  formatDifficulty(difficulty) {
    if (!difficulty) return '-';

    // 如果是十六进制字符串
    if (typeof difficulty === 'string') {
      try {
        const bigInt = BigInt('0x' + difficulty);
        return this.formatNumber(bigInt.toString());
      } catch (e) {
        return difficulty;
      }
    }

    // 如果是数组
    if (Array.isArray(difficulty)) {
      return `[${difficulty.length} bytes]`;
    }

    return String(difficulty);
  },

  /**
   * 格式化百分比
   * @param {number} value - 值
   * @param {number} total - 总值
   * @param {number} decimals - 小数位数
   * @returns {string}
   */
  formatPercentage(value, total, decimals = 1) {
    if (!total || total === 0) return '0%';
    
    const percentage = (value / total) * 100;
    return `${percentage.toFixed(decimals)}%`;
  },

  /**
   * 安全地解析 JSON 字符串
   * @param {string} str - JSON 字符串
   * @param {*} fallback - 解析失败时的默认返回值
   * @returns {*}
   */
  safeJsonParse(str, fallback = null) {
    try {
      return JSON.parse(str);
    } catch (e) {
      return fallback;
    }
  },
};

// 导出供其他模块使用
window.Formatters = Formatters;
