import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import { getDynamicRoutes } from '@/api/modules/account'

// ============ 路由类型定义 ============

export interface AppRouteRecordRaw extends RouteRecordRaw {
  meta?: {
    title?: string
    icon?: string
    roles?: string[]
    permissions?: string[]
    hidden?: boolean
    requireAuth?: boolean
    layout?: 'main' | 'sub' | 'blank'
  }
}

// ============ 静态路由定义 ============

const routes: AppRouteRecordRaw[] = [
  // 登录页（公开）
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/Login.vue'),
    meta: {
      title: '登录',
      layout: 'blank',
      requireAuth: false
    }
  },

  // 404页面
  {
    path: '/404',
    name: 'NotFound',
    component: () => import('@/views/NotFound.vue'),
    meta: {
      title: '404 Not Found',
      layout: 'blank'
    }
  },

  // 403无权限页面
  {
    path: '/403',
    name: 'Forbidden',
    component: () => import('@/views/Forbidden.vue'),
    meta: {
      title: '403 Forbidden',
      layout: 'blank'
    }
  },

  // 主布局路由（需要认证）
  {
    path: '/',
    name: 'Layout',
    component: () => import('@/components/layout/MainLayout.vue'),
    redirect: '/dashboard',
    meta: {
      requireAuth: true,
      layout: 'main'
    },
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/Dashboard.vue'),
        meta: {
          title: '仪表盘',
          icon: 'HomeFilled',
          roles: ['admin', 'operator', 'viewer']
        }
      },

      // 账户管理子路由
      {
        path: 'account',
        name: 'AccountLayout',
        component: () => import('@/components/layout/SubLayout.vue'),
        meta: {
          title: '账户管理',
          icon: 'User',
          roles: ['admin', 'operator']
        },
        children: [
          {
            path: 'register',
            name: 'Register',
            component: () => import('@/views/account/Register.vue'),
            meta: {
              title: '注册',
              icon: 'UserFilled',
              roles: ['admin', 'operator'],
              requireAuth: false
            }
          },
          {
            path: 'wallet-connect',
            name: 'WalletConnect',
            component: () => import('@/views/account/WalletConnect.vue'),
            meta: {
              title: '连接钱包',
              icon: 'Connection',
              roles: ['admin', 'operator'],
              requireAuth: false
            }
          },
          {
            path: 'list',
            name: 'AccountList',
            component: () => import('@/views/account/AccountList.vue'),
            meta: {
              title: '账户列表',
              icon: 'List',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'detail/:address',
            name: 'AccountDetail',
            component: () => import('@/views/account/AccountDetail.vue'),
            meta: {
              title: '账户详情',
              icon: 'Document',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'users',
            name: 'UserList',
            component: () => import('@/views/account/UserList.vue'),
            meta: {
              title: '用户列表',
              icon: 'UserFilled',
              roles: ['admin', 'operator']
            }
          },
          {
            path: 'profile',
            name: 'UserProfile',
            component: () => import('@/views/account/UserProfile.vue'),
            meta: {
              title: '个人资料',
              icon: 'UserCircleFilled',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'roles',
            name: 'RoleList',
            component: () => import('@/views/account/RoleList.vue'),
            meta: {
              title: '角色管理',
              icon: 'Avatar',
              roles: ['admin']
            }
          },
          {
            path: 'permissions',
            name: 'PermissionList',
            component: () => import('@/views/account/PermissionList.vue'),
            meta: {
              title: '权限管理',
              icon: 'Lock',
              roles: ['admin']
            }
          }
        ]
      },

      // 交易管理子路由
      {
        path: 'transaction',
        name: 'TransactionLayout',
        component: () => import('@/components/layout/SubLayout.vue'),
        meta: {
          title: '交易管理',
          icon: 'DocumentCopy',
          roles: ['admin', 'operator', 'viewer']
        },
        children: [
          {
            path: 'list',
            name: 'TransactionList',
            component: () => import('@/views/transaction/TransactionList.vue'),
            meta: {
              title: '交易列表',
              icon: 'List',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'send',
            name: 'SendTransaction',
            component: () => import('@/views/transaction/SendTransaction.vue'),
            meta: {
              title: '发送交易',
              icon: 'Position',
              roles: ['admin', 'operator']
            }
          },
          {
            path: 'pending',
            name: 'PendingTransactions',
            component: () => import('@/views/transaction/PendingTransactions.vue'),
            meta: {
              title: '待确认交易',
              icon: 'Timer',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'detail/:hash',
            name: 'TransactionDetail',
            component: () => import('@/views/transaction/TransactionDetail.vue'),
            meta: {
              title: '交易详情',
              icon: 'Document',
              roles: ['admin', 'operator', 'viewer']
            }
          }
        ]
      },

      // 合约管理子路由
      {
        path: 'contract',
        name: 'ContractLayout',
        component: () => import('@/components/layout/SubLayout.vue'),
        meta: {
          title: '合约管理',
          icon: 'Box',
          roles: ['admin', 'operator', 'viewer']
        },
        children: [
          {
            path: 'deploy',
            name: 'DeployContract',
            component: () => import('@/views/contract/DeployContract.vue'),
            meta: {
              title: '部署合约',
              icon: 'Plus',
              roles: ['admin', 'operator']
            }
          },
          {
            path: 'list',
            name: 'ContractList',
            component: () => import('@/views/contract/ContractList.vue'),
            meta: {
              title: '合约列表',
              icon: 'Files',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'detail/:address',
            name: 'ContractDetail',
            component: () => import('@/views/contract/ContractDetail.vue'),
            meta: {
              title: '合约详情',
              icon: 'Document',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'verify',
            name: 'ContractVerify',
            component: () => import('@/views/contract/ContractVerify.vue'),
            meta: {
              title: '合约验证',
              icon: 'Check',
              roles: ['admin', 'operator']
            }
          }
        ]
      },

      // 节点管理子路由
      {
        path: 'node',
        name: 'NodeLayout',
        component: () => import('@/components/layout/SubLayout.vue'),
        meta: {
          title: '节点管理',
          icon: 'Monitor',
          roles: ['admin', 'operator']
        },
        children: [
          {
            path: 'status',
            name: 'NodeStatus',
            component: () => import('@/views/node/NodeStatus.vue'),
            meta: {
              title: '节点状态',
              icon: 'DataAnalysis',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'peers',
            name: 'PeerList',
            component: () => import('@/views/node/PeerList.vue'),
            meta: {
              title: '节点列表',
              icon: 'Connection',
              roles: ['admin', 'operator']
            }
          },
          {
            path: 'blocks',
            name: 'BlockList',
            component: () => import('@/views/node/BlockList.vue'),
            meta: {
              title: '区块浏览',
              icon: 'Grid',
              roles: ['admin', 'operator', 'viewer']
            }
          },
          {
            path: 'monitor',
            name: 'NodeMonitor',
            component: () => import('@/views/node/NodeMonitor.vue'),
            meta: {
              title: '性能监控',
              icon: 'TrendCharts',
              roles: ['admin', 'operator']
            }
          }
        ]
      }
    ]
  },

  // 通配符404（捕获所有未匹配路由）
  {
    path: '/:pathMatch(.*)*',
    redirect: '/404'
  }
]

// ============ 权限工具函数 ============

/**
 * 检查用户是否有权限访问路由
 */
export const checkRoutePermission = (route: AppRouteRecordRaw, userRoles: string[]): boolean => {
  if (!route.meta?.roles || route.meta.roles.length === 0) {
    return true
  }

  return userRoles.some((role) => route.meta.roles!.includes(role))
}

/**
 * 从路由元数据获取面包屑
 */
export const getBreadcrumb = (route: AppRouteRecordRaw): Array<{ name: string; path: string }> => {
  const breadcrumb: Array<{ name: string; path: string }> = []

  if (route.meta?.title) {
    breadcrumb.push({
      name: route.meta.title,
      path: route.path
    })
  }

  return breadcrumb
}

// ============ 动态路由加载 ============

/**
 * 从后端获取动态路由配置
 * 此函数用于基于用户权限动态生成路由
 */
export const loadDynamicRoutes = async (): Promise<AppRouteRecordRaw[]> => {
  try {
    // 假设后端提供接口返回用户可访问的路由配置
    const response = await getDynamicRoutes()
    const backendRoutes = response.data as any[]

    return transformBackendRoutes(backendRoutes)
  } catch (error) {
    console.error('[Router] Failed to load dynamic routes:', error)
    return []
  }
}

/**
 * 将后端路由格式转换为 Vue Router 格式
 */
const transformBackendRoutes = (backendRoutes: any[]): AppRouteRecordRaw[] => {
  return backendRoutes.map((route) => ({
    path: route.path,
    name: route.name,
    component: () => import(`@/views${route.component}`), // 懒加载
    meta: {
      title: route.title,
      icon: route.icon,
      roles: route.roles,
      permissions: route.permissions,
      hidden: route.hidden || false,
      requireAuth: true,
      layout: route.layout || 'sub'
    },
    children: route.children ? transformBackendRoutes(route.children) : undefined
  }))
}

// ============ 创建路由实例 ============

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      return savedPosition
    }
    return { top: 0 }
  }
})

// ============ 路由守卫 ============

/**
 * 全局前置守卫
 */
router.beforeEach(async (to, from, next) => {
  // 设置页面标题
  if (to.meta?.title) {
    document.title = `${to.meta.title} - NRCS`
  }

  // 检查是否需要认证
  const requireAuth = to.matched.some((record) => record.meta?.requireAuth !== false)

  if (!requireAuth) {
    next()
    return
  }

  // 检查Token是否存在
  const token = localStorage.getItem('access_token')
  if (!token) {
    next({ path: '/login', query: { redirect: to.fullPath } })
    return
  }

  // 如果有角色权限要求，且尚未加载用户信息，则先加载
  const needAuthCheck = to.matched.some((record) => record.meta?.roles?.length)
  if (needAuthCheck) {
    // TODO: 这里可以检查用户角色是否已加载
    // const authStore = useAuthStore()
    // if (!authStore.user) {
    //   await authStore.fetchUserInfo()
    // }
  }

  next()
})

/**
 * 全局后置守卫
 */
router.afterEach((to) => {
  // 发送页面浏览统计（Analytics）
  if (import.meta.env.VITE_ENABLE_ANALYTICS === 'true') {
    console.log(`[Analytics] Page view: ${to.fullPath}`)
  }
})

// ============ 路由工具函数 ============

/**
 * 获取当前路由的父级路由
 */
export const getParentRoutes = (route: AppRouteRecordRaw): AppRouteRecordRaw[] => {
  const parents: AppRouteRecordRaw[] = []
  let parent = route

  while (parent.parent) {
    parent = parent.parent as AppRouteRecordRaw
    parents.unshift(parent)
  }

  return parents
}

/**
 * 判断路由是否激活
 */
export const isRouteActive = (routePath: string): boolean => {
  return router.currentRoute.value.path === routePath ||
    router.currentRoute.value.path.startsWith(routePath + '/')
}

// ============ 导出 ============

export default router
