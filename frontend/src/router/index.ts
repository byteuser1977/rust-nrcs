import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

export interface AppRouteRecordRaw extends Omit<RouteRecordRaw, 'meta'> {
  meta?: {
    title?: string
    icon?: string
    roles?: string[]
    permissions?: string[]
    hidden?: boolean
    requireAuth?: boolean
    layout?: 'main' | 'sub' | 'blank'
    isButton?: boolean
    isDivider?: boolean
  }
}

const routes: AppRouteRecordRaw[] = [
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
  {
    path: '/404',
    name: 'NotFound',
    component: () => import('@/views/NotFound.vue'),
    meta: {
      title: '404 Not Found',
      layout: 'blank'
    }
  },
  {
    path: '/403',
    name: 'Forbidden',
    component: () => import('@/views/Forbidden.vue'),
    meta: {
      title: '403 Forbidden',
      layout: 'blank'
    }
  },
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
      // Dashboard (仪表盘)
      {
        path: 'dashboard',
        name: 'DashboardParent',
        meta: {
          title: '仪表盘',
          icon: 'Odometer',
          requireAuth: true
        },
        children: [
          {
            path: '',
            name: 'Dashboard',
            component: () => import('@/views/dashboard/Dashboard.vue'),
            meta: {
              title: '面板',
              icon: 'Monitor',
              requireAuth: true
            }
          },
          {
            path: 'ledger',
            name: 'Ledger',
            component: () => import('@/views/dashboard/Ledger.vue'),
            meta: {
              title: '账户总账',
              icon: 'Notebook',
              requireAuth: true
            }
          },
          {
            path: 'properties',
            name: 'AccountProperties',
            component: () => import('@/views/dashboard/AccountProperties.vue'),
            meta: {
              title: '账户属性',
              icon: 'Setting',
              requireAuth: true
            }
          },
          {
            path: 'transactions',
            name: 'MyTransactions',
            component: () => import('@/views/dashboard/Transactions.vue'),
            meta: {
              title: '我的交易',
              icon: 'List',
              requireAuth: true
            }
          },
          {
            path: 'approval-requests',
            name: 'DashboardApprovalRequests',
            component: () => import('@/views/dashboard/ApprovalRequests.vue'),
            meta: {
              title: '批准请求',
              icon: 'Finished',
              requireAuth: true
            }
          }
        ]
      },

      // Assets (资产)
      {
        path: 'assets',
        name: 'AssetsParent',
        meta: {
          title: '资产',
          icon: 'TrendCharts',
          requireAuth: true
        },
        children: [
          {
            path: 'exchange',
            name: 'AssetExchange',
            component: () => import('@/views/asset/AssetExchange.vue'),
            meta: {
              title: '资产交易',
              icon: 'Sell',
              requireAuth: true
            }
          },
          {
            path: 'trade-history',
            name: 'TradeHistory',
            component: () => import('@/views/asset/TradeHistory.vue'),
            meta: {
              title: '交易历史',
              icon: 'Timer',
              requireAuth: true
            }
          },
          {
            path: 'transfer-history',
            name: 'AssetTransferHistory',
            component: () => import('@/views/asset/TransferHistory.vue'),
            meta: {
              title: '转移历史',
              icon: 'Sort',
              requireAuth: true
            }
          },
          {
            path: 'deletes-history',
            name: 'DeletesHistory',
            component: () => import('@/views/asset/DeletesHistory.vue'),
            meta: {
              title: '删除历史',
              icon: 'Delete',
              requireAuth: true
            }
          },
          {
            path: 'my-assets',
            name: 'MyAssets',
            component: () => import('@/views/asset/MyAssets.vue'),
            meta: {
              title: '我的资产',
              icon: 'Wallet',
              requireAuth: true
            }
          },
          {
            path: 'open-orders',
            name: 'OpenOrders',
            component: () => import('@/views/asset/OpenOrders.vue'),
            meta: {
              title: '开放订单',
              icon: 'Document',
              requireAuth: true
            }
          },
          {
            path: 'approval-requests',
            name: 'AssetApprovalRequests',
            component: () => import('@/views/asset/ApprovalRequests.vue'),
            meta: {
              title: '批准请求',
              icon: 'Finished',
              requireAuth: true
            }
          },
          {
            path: 'issue',
            name: 'IssueAsset',
            component: () => import('@/views/asset/IssueAsset.vue'),
            meta: {
              title: '发行资产',
              icon: 'Plus',
              requireAuth: true,
              isButton: true
            }
          }
        ]
      },

      // Monetary System (积分系统)
      {
        path: 'monetary',
        name: 'MonetaryParent',
        meta: {
          title: '积分系统',
          icon: 'Coin',
          requireAuth: true
        },
        children: [
          {
            path: 'currencies',
            name: 'Currencies',
            component: () => import('@/views/monetary/Currencies.vue'),
            meta: {
              title: '积分',
              icon: 'Money',
              requireAuth: true
            }
          },
          {
            path: 'exchange-history',
            name: 'ExchangeHistory',
            component: () => import('@/views/monetary/ExchangeHistory.vue'),
            meta: {
              title: '交易历史',
              icon: 'Timer',
              requireAuth: true
            }
          },
          {
            path: 'transfer-history',
            name: 'MonetaryTransferHistory',
            component: () => import('@/views/monetary/TransferHistory.vue'),
            meta: {
              title: '转移历史',
              icon: 'Sort',
              requireAuth: true
            }
          },
          {
            path: 'approval-requests',
            name: 'MonetaryApprovalRequests',
            component: () => import('@/views/monetary/ApprovalRequests.vue'),
            meta: {
              title: '批准请求',
              icon: 'Finished',
              requireAuth: true
            }
          },
          {
            path: 'issue',
            name: 'IssueCurrency',
            component: () => import('@/views/monetary/IssueCurrency.vue'),
            meta: {
              title: '发行积分',
              icon: 'Plus',
              requireAuth: true,
              isButton: true
            }
          }
        ]
      },

      // Voting (投票系统)
      {
        path: 'voting',
        name: 'VotingParent',
        meta: {
          title: '投票系统',
          icon: 'Checked',
          requireAuth: true
        },
        children: [
          {
            path: 'active-polls',
            name: 'ActivePolls',
            component: () => import('@/views/voting/ActivePolls.vue'),
            meta: {
              title: '激活的投票',
              icon: 'DataLine',
              requireAuth: true
            }
          },
          {
            path: 'followed-polls',
            name: 'FollowedPolls',
            component: () => import('@/views/voting/FollowedPolls.vue'),
            meta: {
              title: '关注的投票',
              icon: 'Star',
              requireAuth: true
            }
          },
          {
            path: 'my-votes',
            name: 'MyVotes',
            component: () => import('@/views/voting/MyVotes.vue'),
            meta: {
              title: '我的投票',
              icon: 'Select',
              requireAuth: true
            }
          },
          {
            path: 'my-polls',
            name: 'MyPolls',
            component: () => import('@/views/voting/MyPolls.vue'),
            meta: {
              title: '我创建的投票',
              icon: 'EditPen',
              requireAuth: true
            }
          },
          {
            path: 'create',
            name: 'CreatePoll',
            component: () => import('@/views/voting/CreatePoll.vue'),
            meta: {
              title: '创建投票',
              icon: 'Plus',
              requireAuth: true,
              isButton: true
            }
          }
        ]
      },

      // Marketplace (市场)
      {
        path: 'marketplace',
        name: 'MarketplaceParent',
        meta: {
          title: '市场',
          icon: 'ShoppingCart',
          requireAuth: true
        },
        children: [
          {
            path: 'search',
            name: 'MarketplaceSearch',
            component: () => import('@/views/marketplace/MarketplaceSearch.vue'),
            meta: {
              title: '市场',
              icon: 'Search',
              requireAuth: true
            }
          },
          {
            path: 'purchased',
            name: 'PurchasedProducts',
            component: () => import('@/views/marketplace/PurchasedProducts.vue'),
            meta: {
              title: '已购买产品',
              icon: 'Goods',
              requireAuth: true
            }
          },
          {
            path: 'my-products',
            name: 'MyProducts',
            component: () => import('@/views/marketplace/MyProducts.vue'),
            meta: {
              title: '我的出售产品',
              icon: 'Sell',
              requireAuth: true
            }
          },
          {
            path: 'pending-orders',
            name: 'PendingOrders',
            component: () => import('@/views/marketplace/PendingOrders.vue'),
            meta: {
              title: '待处理订单',
              icon: 'Clock',
              requireAuth: true
            }
          },
          {
            path: 'completed-orders',
            name: 'CompletedOrders',
            component: () => import('@/views/marketplace/CompletedOrders.vue'),
            meta: {
              title: '已完成订单',
              icon: 'CircleCheck',
              requireAuth: true
            }
          },
          {
            path: 'list-product',
            name: 'ListProductForSale',
            component: () => import('@/views/marketplace/ListProduct.vue'),
            meta: {
              title: '出售产品',
              icon: 'Plus',
              requireAuth: true,
              isButton: true
            }
          }
        ]
      },

      // Data Cloud (数据云)
      {
        path: 'datacloud',
        name: 'DataCloudParent',
        meta: {
          title: '数据云',
          icon: 'Cloudy',
          requireAuth: true
        },
        children: [
          {
            path: 'search',
            name: 'DataSearch',
            component: () => import('@/views/datacloud/DataSearch.vue'),
            meta: {
              title: '搜索',
              icon: 'Search',
              requireAuth: true
            }
          },
          {
            path: 'upload',
            name: 'FileUpload',
            component: () => import('@/views/datacloud/FileUpload.vue'),
            meta: {
              title: '文件上传',
              icon: 'Upload',
              requireAuth: true,
              isButton: true
            }
          }
        ]
      },

      // Messages (信息)
      {
        path: 'messages',
        name: 'MessagesParent',
        meta: {
          title: '信息',
          icon: 'ChatDotRound',
          requireAuth: true
        },
        children: [
          {
            path: '',
            name: 'Messages',
            component: () => import('@/views/message/Messages.vue'),
            meta: {
              title: '聊天',
              icon: 'ChatLineRound',
              requireAuth: true
            }
          }
        ]
      },

      // Aliases (别名)
      {
        path: 'aliases',
        name: 'Aliases',
        component: () => import('@/views/alias/Aliases.vue'),
        meta: {
          title: '别名',
          icon: 'Bookmark',
          requireAuth: true
        }
      },

      // Settings pages (from header Settings dropdown)
      {
        path: 'settings',
        name: 'SettingsParent',
        meta: {
          title: '设置',
          icon: 'Setting',
          requireAuth: true,
          hidden: true
        },
        children: [
          {
            path: 'blocks',
            name: 'SettingsBlocks',
            component: () => import('@/views/settings/Blocks.vue'),
            meta: {
              title: '区块',
              icon: 'Grid',
              requireAuth: true
            }
          },
          {
            path: 'peers',
            name: 'SettingsPeers',
            component: () => import('@/views/settings/Peers.vue'),
            meta: {
              title: '节点',
              icon: 'Connection',
              requireAuth: true
            }
          },
          {
            path: 'generators',
            name: 'Generators',
            component: () => import('@/views/settings/Generators.vue'),
            meta: {
              title: '生成者',
              icon: 'Cpu',
              requireAuth: true
            }
          },
          {
            path: 'scheduled-transactions',
            name: 'ScheduledTransactions',
            component: () => import('@/views/settings/ScheduledTransactions.vue'),
            meta: {
              title: '计划交易',
              icon: 'AlarmClock',
              requireAuth: true
            }
          },
          {
            path: 'monitors',
            name: 'FundingMonitors',
            component: () => import('@/views/settings/FundingMonitors.vue'),
            meta: {
              title: '监控',
              icon: 'View',
              requireAuth: true
            }
          },
          {
            path: 'plugins',
            name: 'Plugins',
            component: () => import('@/views/settings/Plugins.vue'),
            meta: {
              title: '插件',
              icon: 'Opportunity',
              requireAuth: true
            }
          },
          {
            path: 'account',
            name: 'AccountSettings',
            component: () => import('@/views/settings/AccountSettings.vue'),
            meta: {
              title: '账户设置',
              icon: 'UserFilled',
              requireAuth: true
            }
          },
          {
            path: 'token',
            name: 'TokenGenerator',
            component: () => import('@/views/settings/TokenGenerator.vue'),
            meta: {
              title: '生成令牌',
              icon: 'Key',
              requireAuth: true
            }
          },
          {
            path: 'hallmark',
            name: 'HallmarkGenerator',
            component: () => import('@/views/settings/HallmarkGenerator.vue'),
            meta: {
              title: '生成标记',
              icon: 'Stamp',
              requireAuth: true
            }
          },
          {
            path: 'hash-calculator',
            name: 'HashCalculator',
            component: () => import('@/views/settings/HashCalculator.vue'),
            meta: {
              title: '计算哈希',
              icon: 'MagicStick',
              requireAuth: true
            }
          },
          {
            path: 'transaction-operations',
            name: 'TransactionOperations',
            component: () => import('@/views/settings/TransactionOperations.vue'),
            meta: {
              title: '交易操作',
              icon: 'Operation',
              requireAuth: true
            }
          }
        ]
      },

      // Contacts (from header)
      {
        path: 'contacts',
        name: 'Contacts',
        component: () => import('@/views/contacts/Contacts.vue'),
        meta: {
          title: '联系人',
          icon: 'Phone',
          requireAuth: true,
          hidden: true
        }
      }
    ]
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/404'
  }
]

export const checkRoutePermission = (route: AppRouteRecordRaw, userRoles: string[]): boolean => {
  if (!route.meta?.roles || route.meta.roles.length === 0) {
    return true
  }
  return userRoles.some((role) => route.meta.roles!.includes(role))
}

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

router.beforeEach(async (to, from, next) => {
  if (to.meta?.title) {
    document.title = `${to.meta.title} - NRCS`
  }

  const requireAuth = to.matched.some((record) => record.meta?.requireAuth !== false)

  if (!requireAuth) {
    next()
    return
  }

  const token = localStorage.getItem('access_token')
  if (!token) {
    next({ path: '/login', query: { redirect: to.fullPath } })
    return
  }

  next()
})

router.afterEach(() => {
})

export const isRouteActive = (routePath: string): boolean => {
  return router.currentRoute.value.path === routePath ||
    router.currentRoute.value.path.startsWith(routePath + '/')
}

export default router
