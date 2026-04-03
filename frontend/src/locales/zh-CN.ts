export default {
  // 通用
  common: {
    confirm: '确认',
    cancel: '取消',
    save: '保存',
    delete: '删除',
    edit: '编辑',
    submit: '提交',
    reset: '重置',
    search: '搜索',
    filter: '筛选',
    refresh: '刷新',
    loading: '加载中...',
    success: '成功',
    failed: '失败',
    error: '错误',
    warning: '警告',
    info: '提示',
    noData: '暂无数据',
    noMore: '没有更多数据',
    confirmDelete: '确定要删除吗？',
    operationSuccess: '操作成功',
    operationFailed: '操作失败',
    networkError: '网络连接失败',
    serverError: '服务器错误',
    unknownError: '未知错误'
  },

  // 菜单
  menu: {
    dashboard: '仪表盘',
    account: '账户',
    transaction: '交易',
    contract: '合约',
    node: '节点监控'
  },

  // 登录/注册
  login: {
    title: 'NRCS 区块链平台',
    subtitle: 'Neo Rapid BlockChain EcoSystem',
    walletAddress: '钱包地址',
    connectWallet: '连接钱包',
    signMessage: '签名消息',
    sign: '签名',
    signPrompt: '请使用钱包签名以下消息以完成登录',
    loginSuccess: '登录成功',
    loginFailed: '登录失败',
    noWallet: '未检测到钱包',
    installWallet: '安装钱包插件',
    register: '注册账号',
    name: '用户名',
    email: '邮箱',
    password: '密码',
    confirmPassword: '确认密码',
    termsAgree: '我已阅读并同意服务条款',
    registerSuccess: '注册成功'
  },

  // 仪表盘
  dashboard: {
    title: '仪表盘',
    totalBalance: '总余额',
    transactions: '交易统计',
    contracts: '合约数量',
    nodes: '节点状态',
    recentTxs: '最近交易',
    viewAll: '查看全部',
    networkStats: '网络统计',
    blockHeight: '区块高度',
    tps: 'TPS',
    difficulty: '难度',
    gasPrice: 'Gas 价格',
    peers: '节点连接数',
    uptime: '运行时间',
    health: '健康状态',
    healthy: '健康',
    unhealthy: '异常'
  },

  // 账户
  account: {
    profile: '个人信息',
    security: '安全设置',
    wallet: '钱包',
    address: '地址',
    balance: '余额',
    role: '角色',
    createdAt: '创建时间',
    updatedAt: '更新时间',
    editProfile: '编辑资料',
    changePassword: '修改密码',
    disconnectWallet: '断开钱包连接',
    disconnectConfirm: '确定要断开钱包连接吗？'
  },

  // 交易
  transaction: {
    history: '交易历史',
    send: '发送交易',
    pending: '待确认交易',
    hash: '哈希',
    block: '区块',
    from: '发送方',
    to: '接收方',
    value: '金额',
    gasPrice: 'Gas 价格',
    gasUsed: 'Gas 使用',
    status: '状态',
    time: '时间',
    success: '成功',
    failed: '失败',
    confirming: '确认中',
    pending: '待确认',
    sendSuccess: '交易发送成功',
    sendFailed: '交易发送失败',
    estimateGas: '预估 Gas',
    maxFee: '最高费用',
    maxPriorityFee: '优先费用',
    nonce: 'Nonce',
    data: '数据',
    amount: '金额 (ETH)',
    recipient: '收款地址',
    send: '发送',
    cancel: '取消',
    estimate: '预估',
    txSent: '交易已发送',
    txConfirmed: '交易已确认',
    viewOnExplorer: '在区块链浏览器查看'
  },

  // 合约
  contract: {
    list: '合约列表',
    deploy: '部署合约',
    detail: '合约详情',
    name: '合约名称',
    address: '合约地址',
    abi: 'ABI',
    bytecode: '字节码',
    deployer: '部署者',
    deployedAt: '部署时间',
    verified: '已验证',
    unverified: '未验证',
    verify: '验证源码',
    sourceCode: '源代码',
    functions: '可调用方法',
    events: '事件',
    read: '只读调用',
    write: '写入交易',
    call: '调用',
    estimateGas: '预估 Gas',
    result: '返回值',
    input: '输入参数',
    deploySuccess: '合约部署成功',
    callSuccess: '调用成功',
    callFailed: '调用失败',
    verifySuccess: '验证成功',
    verifyFailed: '验证失败'
  },

  // 节点
  node: {
    status: '节点状态',
    blocks: '区块浏览',
    info: '节点信息',
    version: '版本',
    network: '网络',
    peers: '节点连接',
    blocks: '最新区块',
    health: '健康状态',
    sync: '同步状态',
    noBlocks: '暂无区块数据',
    loadMore: '加载更多',
    refresh: '刷新',
    synced: '已同步',
    syncing: '同步中',
    caughtUp: '已追上',
    stats: '统计信息',
    totalTxs: '总交易数',
    avgBlockTime: '平均出块时间',
    difficulty: '难度',
    tps: 'TPS',
    gasStats: 'Gas 统计',
    txPool: '交易池'
  },

  // 错误页面
  error: {
    notFound: '页面未找到',
    notFoundDesc: '您访问的页面不存在',
    backHome: '返回首页',
    forbidden: '权限不足',
    forbiddenDesc: '您没有权限访问此页面',
    requestTimeout: '请求超时',
    serverError: '服务器错误',
    networkError: '网络错误'
  },

  // 验证
  validation: {
    required: '必填项',
    invalidEmail: '邮箱格式不正确',
    passwordTooShort: '密码至少 8 位',
    passwordMismatch: '两次密码不一致',
    invalidAddress: '地址格式不正确',
    invalidAmount: '金额格式不正确',
    insufficientBalance: '余额不足'
  },

  // 复制
  copy: {
    copied: '已复制',
    copiedToClipboard: '已复制到剪贴板',
    copyFailed: '复制失败'
  }
}
