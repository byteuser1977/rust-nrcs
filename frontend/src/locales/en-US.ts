export default {
  // Common
  common: {
    confirm: 'Confirm',
    cancel: 'Cancel',
    save: 'Save',
    delete: 'Delete',
    edit: 'Edit',
    submit: 'Submit',
    reset: 'Reset',
    search: 'Search',
    filter: 'Filter',
    refresh: 'Refresh',
    loading: 'Loading...',
    success: 'Success',
    failed: 'Failed',
    error: 'Error',
    warning: 'Warning',
    info: 'Info',
    noData: 'No Data',
    noMore: 'No More',
    confirmDelete: 'Are you sure to delete?',
    operationSuccess: 'Operation successful',
    operationFailed: 'Operation failed',
    networkError: 'Network error',
    serverError: 'Server error',
    unknownError: 'Unknown error'
  },

  // Menu
  menu: {
    dashboard: 'Dashboard',
    account: 'Account',
    transaction: 'Transaction',
    contract: 'Contract',
    node: 'Node Monitor'
  },

  // Login/Register
  login: {
    title: 'NRCS Blockchain Platform',
    subtitle: 'Neo Rapid BlockChain EcoSystem',
    walletAddress: 'Wallet Address',
    connectWallet: 'Connect Wallet',
    signMessage: 'Sign Message',
    sign: 'Sign',
    signPrompt: 'Please sign the message below to complete login',
    loginSuccess: 'Login successful',
    loginFailed: 'Login failed',
    noWallet: 'No wallet detected',
    installWallet: 'Install Wallet Extension',
    register: 'Register',
    name: 'Username',
    email: 'Email',
    password: 'Password',
    confirmPassword: 'Confirm Password',
    termsAgree: 'I agree to the Terms of Service',
    registerSuccess: 'Registration successful'
  },

  // Dashboard
  dashboard: {
    title: 'Dashboard',
    totalBalance: 'Total Balance',
    transactions: 'Transactions',
    contracts: 'Contracts',
    nodes: 'Node Status',
    recentTxs: 'Recent Transactions',
    viewAll: 'View All',
    networkStats: 'Network Stats',
    blockHeight: 'Block Height',
    tps: 'TPS',
    difficulty: 'Difficulty',
    gasPrice: 'Gas Price',
    peers: 'Peers',
    uptime: 'Uptime',
    health: 'Health',
    healthy: 'Healthy',
    unhealthy: 'Unhealthy'
  },

  // Account
  account: {
    profile: 'Profile',
    security: 'Security',
    wallet: 'Wallet',
    address: 'Address',
    balance: 'Balance',
    role: 'Role',
    createdAt: 'Created At',
    updatedAt: 'Updated At',
    editProfile: 'Edit Profile',
    changePassword: 'Change Password',
    disconnectWallet: 'Disconnect Wallet',
    disconnectConfirm: 'Are you sure to disconnect wallet?'
  },

  // Transaction
  transaction: {
    history: 'Transaction History',
    send: 'Send Transaction',
    pending: 'Pending Transactions',
    hash: 'Hash',
    block: 'Block',
    from: 'From',
    to: 'To',
    value: 'Value',
    gasPrice: 'Gas Price',
    gasUsed: 'Gas Used',
    status: 'Status',
    time: 'Time',
    success: 'Success',
    failed: 'Failed',
    confirming: 'Confirming',
    pending: 'Pending',
    sendSuccess: 'Transaction sent successfully',
    sendFailed: 'Transaction failed',
    estimateGas: 'Estimate Gas',
    maxFee: 'Max Fee',
    maxPriorityFee: 'Max Priority Fee',
    nonce: 'Nonce',
    data: 'Data',
    amount: 'Amount (ETH)',
    recipient: 'Recipient Address',
    send: 'Send',
    cancel: 'Cancel',
    estimate: 'Estimate',
    txSent: 'Transaction sent',
    txConfirmed: 'Transaction confirmed',
    viewOnExplorer: 'View on Explorer'
  },

  // Contract
  contract: {
    list: 'Contract List',
    deploy: 'Deploy Contract',
    detail: 'Contract Detail',
    name: 'Contract Name',
    address: 'Address',
    abi: 'ABI',
    bytecode: 'Bytecode',
    deployer: 'Deployer',
    deployedAt: 'Deployed At',
    verified: 'Verified',
    unverified: 'Unverified',
    verify: 'Verify Source',
    sourceCode: 'Source Code',
    functions: 'Functions',
    events: 'Events',
    read: 'Read',
    write: 'Write',
    call: 'Call',
    estimateGas: 'Estimate Gas',
    result: 'Result',
    input: 'Input',
    deploySuccess: 'Contract deployed successfully',
    callSuccess: 'Call successful',
    callFailed: 'Call failed',
    verifySuccess: 'Verification successful',
    verifyFailed: 'Verification failed'
  },

  // Node
  node: {
    status: 'Node Status',
    blocks: 'Block Explorer',
    info: 'Node Info',
    version: 'Version',
    network: 'Network',
    peers: 'Peers',
    blocks: 'Latest Blocks',
    health: 'Health Status',
    sync: 'Sync Status',
    noBlocks: 'No block data',
    loadMore: 'Load More',
    refresh: 'Refresh',
    synced: 'Synced',
    syncing: 'Syncing',
    caughtUp: 'Caught Up',
    stats: 'Statistics',
    totalTxs: 'Total Transactions',
    avgBlockTime: 'Avg Block Time',
    difficulty: 'Difficulty',
    tps: 'TPS',
    gasStats: 'Gas Stats',
    txPool: 'Transaction Pool'
  },

  // Error
  error: {
    notFound: 'Page Not Found',
    notFoundDesc: 'The page you are looking for does not exist',
    backHome: 'Back to Home',
    forbidden: 'Access Forbidden',
    forbiddenDesc: 'You do not have permission to access this page',
    requestTimeout: 'Request Timeout',
    serverError: 'Server Error',
    networkError: 'Network Error'
  },

  // Validation
  validation: {
    required: 'Required field',
    invalidEmail: 'Invalid email format',
    passwordTooShort: 'Password must be at least 8 characters',
    passwordMismatch: 'Passwords do not match',
    invalidAddress: 'Invalid address format',
    invalidAmount: 'Invalid amount format',
    insufficientBalance: 'Insufficient balance'
  },

  // Copy
  copy: {
    copied: 'Copied',
    copiedToClipboard: 'Copied to clipboard',
    copyFailed: 'Copy failed'
  }
}
