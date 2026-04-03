/**
 * 用户模块类型定义
 */

// 用户角色枚举
export enum UserRole {
  ADMIN = 'admin',
  OPERATOR = 'operator',
  VIEWER = 'viewer',
  DEVELOPER = 'developer'
}

// 用户状态枚举
export enum UserStatus {
  ACTIVE = 'active',
  INACTIVE = 'inactive',
  SUSPENDED = 'suspended',
  PENDING = 'pending'
}

// 登录凭证
export interface LoginCredentials {
  username: string
  password: string
  walletSignature?: string  // 钱包签名（可选）
}

// 注册请求
export interface RegisterRequest {
  username: string
  email: string
  password: string
  confirmPassword: string
  fullName?: string
  phone?: string
}

// 登录响应
export interface LoginResponse {
  accessToken: string
  refreshToken: string
  expiresIn: number  // 过期时间（秒）
  user: UserInfo
}

// Token 刷新请求
export interface RefreshTokenRequest {
  refresh_token: string
}

// 用户信息
export interface UserInfo {
  id: string
  username: string
  email: string
  fullName: string
  avatar?: string
  phone?: string
  roles: UserRole[]
  permissions: string[]
  status: UserStatus
  department?: string
  createdAt: string
  updatedAt: string
  lastLoginAt?: string
}

// 修改密码请求
export interface ChangePasswordRequest {
  oldPassword: string
  newPassword: string
  confirmPassword: string
}

// 更新个人信息请求
export interface UpdateProfileRequest {
  fullName?: string
  email?: string
  phone?: string
  avatar?: string
}

// 权限详情
export interface Permission {
  id: string
  name: string
  code: string
  description?: string
  resource: string
  action: 'read' | 'write' | 'delete' | 'manage'
}

// 会话信息
export interface SessionInfo {
  sessionId: string
  userId: string
  ipAddress: string
  userAgent: string
  loginAt: string
  expiresAt: string
}

// 认证上下文
export interface AuthContext {
  user: UserInfo | null
  token: string | null
  refreshToken: string | null
  permissions: string[]
  isAuthenticated: boolean
  isLoading: boolean
}

// 权限检查结果
export interface PermissionCheck {
  hasPermission: boolean
  requiredRole?: UserRole
  requiredPermission?: string
  userRoles: UserRole[]
  userPermissions: string[]
}
