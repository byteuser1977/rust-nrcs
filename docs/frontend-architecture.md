# NRCS 区块链平台 - 前端架构设计文档

## 1. 技术栈选型

### 1.1 核心框架对比

| 技术领域 | 选型 | 版本 | 理由 |
|---------|------|------|------|
| 框架 | Vue 3 | 3.4+ | Composition API、更好的性能、TypeScript 一等支持 |
| 语言 | TypeScript | 5+ | 类型安全、更好的开发体验、IDE 智能提示 |
| 构建工具 | Vite | 5+ | 极速冷启动、原生 ESM、HMR |
| 状态管理 | Pinia | 2+ | Vuex 5 理念、TypeScript 原生支持、API 简洁 |
| 路由 | Vue Router | 4+ | Vue 3 官方路由、懒加载、导航守卫 |
| UI 库 | Element Plus | 最新版 | 企业级组件、TypeScript 支持、主题定制、区块链场景适用 |

### 1.2 UI 库对比评估

| 维度 | Element Plus | Ant Design Vue | Naive UI |
|------|-------------|----------------|----------|
| TypeScript 支持 | ✅ 完整 | ✅ 完整 | ✅ 完整 |
| 组件丰富度 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| 主题定制 | SCSS 变量 | Less 变量 | CSS-in-JS（灵活） |
| 文档质量 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| 区块链组件 | 通用表格/表单 | 通用表格/表单 | 通用表格/表单 |
| 包大小 | ~300KB (gzipped) | ~400KB (gzipped) | ~200KB (gzipped) |
| 社区活跃度 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |

**选型结论：Element Plus**
- 与 Ant Design 相比，包体积更小
- 与 Naive UI 相比，组件更成熟、文档更完善
- 表格组件适合区块链数据展示（交易列表、区块浏览器）
- SCSS 变量便于主题定制（暗色模式支持）

### 1.3 状态管理：Pinia vs Vuex 4

| 特性 | Pinia | Vuex 4 |
|------|-------|--------|
| TypeScript 支持 | 🎯 原生（无需额外配置） | ⚠️ 需要复杂类型声明 |
| API 设计 | 🎯 组合式 API 风格 | 选项式 API |
| Mutations | ❌ 不需要（直接修改 state） | ✅ 必须（强制纯函数） |
| Modules | ✅ 天然模块化（每个 store 独立） | ✅ 需要命名空间 |
| Devtools | ✅ 完美支持 | ✅ 支持 |
| 包大小 | ~4KB | ~10KB |

**选型结论：Pinia**
- Vue 3 官方推荐，Vuex 5 会基于 Pinia 理念
- 更符合 Composition API 开发模式
- 类型推导更准确，减少类型断言
- 代码更简洁（无需 mutations）

### 1.4 HTTP 客户端：Axios vs Fetch

| 特性 | Axios | Fetch API |
|------|-------|-----------|
| 浏览器兼容 | ✅ IE10+（polyfill） | ✅ 现代浏览器 |
| 请求取消 | ✅ CancelToken | ✅ AbortController |
| 请求拦截 | ✅ 全局拦截器 | ❌ 需要手动包装 |
| 响应拦截 | ✅ 全局拦截器 | ❌ 需要手动包装 |
| 自动转换 JSON | ✅ | ✅ |
| 超时设置 | ✅ `timeout` 配置 | ❌ 需要 AbortController |
| 进度监控 | ✅ `onUploadProgress` | ✅ Stream API（复杂） |
| TypeScript 支持 | ✅ 完整 | ✅ 原生 |

**选型结论：Axios + 拦截器**
- 统一错误处理（拦截器）
- 请求/响应日志（调试便利）
- 自动重试机制（网络不稳定场景）
- 请求取消（组件卸载时）
- 拦截 token 自动注入

## 2. 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                      用户界面层 (UI Layer)                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐   │
│  │ 账户页面 │  │ 交易页面 │  │ 合约页面 │  │ 节点监控页面 │   │
│  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬──────┘   │
│       │ 懒加载      │ 懒加载      │ 懒加载          │ 懒加载   │
│       └─────────────┴─────────────┴───────────────┘       │
└─────────────────────────┬───────────────────────────────────┘
                          │ 路由守卫 (auth/permission)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    路由层 (Router Layer)                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ 动态路由配置（基于用户权限）                          │  │
│  │ - /account/*   账户管理（登录/注册/钱包）             │  │
│  │ - /transaction/* 交易操作（发送/历史）                │  │
│  │ - /contract/*   合约交互（部署/调用）                │  │
│  │ - /node/*       节点监控（状态/统计）                │  │
│  │ - /dashboard    仪表盘（首页）                       │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    API 集成层 (API Layer)                  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Axios 实例 + 拦截器链                                 │  │
│  │  ├─ Request Interceptor: 认证 token 注入              │  │
│  │  ├─ Request Interceptor: 请求日志                     │  │
│  │  ├─ Response Interceptor: 错误统一处理                │  │
│  │  ├─ Response Interceptor: 自动登出检测               │  │
│  │  └─ Retry 中间件（网络错误自动重试）                  │  │
│  │                                                       │  │
│  │  API 模块划分（按业务域）：                            │  │
│  │  - account.api.ts   账户认证、钱包信息                │  │
│  │  - transaction.api.ts 交易查询、发送                  │  │
│  │  - contract.api.ts   合约部署、调用                   │  │
│  │  - node.api.ts       节点状态、网络信息               │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   状态管理层 (Store Layer)                │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │ useAccountStore │ useTransactionStore │ useContractStore │   │
│  │ - 用户信息    │  │ - 交易记录   │  │ - 合约列表     │   │
│  │ - 钱包地址   │  │ - 发送状态   │  │ - ABI 存储    │   │
│  │ - 认证 token │  │ - 待确认交易 │  │ - Gas 价格    │   │
│  └──────────────┘  └──────────────┘  └─────────────────┘   │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │ useNodeStore │  │ useUiStore   │  │ useAppStore    │   │
│  │ - 节点状态   │  │ - 主题模式   │  │ - 全局配置    │   │
│  │ - 网络信息   │  │ - 语言设置   │  │ - 版本信息    │   │
│  │ - 区块高度   │  │ - 侧栏状态   │  │ - 初始加载    │   │
│  └──────────────┘  └──────────────┘  └─────────────────┘   │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    类型定义层 (Types Layer)               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  API 响应类型                                        │  │
│  │  - ApiResponse<T>   { code, message, data }         │  │
│  │  - PageResult<T>    { total, list, page, size }    │  │
│  │                                                       │  │
│  │  业务实体类型                                        │  │
│  │  - User            { id, name, address, role }      │  │
│  │  - Transaction     { hash, from, to, value, status }│  │
│  │  - Block           { number, hash, timestamp }      │  │
│  │  - Contract        { address, abi, bytecode }       │  │
│  │                                                       │  │
│  │  枚举定义                                            │  │
│  │  - UserRole        ADMIN / USER / GUEST             │  │
│  │  - TxStatus        PENDING / CONFIRMING / SUCCESS / FAILED│ │
│  │  - ChainId         MAINNET / TESTNET / LOCALNET     │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 3. 目录结构设计

```
frontend/
├── index.html                    # HTML 模板
├── package.json                  # 依赖配置
├── tsconfig.json                 # TypeScript 配置
├── vite.config.ts                # Vite 配置
├── .env.example                  # 环境变量示例
├── .env.development              # 开发环境变量
├── .env.production               # 生产环境变量
├── public/                       # 静态资源
│   ├── favicon.ico
│   └── manifest.json
├── src/
│   ├── main.ts                   # 应用入口
│   ├── App.vue                   # 根组件
│   ├── assets/                   # 静态资源
│   │   ├── styles/               # 全局样式
│   │   │   ├── index.scss        # 主入口
│   │   │   ├── variables.scss    # SCSS 变量（主题）
│   │   │   └── mixins.scss       # 混入
│   │   ├── images/               # 图片资源
│   │   └── fonts/                # 字体
│   ├── components/               # 通用组件
│   │   ├── common/               # 基础组件（Button、Input 等）
│   │   ├── layout/               # 布局组件（Header、Sidebar、Footer）
│   │   └── charts/               # 图表组件（ECharts 封装）
│   ├── composables/              # 组合式函数
│   │   ├── useWallet.ts          # 钱包连接逻辑
│   │   ├── useNetwork.ts         # 网络切换
│   │   └── useCopy.ts            # 剪贴板操作
│   ├── directives/               # 自定义指令
│   │   ├── debounce.ts           # 防抖
│   │   └── permission.ts         # 权限控制
│   ├── locales/                  # 国际化
│   │   ├── index.ts
│   │   ├── zh-CN.ts
│   │   └── en-US.ts
│   ├── router/                   # 路由
│   │   └── index.ts
│   ├── stores/                   # Pinia stores
│   │   ├── index.ts              # 导出所有 stores
│   │   ├── modules/              # 模块化 stores
│   │   │   ├── account.store.ts
│   │   │   ├── transaction.store.ts
│   │   │   ├── contract.store.ts
│   │   │   ├── node.store.ts
│   │   │   └── ui.store.ts
│   │   └── app.store.ts          # 应用级 store
│   ├── types/                    # 类型定义
│   │   ├── index.ts              # 导出所有类型
│   │   ├── api.ts                # API 响应类型
│   │   ├── business.ts           # 业务实体类型
│   │   ├── enums.ts              # 枚举定义
│   │   └── router.ts             # 路由相关类型
│   ├── utils/                    # 工具函数
│   │   ├── format.ts             # 格式化（金额、时间、地址）
│   │   ├── validator.ts          # 表单验证
│   │   ├── crypto.ts             # 加密/签名
│   │   └── blockchain.ts         # 区块链工具（地址转换、哈希）
│   └── api/                      # API 集成层
│       ├── client.ts             # Axios 实例 + 拦截器
│       ├── error-handler.ts      # 错误处理
│       ├── retry.ts              # 重试逻辑
│       └── modules/              # API 模块
│           ├── index.ts          # 导出所有模块
│           ├── account.api.ts
│           ├── transaction.api.ts
│           ├── contract.api.ts
│           └── node.api.ts
├── .gitignore
└── README.md
```

## 4. 开发规范

### 4.1 TypeScript 严格模式

```jsonc
// tsconfig.json 关键配置
{
  "compilerOptions": {
    "strict": true,                    // 启用所有严格类型检查
    "noImplicitAny": true,            // 禁止隐式 any 类型
    "noImplicitThis": true,           // 禁止 this 隐式 any
    "alwaysStrict": true,             // 以严格模式解析
    "strictNullChecks": true,         // 严格的空值检查
    "strictFunctionTypes": true,      // 严格的函数类型检查
    "noUnusedLocals": true,           // 报告未使用的局部变量
    "noUnusedParameters": true,       // 报告未使用的参数
    "noImplicitReturns": true,        // 函数必须有返回值
    "noFallthroughCasesInSwitch": true // switch 必须 break/return
  }
}
```

### 4.2 代码组织原则

1. **单一职责**：每个 store、每个 API 模块只负责一个业务域
2. **依赖倒置**：页面组件依赖 composables/stores，不直接依赖 API
3. **组合优先**：使用 composables 封装可复用逻辑（如 wallet connect）
4. **类型驱动**：所有 API 响应必须定义 TypeScript 类型

### 4.3 命名约定

| 项目 | 约定 | 示例 |
|------|------|------|
| 组件文件 | PascalCase | `TransactionTable.vue` |
| Store 文件 | `.store.ts` 后缀 | `account.store.ts` |
| Composable 函数 | `use` 前缀 + camelCase | `useWallet`, `useCopy` |
| 工具函数 | camelCase | `formatAddress`, `formatTime` |
| 常量 | UPPER_SNAKE_CASE | `DEFAULT_TIMEOUT` |
| 枚举 | PascalCase + `Enum` | `UserRoleEnum` |
| 类型 | PascalCase + `Type` | `TransactionType` |
| 接口 | PascalCase + `Interface` | `IUserInfo` |

### 4.4 组件开发规范

```vue
<template>
  <!-- 必须 kebab-case -->
  <transaction-table
    :transactions="transactions"
    @refresh="handleRefresh"
  />
</template>

<script setup lang="ts">
// 导入顺序：Vue/TS 内置 -> 第三方 -> 内部
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAccountStore } from '@/stores/modules/account.store'
import { formatAddress } from '@/utils/format'

// 定义 emits
const emit = defineEmits<{
  refresh: []
}>()

// Props（如果需要）
const props = defineProps<{
  transactions: Transaction[]
}>()

// Store 和 composables
const accountStore = useAccountStore()
const router = useRouter()
const { t } = useI18n()

// 响应式数据
const loading = ref(false)

// 计算属性
const formattedList = computed(() =>
  props.transactions.map(tx => ({
    ...tx,
    shortHash: formatAddress(tx.hash)
  }))
)

// 方法
const handleRefresh = async () => {
  loading.value = true
  try {
    await accountStore.fetchTransactions()
    emit('refresh')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped lang="scss">
// SCSS 变量优先从 themes 导入
@use "@/assets/styles/variables" as *;

.transaction-table {
  background: $bg-color;
}
</style>
```

### 4.5 Store 开发规范

```typescript
// stores/modules/account.store.ts
export const useAccountStore = defineStore('account', {
  state: () => ({
    userInfo: null as User | null,
    walletAddress: '',
    accessToken: '',
    roles: [] as UserRole[]
  }),

  getters: {
    isAdmin: (state) => state.roles.includes(UserRole.ADMIN),
    isLoggedIn: (state) => !!state.accessToken
  },

  actions: {
    async login(credentials: LoginCredentials): Promise<void> {
      const response = await accountApi.login(credentials)
      this.userInfo = response.data.user
      this.walletAddress = response.data.wallet_address
      this.accessToken = response.data.access_token
      // 持久化 token（可选）
    },

    async logout(): Promise<void> {
      await accountApi.logout()
      this.$reset()  // 重置所有 state
    },

    async fetchUserInfo(): Promise<void> {
      if (!this.accessToken) return
      const response = await accountApi.getUserInfo()
      this.userInfo = response.data
    }
  },

  // 持久化配置（敏感数据不持久化）
  persist: {
    key: 'account-store',
    storage: localStorage,
    pick: ['accessToken']  // 只持久化 token
  }
})
```

### 4.6 API 模块规范

```typescript
// api/modules/account.api.ts
import type { ApiResponse, User, LoginCredentials } from '@/types'
import request from '@/api/client'

export const accountApi = {
  // 登录
  login(credentials: LoginCredentials) {
    return request.post<ApiResponse<User>>('/api/v1/auth/login', credentials)
  },

  // 获取用户信息
  getUserInfo() {
    return request.get<ApiResponse<User>>('/api/v1/auth/userinfo')
  },

  // 登出
  logout() {
    return request.post<ApiResponse<null>>('/api/v1/auth/logout')
  },

  // 刷新 token
  refreshToken(refreshToken: string) {
    return request.post<ApiResponse<{ access_token: string }>>(
      '/api/v1/auth/refresh',
      { refresh_token: refreshToken }
    )
  }
}
```

### 4.7 错误处理规范

```typescript
// utils/error-handler.ts
export enum ErrorCode {
  NETWORK_ERROR = 'NETWORK_ERROR',
  AUTH_EXPIRED = 'AUTH_EXPIRED',
  INVALID_PARAMS = 'INVALID_PARAMS',
  SERVER_ERROR = 'SERVER_ERROR',
  WEB3_ERROR = 'WEB3_ERROR'
}

export interface ApiError {
  code: ErrorCode
  message: string
  details?: any
}

// 统一错误处理
export function handleApiError(error: any): ApiError {
  if (!error.response) {
    return {
      code: ErrorCode.NETWORK_ERROR,
      message: '网络连接失败，请检查网络'
    }
  }

  const { status, data } = error.response

  switch (status) {
    case 401:
      return {
        code: ErrorCode.AUTH_EXPIRED,
        message: '登录已过期，请重新登录'
      }
    case 400:
      return {
        code: ErrorCode.INVALID_PARAMS,
        message: data.message || '请求参数错误'
      }
    default:
      return {
        code: ErrorCode.SERVER_ERROR,
        message: data.message || '服务器错误'
      }
  }
}
```

### 4.8 路由守卫规范

```typescript
// router/index.ts
router.beforeEach(async (to, from, next) => {
  // 1. 检查是否需要认证
  const requiresAuth = to.meta.requiresAuth ?? true

  if (!requiresAuth) {
    return next()
  }

  // 2. 检查 token
  const token = accountStore.accessToken
  if (!token) {
    return next({ name: 'login', query: { redirect: to.fullPath } })
  }

  // 3. 检查权限（如果有）
  const requiredRoles = to.meta.roles as UserRole[]
  if (requiredRoles && !accountStore.roles.some(r => requiredRoles.includes(r))) {
    return next({ name: '403' })
  }

  next()
})
```

### 4.9 国际化规范

```typescript
// locales/zh-CN.ts
export default {
  common: {
    confirm: '确认',
    cancel: '取消',
    loading: '加载中...',
    success: '成功',
    failed: '失败'
  },
  account: {
    login: '登录',
    logout: '登出',
    walletConnected: '钱包已连接：{address}'
  },
  transaction: {
    send: '发送交易',
    history: '交易历史',
    status: {
      pending: '待确认',
      confirming: '确认中',
      success: '成功',
      failed: '失败'
    }
  }
}
```

## 5. 性能优化建议

1. **路由懒加载**：每个页面独立 chunk（已配置）
2. **组件懒加载**：`defineAsyncComponent` 加载非关键组件
3. **图片优化**：WebP 格式 + `loading="lazy"`
4. **API 缓存**：Pinia state + localStorage（不敏感数据）
5. **按需引入 Element Plus**：使用 `unplugin-vue-components`
6. **Tree Shaking**：ESM 构建 + Vite 天然支持
7. **Gzip 压缩**：Nginx 配置（生产环境）

## 6. 安全规范

1. **Token 存储**：accessToken 存 localStorage（必要），refreshToken 仅存内存
2. **HTTPS 强制**：生产环境强制 HTTPS（vite config 配置）
3. **CSP 策略**：通过 meta 标签配置 Content Security Policy
4. **XSS 防护**：Vue 模板自动转义，避免 `v-html`
5. **敏感日志**：不在控制台输出 token、私钥、钱包地址（开发环境可配置）

## 7. 开发工具配置

```bash
# 代码检查
npm run lint          # ESLint + Vue 3 plugin
npm run lint:fix      # 自动修复

# 代码格式化
npm run format        # Prettier

# 类型检查
npm run type-check    # Vue TS 类型检查

# 构建预览
npm run build         # 生产构建
npm run preview       # 本地预览构建结果
```

## 8. 环境变量规范

```bash
# .env.development
VITE_API_BASE_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8081
VITE_CHAIN_ID=1
VITE_DEBUG=true

# .env.production
VITE_API_BASE_URL=https://api.nrcs.io
VITE_WS_URL=wss://ws.nrcs.io
VITE_CHAIN_ID=1
VITE_DEBUG=false
```

访问：`import.meta.env.VITE_API_BASE_URL`
