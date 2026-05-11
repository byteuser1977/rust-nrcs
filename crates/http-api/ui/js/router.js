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
    
    // 绑定事件处理
    this.handleHashChange = this.handleHashChange.bind(this);
  }

  /**
   * 初始化路由系统
   */
  init() {
    window.addEventListener('hashchange', this.handleHashChange);
    
    // 初始加载时处理当前 hash
    if (window.location.hash) {
      this.handleHashChange();
    } else {
      // 默认路由到 dashboard
      this.navigate('/dashboard');
    }
    
    console.log('Router initialized');
  }

  /**
   * 注册路由
   * @param {string} path - 路由路径 (支持 :param 参数)
   * @param {Object} options - 路由配置
   * @param {Function} options.handler - 路由处理函数
   * @param {string} options.title - 页面标题
   * @param {Object} options.meta - 元数据
   */
  register(path, options) {
    const route = {
      path,
      handler: options.handler || (() => {}),
      title: options.title || '',
      meta: options.meta || {},
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
   * 导航到指定路径
   * @param {string} path - 目标路径
   * @param {Object} params - 可选的查询参数
   */
  navigate(path, params = {}) {
    let url = `#${path}`;
    
    if (Object.keys(params).length > 0) {
      const query = Object.entries(params)
        .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
        .join('&');
      url += `?${query}`;
    }
    
    window.location.href = url;
  }

  /**
   * 替换当前路由（不添加历史记录）
   */
  replace(path) {
    window.location.replace(`#${path}`);
  }

  /**
   * 返回上一页
   */
  back() {
    if (window.history.length > 1) {
      window.history.back();
    } else {
      this.navigate('/');
    }
  }

  /**
   * 处理 hash 变化事件
   */
  async handleHashChange() {
    const hash = window.location.hash.slice(1) || '/';
    const [pathname, queryString] = hash.split('?');
    const params = this.parseQuery(queryString);
    
    // 特殊处理: 如果是 /lock 路径，不进行路由匹配，直接返回
    // 避免未登录时的无限循环重定向
    if (pathname === '/lock' || pathname === 'lock') {
      console.debug('Lock screen requested, skipping route matching');
      return;
    }
    
    // 查找匹配的路由
    const matchedRoute = this.findRoute(pathname);
    
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
      await matchedRoute.handler(this.routeParams);
      
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
    } catch (error) {
      console.error('Route handler error:', error);
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
