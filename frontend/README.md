# NRCS 区块链平台 - 前端

基于 Vue 3 + TypeScript + Vite + Pinia + Element Plus 构建的区块链平台前端应用。

## 技术栈

- **Vue 3.4+** - 渐进式框架
- **TypeScript 5+** - 类型安全的 JavaScript
- **Vite 5+** - 下一代前端构建工具
- **Vue Router 4+** - 官方路由
- **Pinia 2+** - 状态管理
- **Element Plus** - Vue 3 UI 组件库
- **Vue I18n** - 国际化
- **Axios** - HTTP 客户端

## 项目结构

```
frontend/
├── docs/                          # 文档
│   ├── frontend-architecture.md  # 前端架构设计
│   └── api-integration-design.md # API 集成设计
├── public/                        # 静态资源
├── src/
│   ├── api/                       # API 集成层
│   │   ├── client.ts             # Axios 实例配置
│   │   └── modules/              # API 模块
│   ├── assets/                    # 静态资源
│   │   ├── styles/               # 全局样式
│   │   └── images/               # 图片
│   ├── components/               # 通用组件
│   │   └── layout/               # 布局组件
│   ├── composables/              # 组合式函数
│   ├── locales/                  # 国际化语言包
│   ├── router/                   # 路由配置
│   ├── stores/                   # Pinia stores
│   │   └── modules/              # 模块化 stores
│   ├── types/                    # TypeScript 类型
│   ├── utils/                    # 工具函数
│   ├── views/                    # 页面视图
│   ├── App.vue                   # 根组件
│   └── main.ts                   # 应用入口
├── .env.example                  # 环境变量示例
├── .gitignore                    # Git 忽略文件
├── index.html                    # HTML 模板
├── package.json                  # 依赖配置
├── tsconfig.json                 # TypeScript 配置
└── vite.config.ts                # Vite 配置
```

## 快速开始

### 环境要求

- Node.js >= 18.0.0
- pnpm / npm / yarn

### 安装依赖

```bash
npm install
# 或
pnpm install
# 或
yarn install
```

### 环境配置

复制 `.env.example` 为 `.env.development` 并修改配置：

```bash
cp .env.example .env.development
```

```env
VITE_API_BASE_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8081
VITE_CHAIN_ID=1
VITE_DEBUG=true
```

### 开发服务器

```bash
npm run dev
# 或
pnpm dev
```

应用将在 [http://localhost:3000](http://localhost:3000) 启动。

### 构建

```bash
npm run build
# 或
pnpm build
```

构建产物将输出到 `dist/` 目录。

### 代码检查

```bash
npm run lint        # 修复问题
npm run lint:check  # 仅检查
npm run format      # 格式化代码
```

### 类型检查

```bash
npm run type-check
```

### 测试

```bash
npm run test
```

## 开发指南

### 路由结构

- `/login` - 登录页
- `/register` - 注册页
- `/` (需要认证)
  - `/dashboard` - 仪表盘
  - `/account/profile` - 个人信息
  - `/account/security` - 安全设置
  - `/transaction/history` - 交易历史
  - `/transaction/send` - 发送交易
  - `/transaction/pending` - 待确认交易
  - `/contract/list` - 合约列表
  - `/contract/deploy` - 部署合约
  - `/contract/detail/:address` - 合约详情
  - `/node/status` - 节点状态（需管理员）
  - `/node/blocks` - 区块浏览（需管理员）
- `/404` - 页面未找到
- `/403` - 权限不足

### 状态管理 (Pinia)

所有 Store 位于 `src/stores/modules/`：

- `account.store.ts` - 账户信息、认证状态
- `transaction.store.ts` - 交易数据管理
- `contract.store.ts` - 合约数据管理
- `node.store.ts` - 节点监控数据
- `ui.store.ts` - UI 状态（侧边栏、主题等）

### API 集成

API 模块位于 `src/api/modules/`，使用统一的 Axios 实例：

```typescript
import { accountApi } from '@/api/modules'

// 调用示例
const user = await accountApi.login({ wallet_address, signature, message })
```

错误处理由拦截器统一处理，业务错误码需符合后端规范。

### 工具函数

常用工具位于 `src/utils/`：

- `format.ts` - 格式化工具（地址、金额、时间）
- `crypto.ts` - 加密/区块链相关工具
- 后续可添加 `validator.ts` - 表单验证

### 国际化

语言包位于 `src/locales/`：

- `zh-CN.ts` - 简体中文
- `en-US.ts` - 英文

使用示例：

```vue
<script setup>
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <div>{{ t('common.confirm') }}</div>
</template>
```

### 样式定制

全局 SCSS 变量位于 `src/assets/styles/variables.scss`，可修改以下主题：

```scss
$primary-color: #409eff;     // 主色
$bg-color: #0a0a0a;          // 背景色
$text-primary: #e5e5e5;      // 文本色
// ... 更多变量
```

### 开发规范

1. **TypeScript** - 严格模式（strict mode）
2. **组件命名** - PascalCase（如 `TransactionTable.vue`）
3. **文件名** - kebab-case（如 `transaction-list.vue`）
4. **Store 文件** - 使用 `.store.ts` 后缀
5. **Composable** - `use` 前缀（如 `useWallet`）

详细规范见 `docs/frontend-architecture.md`。

## 贡献指南

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

MIT

## 联系

- Issue: [GitHub Issues](https://github.com/your-repo/issues)
- Email: dev@nrcs.io
