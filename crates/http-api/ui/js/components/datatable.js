/**
 * NRCS Wallet - DataTable Component
 * 
 * P2-UI-1: 轻量级 DataTable 组件
 * 参考 Bootstrap DataTable 和 NRCS 表格实现
 * 
 * 功能特性:
 * - 服务端分页 (Server-side Pagination)
 * - 列排序 (Column Sorting)
 * - 行内编辑支持
 * - 响应式布局
 * - 无障碍访问支持
 */

class NrcsDataTable {
  /**
   * 构造函数
   * 
   * @param {string|HTMLElement} container - 容器元素或选择器
   * @param {Object} options - 配置选项
   */
  constructor(container, options = {}) {
    this.container = typeof container === 'string' 
      ? document.querySelector(container) 
      : container;
    
    if (!this.container) {
      throw new Error('DataTable container not found');
    }
    
    // 配置项
    this.options = {
      columns: [],           // 列配置 [{key, title, sortable, render, width}]
      data: [],              // 数据数组 (客户端模式) 或 null (服务端模式)
      serverSide: false,     // 是否使用服务端分页
      pageSize: 50,          // 每页行数 (参考 NRCS 默认值)
      showPagination: true,   // 显示分页控件
      showPageNumbers: true,  // 显示页码数字 (参考 NRCS: NRS.showPageNumbers)
      sortable: true,        // 允许排序
      selectable: true,       // 允许行选择
      emptyMessage: 'No data available',  // 空数据提示
      loadingMessage: 'Loading...',       // 加载提示
      onRowClick: null,       // 行点击回调
      onPageChange: null,      // 页码变更回调
      onSortChange: null,      // 排序变更回调
      ...options,
    };
    
    // 状态
    this.currentPage = 1;
    this.totalItems = 0;
    this.totalPages = 0;
    this.sortColumn = null;
    this.sortDirection = 'asc';  // 'asc' | 'desc'
    this.isLoading = false;
    this.selectedRows = new Set();
    
    // 初始化
    this.init();
  }

  /**
   * 初始化组件
   */
  init() {
    this.render();
    this.bindEvents();
    
    console.debug('[DataTable] Initialized with', this.options.columns.length, 'columns');
  }

  /**
   * 渲染表格
   */
  render() {
    const { columns, serverSide } = this.options;
    
    this.container.innerHTML = `
      <div class="nrcs-datatable">
        <div class="dt-header">
          <div class="dt-toolbar">
            ${this.renderToolbar()}
          </div>
        </div>
        
        <div class="dt-body">
          <table class="dt-table" role="grid" aria-label="Data table">
            <thead>
              <tr>
                ${columns.map((col, idx) => this.renderHeaderCell(col, idx)).join('')}
              </tr>
            </thead>
            <tbody>
              <tr class="dt-loading-row" aria-live="polite">
                <td colspan="${columns.length}" class="dt-loading">
                  <span class="loading-dots"><span>.</span><span>.</span><span>.</span></span>
                  ${this.options.loadingMessage}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        
        ${this.options.showPagination ? this.renderPagination() : ''}
      </div>
    `;
    
    // 如果是客户端模式且有数据，立即加载
    if (!serverSide && this.options.data && this.options.data.length > 0) {
      this.loadLocalData(this.options.data);
    }
  }

  /**
   * 渲染表头单元格
   */
  renderHeaderCell(column, index) {
    const { key, title, sortable, width } = column;
    const isSortable = this.options.sortable && (sortable !== false);
    const sortClass = this.sortColumn === key 
      ? `sorted-${this.sortDirection}` 
      : '';
    
    return `
      <th 
        class="dt-th ${isSortable ? 'sortable' : ''} ${sortClass}"
        style="${width ? `width: ${width};` : ''}"
        data-column="${key}"
        role="columnheader"
        aria-sort="${this.getAriaSort(key)}"
        tabindex="${isSortable ? 0 : -1}"
      >
        <span class="dt-th-content">${title}</span>
        ${isSortable ? '<span class="dt-sort-icon" aria-hidden="true"></span>' : ''}
      </th>
    `;
  }

  /**
   * 获取 ARIA 排序属性值
   */
  getAriaSort(columnKey) {
    if (this.sortColumn !== columnKey) return 'none';
    return this.sortDirection === 'asc' ? 'ascending' : 'descending';
  }

  /**
   * 渲染工具栏
   */
  renderToolbar() {
    return `
      <div class="dt-info" aria-live="polite">
        <span class="dt-record-count">0 records</span>
      </div>
      
      <div class="dt-actions">
        ${this.options.selectable ? `
          <label class="dt-select-all">
            <input type="checkbox" 
                   id="dt-select-all" 
                   aria-label="Select all rows"
                   tabindex="0">
          </label>
        ` : ''}
      </div>
    `;
  }

  /**
   * 渲染分页控件
   * 
   * 参考 NRCS 分页样式:
   * - 上一页/下一页按钮
   * - 页码数字 (可选)
   * - 总记录数显示
   */
  renderPagination() {
    return `
      <div class="dt-pagination" role="navigation" aria-label="Table pagination">
        <button class="dt-page-btn dt-prev" 
                disabled="${this.currentPage <= 1}" 
                aria-label="Previous page"
                tabindex="0">
          ‹ Previous
        </button>
        
        <div class="dt-pages" aria-label="Page numbers">
          ${this.renderPageNumbers()}
        </div>
        
        <button class="dt-page-btn dt-next"
                disabled="${this.currentPage >= this.totalPages || this.totalPages === 0}" 
                aria-label="Next page"
                tabindex="0">
          Next ›
        </button>
        
        <div class="dt-page-size">
          <select id="dt-page-size" aria-label="Rows per page">
            <option value="10">10</option>
            <option value="20">20</option>
            <option value="50" selected>50</option>
            <option value="100">100</option>
          </select>
          rows per page
        </div>
      </div>
    `;
  }

  /**
   * 渲染页码数字
   */
  renderPageNumbers() {
    if (!this.options.showPageNumbers || this.totalPages === 0) {
      return `<span class="dt-current-page">Page ${this.currentPage}</span>`;
    }
    
    let pages = [];
    const maxVisible = 7;  // 最大显示页码数
    
    if (this.totalPages <= maxVisible) {
      pages = Array.from({ length: this.totalPages }, (_, i) => i + 1);
    } else {
      // 省略号算法
      const start = Math.max(1, this.currentPage - 3);
      const end = Math.min(this.totalPages, start + maxVisible - 1);
      
      if (start > 1) pages.push(1, '...');
      for (let i = start; i <= end; i++) pages.push(i);
      if (end < this.totalPages) pages.push('...', this.totalPages);
    }
    
    return pages.map(page => {
      if (page === '...') {
        return `<span class="dt-ellipsis" aria-hidden="true">...</span>`;
      }
      
      return `
        <button class="dt-page-num ${page === this.currentPage ? 'active' : ''}"
                data-page="${page}"
                aria-label="Go to page ${page}"
                aria-current="${page === this.currentPage ? 'page' : 'false'}"
                tabindex="0">
          ${page}
        </button>
      `;
    }).join('');
  }

  /**
   * 绑定事件处理
   */
  bindEvents() {
    const dt = this.container.querySelector('.nrcs-datatable');
    
    // 排序点击
    dt.querySelectorAll('.dt-th.sortable').forEach(th => {
      th.addEventListener('click', () => this.handleSort(th));
      th.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          this.handleSort(th);
        }
      });
    });
    
    // 分页按钮
    dt.addEventListener('click', (e) => this.handlePaginationClick(e));
    
    // 每页数量变更
    const pageSizeSelect = dt.querySelector('#dt-page-size');
    if (pageSizeSelect) {
      pageSizeSelect.addEventListener('change', (e) => {
        this.options.pageSize = parseInt(e.target.value);
        this.currentPage = 1;
        this.loadData();
      });
    }
    
    // 全选复选框
    const selectAllCheckbox = dt.querySelector('#dt-select-all');
    if (selectAllCheckbox) {
      selectAllCheckbox.addEventListener('change', () => this.toggleAllSelection());
    }
  }

  /**
   * 处理排序
   */
  handleSort(headerCell) {
    const columnKey = headerCell.dataset.column;
    
    if (this.sortColumn === columnKey) {
      // 切换排序方向
      this.sortDirection = this.sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      // 新列排序
      this.sortColumn = columnKey;
      this.sortDirection = 'asc';
    }
    
    this.currentPage = 1;  // 重置到第一页
    
    // 更新 UI
    this.updateSortUI();
    
    // 触发回调
    if (this.options.onSortChange) {
      this.options.onSortChange(this.sortColumn, this.sortDirection);
    }
    
    // 重新加载数据
    this.loadData();
  }

  /**
   * 更新排序 UI 状态
   */
  updateSortUI() {
    this.container.querySelectorAll('.dt-th').forEach(th => {
      th.classList.remove('sorted-asc', 'sorted-desc');
      th.setAttribute('aria-sort', 'none');
    });
    
    if (this.sortColumn) {
      const activeTh = this.container.querySelector(`.dt-th[data-column="${this.sortColumn}"]`);
      if (activeTh) {
        activeTh.classList.add(`sorted-${this.sortDirection}`);
        activeTh.setAttribute('aria-sort', this.getAriaSort(this.sortColumn));
      }
    }
  }

  /**
   * 处理分页点击
   */
  handlePaginationClick(event) {
    const target = event.target;
    
    // 上一页/下一页
    if (target.classList.contains('dt-prev')) {
      if (this.currentPage > 1) {
        this.goToPage(this.currentPage - 1);
      }
      return;
    }
    
    if (target.classList.contains('dt-next')) {
      if (this.currentPage < this.totalPages) {
        this.goToPage(this.currentPage + 1);
      }
      return;
    }
    
    // 页码数字
    if (target.classList.contains('dt-page-num')) {
      const page = parseInt(target.dataset.page);
      if (page && page !== this.currentPage) {
        this.goToPage(page);
      }
      return;
    }
  }

  /**
   * 跳转到指定页
   * 
   * @param {number} page - 目标页码
   */
  goToPage(page) {
    if (page < 1 || page > this.totalPages) return;
    
    this.currentPage = page;
    
    // 更新 Store 的分页状态 (P1-R3 集成)
    if (typeof store !== 'undefined' && store.setPageNumber) {
      store.setPageNumber(page);
    }
    
    // 更新 UI
    this.updatePaginationUI();
    
    // 触发回调
    if (this.options.onPageChange) {
      this.options.onPageChange(page);
    }
    
    // 加载新数据
    this.loadData();
  }

  /**
   * 更新分页 UI
   */
  updatePaginationUI() {
    // 更新页码激活状态
    this.container.querySelectorAll('.dt-page-num').forEach(btn => {
      btn.classList.toggle('active', parseInt(btn.dataset.page) === this.currentPage);
      btn.setAttribute('aria-current', parseInt(btn.dataset.page) === this.currentPage ? 'page' : 'false');
    });
    
    // 更新上一页/下一页按钮状态
    this.container.querySelector('.dt-prev').disabled = this.currentPage <= 1;
    this.container.querySelector('.dt-next').disabled = this.currentPage >= this.totalPages || this.totalPages === 0;
    
    // 更新当前页显示
    const currentPageEl = this.container.querySelector('.dt-current-page');
    if (currentPageEl) {
      currentPageEl.textContent = `Page ${this.currentPage}`;
    }
  }

  /**
   * 加载数据 (根据模式自动选择)
   */
  async loadData() {
    if (this.isLoading) return;
    
    this.isLoading = true;
    this.showLoadingState();
    
    try {
      if (this.options.serverSide) {
        // 服务端模式: 调用 API
        await this.loadServerData();
      } else {
        // 客户端模式: 从本地数据切片
        this.loadLocalData(this.options.data || []);
      }
    } catch (error) {
      console.error('[DataTable] Load error:', error);
      this.showErrorState(error.message);
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * 加载本地数据 (客户端分页)
   */
  loadLocalData(data) {
    const { columns, pageSize } = this.options;
    
    this.totalItems = data.length;
    this.totalPages = Math.ceil(this.totalItems / pageSize) || 1;
    
    // 排序数据
    let sortedData = [...data];
    if (this.sortColumn) {
      sortedData.sort((a, b) => {
        const aVal = a[this.sortColumn];
        const bVal = b[this.sortColumn];
        const cmp = String(aVal).localeCompare(String(bVal), undefined, { numeric: true });
        return this.sortDirection === 'asc' ? cmp : -cmp;
      });
    }
    
    // 分页切片
    const startIndex = (this.currentPage - 1) * pageSize;
    const endIndex = startIndex + pageSize;
    const pageData = sortedData.slice(startIndex, endIndex);
    
    // 渲染表格体
    this.renderTableBody(pageData);
    
    // 更新信息
    this.updateInfo(startIndex + 1, Math.min(endIndex, this.totalItems));
    
    // 更新分页 UI
    this.updatePaginationUI();
  }

  /**
   * 加载服务端数据 (异步 API 调用)
   */
  async loadServerData() {
    // 这里应该调用实际的 API
    // 示例:
    // const result = await api.getBlockchainTransactions(accountId, firstIndex, lastIndex);
    // this.renderTableBody(result.transactions);
    
    console.warn('[DataTable] Server-side loading not implemented. Please override loadServerData().');
    this.showErrorState('Server-side loading requires custom implementation.');
  }

  /**
   * 渲染表格体
   */
  renderTableBody(data) {
    const tbody = this.container.querySelector('.dt-table tbody');
    const { columns } = this.options;
    
    if (!data || data.length === 0) {
      tbody.innerHTML = `
        <tr class="dt-empty-row">
          <td colspan="${columns.length}" class="dt-empty" role="alert">
            ${this.options.emptyMessage}
          </td>
        </tr>
      `;
      return;
    }
    
    tbody.innerHTML = data.map((row, rowIndex) => `
      <tr class="dt-row ${this.selectedRows.has(rowIndex) ? 'selected' : ''}"
          data-row-index="${rowIndex}"
          role="row"
          tabindex="0"
          aria-selected="${this.selectedRows.has(rowIndex)}">
        ${columns.map(col => `
          <td class="td-cell td-col-${col.key}" role="cell">
            ${col.render ? col.render(row[col.key], row, rowIndex) : this.defaultRender(row[col.key])}
          </td>
        `).join('')}
      </tr>
    `).join('');
    
    // 绑定行点击事件
    if (this.options.onRowClick) {
      tbody.querySelectorAll('.dt-row').forEach(row => {
        row.addEventListener('click', (e) => {
          const index = parseInt(row.dataset.rowIndex);
          this.options.onRowClick(data[index], e);
        });
      });
    }
  }

  /**
   * 默认渲染器
   */
  defaultRender(value) {
    if (value === null || value === undefined) return '-';
    return String(value);
  }

  /**
   * 显示加载状态
   */
  showLoadingState() {
    const tbody = this.container.querySelector('.dt-table tbody');
    const colCount = this.options.columns.length;
    
    tbody.innerHTML = `
      <tr class="dt-loading-row">
        <td colspan="${colCount}" class="dt-loading" aria-busy="true">
          <span class="loading-dots"><span>.</span><span>.</span><span>.</span></span>
          ${this.options.loadingMessage}
        </td>
      </tr>
    `;
  }

  /**
   * 显示错误状态
   */
  showErrorState(message) {
    const tbody = this.container.querySelector('.dt-table tbody');
    const colCount = this.options.columns.length;
    
    tbody.innerHTML = `
      <tr class="dt-error-row">
        <td colspan="${colCount}" class="dt-error" role="alert">
          ⚠️ Error: ${message}
        </td>
      </tr>
    `;
  }

  /**
   * 更新记录数信息
   */
  updateInfo(from, to) {
    const infoEl = this.container.querySelector('.dt-record-count');
    if (infoEl) {
      infoEl.textContent = `Showing ${from}-${to} of ${this.totalItems} records`;
    }
  }

  /**
   * 全选/取消全选
   */
  toggleAllSelection() {
    const selectAllCheckbox = this.container.querySelector('#dt-select-all');
    const isChecked = selectAllCheckbox?.checked;
    
    this.container.querySelectorAll('.dt-row').forEach(row => {
      row.classList.toggle('selected', isChecked);
      row.setAttribute('aria-selected', isChecked);
      const index = parseInt(row.dataset.rowIndex);
      if (isChecked) {
        this.selectedRows.add(index);
      } else {
        this.selectedRows.delete(index);
      }
    });
  }

  /**
   * 获取选中的行数据
   * 
   * @returns {Array} 选中的行索引数组
   */
  getSelectedRows() {
    return Array.from(this.selectedRows);
  }

  /**
   * 设置数据 (用于外部更新)
   * 
   * @param {Array} data - 新数据
   */
  setData(data) {
    this.options.data = data;
    this.currentPage = 1;
    this.loadData();
  }

  /**
   * 销毁实例
   */
  destroy() {
    this.container.innerHTML = '';
    console.log('[DataTable] Destroyed');
  }
}

// 导出全局类
window.NrcsDataTable = NrcsDataTable;
