/**
 * NRCS Wallet - Router
 * 
 * 单页应用路由管理器
 * 支持基于 hash 的客户端路由
 */

class Router {
  constructor() {
    this.routes = new Map();
    this.currentRoute = null;
    this.previousRoute = null;
    this.routeParams = {};
    this.middlewares = [];
    this.beforeHooks = [];
    this.afterHooks = [];
    
    // P2-R2: 导航模式配置
    // 支持 HTML5 History API 和 Hash 模式
    this.useHistory = this.supportsHistoryAPI();
    this.navigationHistory = [];  // 导航历史栈
    this.maxHistoryLength = 50;   // 最大历史记录数
    
    // 绑定事件处理
    this.handleHashChange = this.handleHashChange.bind(this);
    this.handlePopState = this.handlePopState.bind(this);
  }

  /**
   * 检测浏览器是否支持 History API
   * 
   * @returns {boolean} 是否支持
   */
  supportsHistoryAPI() {
    return !!(window.history && history.pushState);
  }
  
  /**
   * 初始化路由系统
   */
  init() {
    if (this.useHistory) {
      // 使用 HTML5 History API 模式
      window.addEventListener('popstate', this.handlePopState);
      
      // 初始加载时处理当前路径
      const initialPath = window.location.pathname + window.location.search;
      if (initialPath !== '/' && initialPath !== '/index.html') {
        this.handleHistoryNavigation(initialPath);
      } else {
        // 默认路由到 dashboard
        this.navigate('/dashboard');
      }
      
      console.log('[Router] Initialized with HTML5 History API mode');
    } else {
      // 降级为 Hash 模式
      window.addEventListener('hashchange', this.handleHashChange);
      
      // 初始加载时处理当前 hash
      if (window.location.hash) {
        this.handleHashChange();
      } else {
        // 默认路由到 dashboard
        this.navigate('/dashboard');
      }
      
      console.log('[Router] Initialized with Hash fallback mode');
    }
  }

  /**
   * 注册路由 (P2-R4 增强: 支持回调函数)
   * 
   * @param {string} path - 路由路径 (支持 :param 参数)
   * @param {Object} options - 路由配置
   * @param {Function} options.handler - 路由处理函数
   * @param {string} options.title - 页面标题
   * @param {Object} options.meta - 元数据
   * @param {Function} [options.callback] - 页面加载后回调 (参考 NRCS: data.callback)
   */
  register(path, options) {
    const route = {
      path,
      handler: options.handler || (() => {}),
      title: options.title || '',
      meta: options.meta || {},
      callback: options.callback || null,  // P2-R4: 页面回调函数
      pattern: this.pathToRegex(path),
    };
    
    this.routes.set(path, route);
    return this; // 支持链式调用
  }

  /**
   * 将路径转换为正则表达式
   */
  pathToRegex(path) {
    const paramNames = [];
    const regexStr = path.replace(/:([^/]+)/g, (_, paramName) => {
      paramNames.push(paramName);
      return '([^/]+)';
    });
    
    return {
      regex: new RegExp(`^${regexStr}$`),
      paramNames,
    };
  }

  /**
   * 导航到指定路径 (P2-R2 增强)
   * 
   * 支持 HTML5 History API (pushState) 和 Hash 模式
   * 自动选择最佳导航方式
   * 
   * @param {string} path - 目标路径 (例如: '/dashboard')
   * @param {Object} params - 可选的查询参数
   * @param {Object} options - 导航选项 { replace: boolean, title: string }
   */
  navigate(path, params = {}, options = {}) {
    const query = Object.keys(params).length > 0
      ? '?' + Object.entries(params)
          .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
          .join('&')
      : '';
    
    const fullPath = `${path}${query}`;
    
    if (this.useHistory && !options.forceHash) {
      // 使用 HTML5 History API (更美观的 URL)
      try {
        const url = path;  // 不包含查询字符串（可以放在 state 中）
        
        if (options.replace) {
          window.history.replaceState(
            { 
              path, 
              params,
              timestamp: Date.now() 
            },
            options.title || document.title,
            url + query
          );
        } else {
          window.history.pushState(
            { 
              path, 
              params,
              timestamp: Date.now() 
            },
            options.title || document.title,
            url + query
          );
          
          // 记录到导航历史栈
          this.addToNavigationHistory({
            path,
            params,
            type: 'push',
            timestamp: Date.now(),
          });
        }
        
        console.debug(`[Router] History navigation: ${fullPath}`);
        
        // 手动触发路由处理
        this.handleHistoryNavigation(fullPath);
        
      } catch (error) {
        console.warn('[Router] History API failed, falling back to hash:', error);
        this.navigateWithHash(path, params);
      }
    } else {
      // 使用 Hash 模式 (兼容性更好)
      this.navigateWithHash(path, params);
    }
  }

  /**
   * Hash 模式导航 (降级方案)
   */
  navigateWithHash(path, params = {}) {
    let url = `#${path}`;
    
    if (Object.keys(params).length > 0) {
      const query = Object.entries(params)
        .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
        .join('&');
      url += `?${query}`;
    }
    
    window.location.href = url;
    
    console.debug(`[Router] Hash navigation: ${url}`);
  }

  /**
   * 处理 popstate 事件 (浏览器前进/后退按钮)
   * 
   * P2-R2 新增:
   * 响应浏览器的前进/后退操作
   */
  handlePopState(event) {
    if (event.state && event.state.path) {
      console.debug(`[Router] Popstate event:`, event.state);
      this.handleHistoryNavigation(event.state.path);
    }
  }

  /**
   * 处理 History 模式的导航请求
   * 
   * @param {string} fullPath - 完整路径 (包含可能的查询字符串)
   */
  async handleHistoryNavigation(fullPath) {
    const [pathname, queryString] = fullPath.split('?');
    const params = this.parseQuery(queryString);
    
    // 特殊处理 /lock 路径
    if (pathname === '/lock' || pathname === 'lock') {
      console.debug('Lock screen requested via history navigation');
      return;
    }
    
    // 查找匹配的路由
    const matchedRoute = this.findRoute(pathname);
    
    if (!matchedRoute) {
      console.warn(`[Router] No route found for: ${pathname}`);
      
      if (pathname !== '/' && pathname !== '/dashboard') {
        this.navigate('/dashboard', {}, { replace: true });
      }
      return;
    }
    
    // 执行路由处理器 (复用现有逻辑)
    await this.executeRouteHandler(matchedRoute, params, pathname);
  }

  /**
   * 替换当前路由（不添加历史记录）
   * 
   * @param {string} path - 目标路径
   * @param {Object} params - 查询参数
   */
  replace(path, params = {}) {
    this.navigate(path, params, { replace: true });
  }

  /**
   * 返回上一页
   * 
   * P2-R2 增强:
   * 优先使用浏览器历史，否则使用内部导航历史
   */
  back() {
    if (window.history.length > 1) {
      window.history.back();
    } else if (this.navigationHistory.length > 1) {
      // 从内部导航历史返回
      const previousNav = this.navigationHistory[this.navigationHistory.length - 2];
      if (previousNav) {
        this.navigate(previousNav.path, previousNav.params, { replace: true });
      }
    } else {
      this.navigate('/dashboard');
    }
  }

  /**
   * 添加到导航历史栈 (用于调试和自定义后退逻辑)
   * 
   * @param {Object} navEntry - 导航条目
   */
  addToNavigationHistory(navEntry) {
    this.navigationHistory.push(navEntry);
    
    // 限制历史记录长度
    if (this.navigationHistory.length > this.maxHistoryLength) {
      this.navigationHistory.shift();
    }
  }

  /**
   * 获取导航历史
   * 
   * @returns {Array} 导航历史数组
   */
  getNavigationHistory() {
    return [...this.navigationHistory];
  }

  /**
   * 清空导航历史
   */
  clearNavigationHistory() {
    this.navigationHistory = [];
    console.log('[Router] Navigation history cleared');
  }

  /**
   * 处理 hash 变化事件
   */
  async handleHashChange() {
    const hash = window.location.hash.slice(1) || '/';
    await this.executeRouteHandlerFromPath(hash);
  }

  /**
   * 从路径字符串执行路由处理 (公共方法)
   * 
   * P2-R2 新增:
   * 提取公共路由处理逻辑，供 Hash 模式和 History 模式共享
   * 
   * @param {string} fullPath - 完整路径 (例如: '/dashboard?tab=tx')
   */
  async executeRouteHandlerFromPath(fullPath) {
    const [pathname, queryString] = fullPath.split('?');
    const params = this.parseQuery(queryString);
    
    // 特殊处理: 如果是 /lock 路径，不进行路由匹配，直接返回
    if (pathname === '/lock' || pathname === 'lock') {
      console.debug('Lock screen requested, skipping route matching');
      return;
    }
    
    const matchedRoute = this.findRoute(pathname);
    
    if (!matchedRoute) {
      console.warn(`No route found for: ${pathname}`);
      
      if (pathname !== '/' && pathname !== '/dashboard') {
        this.navigate('/dashboard');
      }
      return;
    }
    
    await this.executeRouteHandler(matchedRoute, params, pathname);
  }

  /**
   * 执行路由处理器的核心逻辑 (公共方法)
   * 
   * P2-R2 重构:
   * 从 handleHashChange 提取出通用逻辑
   * 
   * @param {Object} matchedRoute - 匹配的路由对象
   * @param {Object} params - 路由参数
   * @param {string} pathname - 路径名
   */
  async executeRouteHandler(matchedRoute, params, pathname) {
    
    if (!matchedRoute) {
      console.warn(`No route found for: ${pathname}`);
      
      // 只在不是默认首页的情况下才重定向，避免循环
      if (pathname !== '/' && pathname !== '/dashboard') {
        this.navigate('/dashboard');
      }
      return;
    }
    
    // 执行前置钩子
    for (const hook of this.beforeHooks) {
      try {
        await hook(matchedRoute, this.routeParams);
      } catch (error) {
        console.error('Before hook error:', error);
        if (error.message === 'Navigation cancelled') {
          return;
        }
      }
    }
    
    // 更新路由状态
    this.previousRoute = this.currentRoute;
    this.currentRoute = matchedRoute;
    this.routeParams = { ...params, ...matchedRoute.params };
    
    // 执行中间件
    for (const middleware of this.middlewares) {
      try {
        await middleware(this.routeParams);
      } catch (error) {
        // 特殊处理: 导航取消错误 (未登录时显示锁屏)
        if (error.message === 'Navigation cancelled') {
          console.debug('Navigation cancelled by middleware');
          return;
        }
        // 其他错误继续抛出
        throw error;
      }
    }
    
    // 执行路由处理器
    try {
      // Step 1: 显示页面加载动画 (参考 NRCS nrs.js 第 693-699 行)
      this.showPageLoading(matchedRoute);
      
      // Step 2: 重置分页状态 (参考 NRCS nrs.js 第 622-624 行)
      this.resetPaginationState();
      
      // Step 3: 执行实际的路由处理函数
      await matchedRoute.handler(this.routeParams);
      
      // Step 4: 隐藏加载动画 (参考 NRCS nrs.js 第 701-706 行)
      this.hidePageLoading(matchedRoute);
      
      // 更新页面标题
      if (matchedRoute.title) {
        document.title = `${matchedRoute.title} | NRCS Wallet`;
      }
      
      // 更新全局状态
      store.setState('ui.currentPage', pathname.replace('/', ''));
      
      // 执行后置钩子
      for (const hook of this.afterHooks) {
        await hook(matchedRoute, this.routeParams);
      }
      
      // Step 5: 执行页面回调函数 (P2-R4 新增)
      // 参考 NRCS nrs.js 第 569-571 行:
      // if (data && data.callback) { data.callback(); }
      if (matchedRoute.callback && typeof matchedRoute.callback === 'function') {
        try {
          console.debug(`[Router] Executing page callback for: ${pathname}`);
          await matchedRoute.callback(this.routeParams, matchedRoute);
        } catch (callbackError) {
          console.error(`[Router] Page callback error for ${pathname}:`, callbackError);
          // 回调错误不阻止页面加载，仅记录日志
        }
      }
    } catch (error) {
      console.error('Route handler error:', error);
      
      // 确保隐藏加载动画（即使出错）
      this.hidePageLoading(matchedRoute);
      
      this.showErrorPage(error);
    }
  }

  /**
   * 查找匹配的路由
   */
  findRoute(pathname) {
    for (const [, route] of this.routes) {
      const match = pathname.match(route.pattern.regex);
      
      if (match) {
        const params = {};
        
        route.pattern.paramNames.forEach((name, index) => {
          params[name] = decodeURIComponent(match[index + 1]);
        });
        
        return { ...route, params };
      }
    }
    
    return null;
  }

  /**
   * 解析查询字符串
   */
  parseQuery(queryString) {
    if (!queryString) return {};
    
    const params = {};
    const pairs = queryString.split('&');
    
    for (const pair of pairs) {
      const [key, value] = pair.split('=');
      if (key) {
        params[decodeURIComponent(key)] = decodeURIComponent(value || '');
      }
    }
    
    return params;
  }

  /**
   * 添加中间件
   * @param {Function} middleware - 中间件函数
   */
  use(middleware) {
    this.middlewares.push(middleware);
    return this;
  }

  /**
   * 添加前置钩子
   * @param {Function} hook - 钩子函数
   */
  beforeEach(hook) {
    this.beforeHooks.push(hook);
    return this;
  }

  /**
   * 添加后置钩子
   * @param {Function} hook - 钩子函数
   */
  afterEach(hook) {
    this.afterHooks.push(hook);
    return this;
  }

  /**
   * 显示错误页面
   */
  showErrorPage(error) {
    const pageContent = document.getElementById('page-content');
    
    pageContent.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon">⚠️</div>
        <h2 class="empty-state-title">Oops! Something went wrong</h2>
        <p class="empty-state-description">
          ${error.message || 'An unexpected error occurred. Please try again.'}
        </p>
        <button class="btn btn-primary" onclick="router.navigate('/dashboard')">
          Go to Dashboard
        </button>
      </div>
    `;
  }

  /**
   * 获取当前路由信息
   * @returns {Object}
   */
  getCurrentRoute() {
    return this.currentRoute;
  }

  /**
   * 获取路由参数
   * @returns {Object}
   */
  getParams() {
    return { ...this.routeParams };
  }

  /**
   * 获取所有注册的路由
   * @returns {Array}
   */
  getRoutes() {
    return Array.from(this.routes.values());
  }

  /**
   * 显示页面加载动画
   * 
   * 参考 NRCS nrs.js 第 693-699 行:
   * NRS.pageLoading = function () {
   *     NRS.hasMorePages = false;
   *     var $pageHeader = $("#" + NRS.currentPage + "_page .content-header h1");
   *     $pageHeader.find(".loading_dots").remove();
   *     $pageHeader.append("<span class='loading_dots'><span>.</span><span>.</span><span>.</span></span>");
   * };
   * 
   * @param {Object} route - 当前路由对象
   */
  showPageLoading(route) {
    if (!route || !route.path) return;
    
    // 构建页面 ID (例如: /dashboard → dashboard_page)
    const pageId = route.path.replace('/', '') + '_page';
    const pageElement = document.getElementById(pageId);
    
    if (!pageElement) return;
    
    // 查找页面标题元素 (参考 NRCS: .content-header h1)
    const headerElement = pageElement.querySelector('.content-header h1') || 
                          pageElement.querySelector('h1') ||
                          pageElement.querySelector('.page-title');
    
    if (!headerElement) return;
    
    // 移除已有的 loading_dots (参考 NRCS)
    this.removeExistingLoadingDots(headerElement);
    
    // 添加新的 loading 动画 (参考 NRCS)
    const loadingDots = document.createElement('span');
    loadingDots.className = 'loading-dots';
    loadingDots.innerHTML = '<span>.</span><span>.</span><span>.</span>';
    headerElement.appendChild(loadingDots);
    
    console.debug(`[Router] Page loading animation shown for: ${pageId}`);
  }

  /**
   * 隐藏页面加载动画
   * 
   * 参考 NRCS nrs.js 第 701-706 行:
   * NRS.pageLoaded = function (callback) {
   *     var $currentPage = $("#" + NRS.currentPage + "_page");
   *     $currentPage.find(".content-header h1 .loading_dots").remove();
   *     ...
   * };
   * 
   * @param {Object} route - 当前路由对象
   */
  hidePageLoading(route) {
    if (!route || !route.path) return;
    
    // 构建页面 ID
    const pageId = route.path.replace('/', '') + '_page';
    const pageElement = document.getElementById(pageId);
    
    if (!pageElement) return;
    
    // 查找并移除所有 loading-dots 元素 (参考 NRCS)
    const loadingElements = pageElement.querySelectorAll('.loading-dots');
    loadingElements.forEach(element => element.remove());
    
    console.debug(`[Router] Page loading animation hidden for: ${pageId}`);
  }

  /**
   * 移除已存在的加载动画元素
   * @param {HTMLElement} parent - 父元素
   */
  removeExistingLoadingDots(parent) {
    const existingDots = parent.querySelectorAll('.loading-dots');
    existingDots.forEach(dot => dot.remove());
  }

  /**
   * 重置分页状态
   * 
   * 参考 NRCS nrs.js 第 622-624 行:
   * NRS.currentSubPage = "";
   * NRS.pageNumber = 1;
   * NRS.showPageNumbers = false;
   */
  resetPaginationState() {
    // 更新全局 Store 的分页状态
    store.setState('pagination', {
      currentPage: 1,           // 参考 NRCS: NRS.pageNumber = 1
      itemsPerPage: 50,         // NRCS 默认值
      hasMorePages: false,      // 参考 NRCS: NRS.hasMorePages = false
      showPageNumbers: false,   // 参考 NRCS: NRS.showPageNumbers = false
      currentSubPage: '',       // 参考 NRCS: NRS.currentSubPage = ""
    });
    
    console.debug('[Router] Pagination state reset to defaults');
  }

  /**
   * 销毁路由器
   */
  destroy() {
    window.removeEventListener('hashchange', this.handleHashChange);
    this.routes.clear();
    this.middlewares = [];
    this.beforeHooks = [];
    this.afterHooks = [];
  }
}

// 创建全局单例实例
window.router = new Router();
