export const zh = {
  // Common
  loading: '加载中...',
  cancel: '取消',
  back: '返回',
  close: '关闭',
  rename: '重命名',
  saving: '保存中...',

  // Sidebar
  allAccounts: '全部账号',
  groups: '分组',
  tags: '标签',
  newGroupName: '新分组名称',
  deleteGroup: '删除分组',
  deleteGroupConfirm: '确定要删除该分组吗？账号将移至未分组。',
  lightMode: '浅色模式',
  darkMode: '深色模式',

  // Header
  searchPlaceholder: '搜索账号、标签或备注...',
  addAccount: '添加账号',

  // AccountGrid
  noAccounts: '暂无账号',
  noAccountsHint: '点击右上角"添加账号"开始管理',

  // AccountCard
  deleteAccountConfirm: '确定要删除该账号吗？',
  copyUsername: '复制用户名',
  editInfo: '编辑信息',
  moveToGroup: '移动到分组',
  ungroup: '取消分组',
  deleteAccount: '删除账号',
  switching: '切换中...',
  useThisAccount: '使用此账号',
  inUse: '使用中',

  // AccountModal
  noGroup: '不指定分组',
  enterBattleTag: '请填写战网ID',
  operationFailed: '操作失败，请重试',
  selectAction: '选择操作',
  editAccount: '编辑账号',
  addAccountTitle: '录入账号',
  selectActionDesc: '选择您要执行的操作',
  editAccountDesc: '修改账号信息，分组可随时修改。',
  addAccountDesc: '填写账号信息，分组可随时修改。',
  saveCurrentState: '保存当前状态',
  saveCurrentStateDesc: '提取当前战网已登录的配置文件并永久保存',
  loginNewAccount: '前往登录新号',
  loginNewAccountDesc: '强制关闭当前战网，让你能够输入新的账密',
  battleTag: '战网ID',
  battleTagPlaceholder: '例如: Player#1234',
  belongsToGroup: '所属分组',
  saveToGroup: '保存到分组',
  optionalPressEnter: '(选填，按 Enter 添加)',
  enterTagName: '输入标签名称',
  accountRemark: '账号备注',
  remarkPlaceholder: '例如: 未定级',
  saveChanges: '保存修改',
  confirmSave: '确定保存',

  // App.tsx
  saveFailed: '保存失败，请重试',
  saveFailedLogin: '保存失败，请确认您已在战网客户端完成登录',

  // UpdateModal
  newVersion: '发现新版本',
  updateAvailable: '有新的更新可用',
  newVersionLabel: '新版本',
  releaseDate: '发布时间',
  updateNotes: '更新内容',
  updateDownloaded: '更新已下载，应用将自动重启完成安装',
  updateLater: '稍后更新',
  downloading: '下载中...',
  updateNow: '立即更新',
  installFailed: '安装失败，请重试或手动下载更新',
} as const

export type TranslationKey = keyof typeof zh
